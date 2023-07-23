use std::path::PathBuf;

pub(crate) fn path_is_dir(input: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(input);
    if !path.is_dir() {
        return Err(String::from("The given path needs to be an directory"));
    }
    Ok(path)
}
