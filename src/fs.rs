use std::{
    fs::{self, File},
    io::Error,
    path::Path,
};

pub fn open_file(file_path: String) -> Result<File, Error> {
    if let Some(parent_dir) = Path::new(&file_path).parent() {
        fs::create_dir_all(parent_dir)?;
    }

    let file = File::create(file_path)?;

    Ok(file)
}
