fn main() -> anyhow::Result<()> {
    let is_cross_compiling = detect_cross_compilation();

    if !is_cross_compiling {
        println!("cargo:rustc-cfg=feature=\"use_glib_build_tools\"");
    }

    compile_gresources(is_cross_compiling)?;

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
        .context("Failed to compile winresource resource")
}

fn detect_cross_compilation() -> bool {
    let host = std::env::var("HOST").unwrap_or_default();
    let target = std::env::var("TARGET").unwrap_or_default();
    host != target
}

fn compile_gresources(is_cross_compiling: bool) -> anyhow::Result<()> {
    use std::env;
    use std::path::PathBuf;
    use std::process::Command;

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let output_path = PathBuf::from(&out_dir).join("compiled.gresource");

    // First, try using the system's glib-compile-resources
    let system_result = Command::new("glib-compile-resources")
        .args(&[
            "--sourcedir=data",
            "data/resources.gresource.xml",
            &format!("--target={}", output_path.display()),
        ])
        .status();

    match system_result {
        Ok(status) if status.success() => return Ok(()),
        Ok(_) => println!("glib-compile-resources command failed, trying fallback method..."),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("glib-compile-resources not found, trying fallback method...")
        }
        Err(e) => println!(
            "Error executing glib-compile-resources: {}, trying fallback method...",
            e
        ),
    }

    // If cross-compiling, don't use glib_build_tools
    if is_cross_compiling {
        return Err(anyhow::anyhow!(
            "Failed to compile gresources: system glib-compile-resources failed and we're cross-compiling. \
            Please ensure you have glib development tools installed on your target system."
        ));
    }

    // If not cross-compiling and system command fails, fall back to glib_build_tools if available
    #[cfg(feature = "use_glib_build_tools")]
    {
        println!("Attempting to use glib_build_tools::compile_resources...");
        glib_build_tools::compile_resources(
            &["data"],
            "data/resources.gresource.xml",
            output_path.to_str().unwrap(),
        );
        Ok(())
    }

    #[cfg(not(feature = "use_glib_build_tools"))]
    Err(anyhow::anyhow!(
        "Failed to compile gresources: system glib-compile-resources failed and glib_build_tools is not available. \
        Please ensure you have glib development tools installed on your system."
    ))
}
