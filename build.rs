use std::{ io, fs };
use simple_home_dir::home_dir;

fn main() -> io::Result<()> {
    let mut path_to_config_file = home_dir().expect("Couldn’t get your home dir");
    path_to_config_file.push(".config/just_code.toml");

    if path_to_config_file.exists() {
        println!("cargo:warning=Couldn’t create `~/.config/just_code.toml`, file already exist");
    }
    else {
        fs::copy("just_code.toml", &path_to_config_file)?;
    }

    Ok(())
}
