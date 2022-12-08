#[cfg(target_os = "windows")]
extern crate winres;

fn main() -> anyhow::Result<()> {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("./data/icons/rnote.ico");
        res.compile()?;
    }

    Ok(())
}
