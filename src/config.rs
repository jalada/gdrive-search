extern crate dirs;
use dirs::config_dir;
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = "gdrive-search";

pub fn setup() -> std::io::Result<()> {
    // Make sure we can create (or there exists) a configuration directory
    let path = config_dir();

    match path {
        Some(mut p) => {
            p.push(CONFIG_DIR);
            fs::create_dir_all(p.as_path())
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not get suitable config dir",
        )),
    }
}

pub fn config_path(file: &str) -> PathBuf {
    let mut path = config_dir().unwrap();
    path.push(CONFIG_DIR);
    path.push(file);

    path
}
