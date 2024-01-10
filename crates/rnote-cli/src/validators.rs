use std::path::Path;

pub(crate) fn path_is_dir(path: &Path) -> anyhow::Result<()> {
    if !path.is_dir() {
        return Err(anyhow::anyhow!(
            "Expected directory, found file \"{}\"",
            path.display()
        ));
    }
    Ok(())
}

pub(crate) fn path_is_file(path: &Path) -> anyhow::Result<()> {
    if !path.is_file() {
        return Err(anyhow::anyhow!(
            "Expected file, found directory \"{}\"",
            path.display()
        ));
    }
    Ok(())
}

pub(crate) fn file_has_ext(path: &Path, expected_ext: &str) -> anyhow::Result<()> {
    path_is_file(path)?;
    match path.extension() {
        Some(ext) if ext == expected_ext => Ok(()),
        Some(ext) => Err(anyhow::anyhow!(
            "Expected file with extension \"{expected_ext}\", found extension \"{ext:?}\", file \"{}\".",
            path.display()
        )),
        None => Err(anyhow::anyhow!(
            "Expected file with extension \"{expected_ext}\", no extension found for file \"{}\".",
            path.display()
        ))
    }
}
