use std::{
    fs,
    env,
    boxed::Box,
    error::Error,
    path::PathBuf,
    process::Command,
    io::{ self, Write },
    os::unix::fs::PermissionsExt,
};
use toml::Table;
use simple_home_dir::home_dir;
use git2::Repository;
use tuple_map::TupleMap2;
use convert_case::{ Case, Converter };

/// A struct representing a file needed to be generated
struct NewFile {
    /// Path of the file (without the extention)
    path: String,
    /// Extention of that file, used in part to get the correct template
    extention: String,
    /// Specifies wheather or not we should discard the extention in generated file
    no_extention: bool,
}

/// Error set for the NewFile::Create method.
#[derive(Debug, thiserror::Error)]
enum CreateError {
    /// An IO error accured
    #[error("An IO error accured")]
    IO(#[from] io::Error),
    /// A file with the same name already exists
    #[error("The file `{0}` already exists")]
    FileExists(String),
    /// Template doesn’t exist for that extention
    #[error("Template for file extention `{0}` doesn’t exist")]
    TemplateDoesntExist(String),
    #[error("Template for file extention `{0}` has an invalid type, should be a string")]
    InvalidValueType(String),
}

impl NewFile {
    fn to_string(&self) -> String {
        let mut rv = self.path.clone();
        if !self.no_extention {
            rv += ".";
            rv += &self.extention;
        }
        rv
    }

    fn create(&self, templates: &Table, executable_file_extentions: &[String]) -> Result<(), CreateError> {
        let full_path_to_file: PathBuf = {
            let mut tmp: PathBuf = env::current_dir()?;
            tmp.push(self.to_string());
            tmp
        };

        if full_path_to_file.exists() {
            let mut full_file_name = self.path.clone();
            full_file_name += ".";
            full_file_name += &self.extention;
            return Err(CreateError::FileExists(full_file_name))
        }

        let Some(toml_value) = templates.get(&self.extention) else {
            return Err(CreateError::TemplateDoesntExist(self.extention.clone()))
        };

        let toml::Value::String(mut file_contents) = toml_value.clone() else {
            return Err(CreateError::InvalidValueType(self.extention.clone()))
        };

        for case in Case::deterministic_cases().iter() {
            let converter = Converter::new().to_case(*case);
            let pattern = format!("${}$", converter.convert("file name"));
            if file_contents.contains(&pattern) {
                let path = PathBuf::from(self.path.clone());
                let formatted_file_name = converter.convert(
                        path.file_name().unwrap().to_str().unwrap()
                );
                file_contents = file_contents.replace(&pattern, &formatted_file_name);
            }
        }

        let mut new_file = fs::File::create(&full_path_to_file)?;
        let mut permissions = new_file.metadata()?.permissions();

        if executable_file_extentions.iter().any(|ext| ext == &self.extention) {
            permissions.set_mode(0o744);  // Read/Write/Execute for user, ReadOnly for others
        } else {
            permissions.set_mode(0o644);  // Read/Writefor user, ReadOnly for others
        }
        new_file.set_permissions(permissions)?;

        new_file.write_all(file_contents.as_ref())?;

        Ok(())
    }
}

fn get_templates() -> Result<(Vec<String>, Table), Box<dyn Error>> {
    let path_to_config_file = {
        let mut tmp = home_dir().unwrap();
        tmp.push(".config/just_code.toml");
        tmp
    };

    let mut templates = fs::read_to_string(&path_to_config_file)?.parse::<Table>()?;
    let exec_file_ext = {
        let mut tmp = Vec::<String>::new();
        if let Some(toml::Value::Array(arr)) = templates.remove("executable_file_extentions") {
            for item in arr.iter() {
                let toml::Value::String(s) = item else {
                    panic!("Invalid value in `executable_file_extentions` : {item}");
                };

                tmp.push(s.clone());
            }
        }
        tmp
    };

    Ok((exec_file_ext, templates))
}

struct Args {
    files_to_process: Vec<NewFile>,
    create_git_repo: bool,
    no_editor: bool,
    extra_editor_args: Vec<String>,
}

fn parse_args(args: &[String]) -> Option<Args> {
    let mut files_to_process = Vec::<NewFile>::new();
    let mut create_git_repo = false;
    let mut no_editor = false;

    let cli_args: &[String];
    let extra_editor_args: Vec<String>;
    if let Some(separator) = args.iter().position(|s| s == "--") {
        unsafe {
            cli_args = args.get_unchecked(..separator);
            extra_editor_args = args.get_unchecked(separator + 1..).into();
        }
    }
    else {
        cli_args = args;
        extra_editor_args = Vec::new();
    }

    for item in cli_args {
        if item == "-g" || item == "--create-git-repo" {
            create_git_repo = true;
            continue
        }

        if item == "-n" || item == "--no-editor" {
            no_editor = true;
            continue
        }

        let no_extention = item.starts_with("re:");
        let (path, extention): (String, String) =
            item.trim_start_matches("re:")
                .rsplit_once('.')
                .unwrap_or((item.as_str(), ""))
                .map(|e| e.into());

        files_to_process.push(NewFile { path, extention, no_extention });
    }

    Some(Args {
        files_to_process,
        create_git_repo,
        no_editor,
        extra_editor_args,
    })
}

fn show_help() {
    println!(r#"
just-code: A minimalist helloworld generator
============================================

usage: just-code [-g|--create-git-repo] [-n|--no-editor] ([re:]file_name)+
                 -- [extra editor args]

This programm creates all of the files_names given as argument, copies the
template from the ~/.config/just_code.toml file, then opens your code editor
with all file names as argument. The editor is set in "$EDITOR" env var.

The programm will replace any occurences of `$file name$` with the name of the
newly created file, keeping the same case style used in `$file name$`.
For example, if you are creating a file called `my_module.py` :

- `$file name$` will become `my module`
- `$FILE_NAME$` will become `MY_MODULE`
- `$FileName$`  will become `MyModule`…

Prefixing a file name with 're:' will remove the extention, for example :
`just-code re:hello.sh` will generate and open `hello` instead of `hello.sh`
but the file will still be handled as a `sh` file.

Extra arguments can be passed to the editor by writing them after `--`.
They will be added before the file names, for instance, if my EDITOR="nvim",
`just-code main.py module.py -- -O` will launch `nvim -O main.py module.py`

The `-n` flag prevents the editor from opening the new files.

The `-g` flag creates a new git repo"#);
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd_line_args: Vec<String> = {
        let mut tmp = env::args();
        tmp.next();  // skip programm name, don’t care about it
        tmp.collect()
    };

    if cmd_line_args.len() == 0 || cmd_line_args.iter().any(|s| s == "-h" || s == "--help") {
        show_help();
        return Ok(())
    }

    let Some(args) = parse_args(&cmd_line_args) else {
        panic!("-n flag needs to be after the file path")
    };

    let (exec_file_ext, templates) = get_templates()?;
    args.files_to_process.iter()
        .try_for_each(|file| file.create(&templates, &exec_file_ext))?;

    if args.create_git_repo {
        Repository::init(env::current_dir()?)?;
    }

    if !args.no_editor {
        let mut editor_args = args.extra_editor_args.clone();
        args.files_to_process.iter().for_each(|f| editor_args.push(f.to_string()));
        Command::new(env::var("EDITOR")?).args(editor_args).spawn()?.wait()?;
    }
    Ok(())
}
