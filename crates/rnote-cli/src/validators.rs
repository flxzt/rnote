use std::path::Path;

pub(crate) fn path_is_dir(path: &Path) -> anyhow::Result<()> {
    if !path.is_dir() {
        return Err(anyhow::anyhow!(
            "expected directory, found file {}",
            path.display()
        ));
    }
    Ok(())
}

pub(crate) fn path_is_file(path: &Path) -> anyhow::Result<()> {
    if !path.is_file() {
        return Err(anyhow::anyhow!(
            "expected file, found directory {}",
            path.display()
        ));
    }
    Ok(())
}

pub(crate) fn file_has_ext(path: &Path, expected_ext: &str) -> anyhow::Result<()> {
    path_is_file(path)?;
    match path.extension() {
        Some(ext) if ext == expected_ext => Ok(()),
        _ => Err(anyhow::anyhow!(
            "expected .{expected_ext} file, found {}",
            path.display()
        )),
    }
}
