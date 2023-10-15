just-code: A minimalist helloworld generator
============================================

```
usage: just-code [-g|--create-git-repo] [-n|--no-editor] ([re:]file_name)+
                 -- [extra editor args]
```

This programm creates all of the files_names given as argument, copies the
template from the `~/.config/just_code.toml` file, then opens your code editor
with all file names as argument. The editor is set in "$EDITOR" env var.

The programm will replace any occurences of `$file name$` with the name of the
newly created file, keeping the same case style used in `$file name$`.
For example, if you are creating a file called `my_module.py` :

- `$file name$` will become `my module`
- `$FILE_NAME$` will become `MY_MODULE`
- `$FileName$`  will become `MyModule`…

Prefixing a file name with `re:` will remove the extention, for example :
`just-code re:hello.sh` will generate and open `hello` instead of `hello.sh`
but the file will still be handled as a `sh` file.

Extra arguments can be passed to the editor by writing them after `--`.
They will be added before the file names, for instance, if my EDITOR="nvim",
`just-code main.py module.py -- -O` will launch `nvim -O main.py module.py`

The `-n` flag prevents the editor from opening the new files.

The `-g` flag creates a new git repo
