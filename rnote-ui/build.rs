fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    compile_icon_winres()?;

    Ok(())
}

#[cfg(windows)]
fn compile_icon_winres() -> anyhow::Result<()> {
    use anyhow::Context;

    let mut res = winresource::WindowsResource::new();
    res.set("OriginalFileName", "rnote.exe");
    res.set_icon("./data/icons/rnote.ico");
    res.compile()
        .context("Failed to compile winresource resource.")
}
