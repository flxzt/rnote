// Imports
use crate::config;
use anyhow::Context;
use path_absolutize::Absolutize;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::{env, fs};
use tracing::debug;

pub(crate) fn lib_dir() -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let exec_dir = exec_parent_dir()?;

        Ok(exec_dir.join("..\\lib"))
    } else if cfg!(target_os = "macos") {
        let canonicalized_exec_dir = exec_parent_dir()?.canonicalize()?;

        if macos_is_in_app_bundle(&canonicalized_exec_dir) {
            Ok(canonicalized_exec_dir.join("../Frameworks"))
        } else {
            Ok(PathBuf::from(config::LIBDIR))
        }
    } else {
        Ok(PathBuf::from(config::LIBDIR))
    }
}

pub(crate) fn data_dir() -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let exec_dir = exec_parent_dir()?;

        Ok(exec_dir.join("..\\share"))
    } else if cfg!(target_os = "macos") {
        let canonicalized_exec_dir = exec_parent_dir()?.canonicalize()?;

        if macos_is_in_app_bundle(&canonicalized_exec_dir) {
            Ok(canonicalized_exec_dir.join("../Resources/share"))
        } else {
            Ok(PathBuf::from(config::DATADIR))
        }
    } else {
        Ok(PathBuf::from(config::DATADIR))
    }
}

pub(crate) fn pkg_data_dir() -> anyhow::Result<PathBuf> {
    Ok(data_dir()?.join(config::APP_NAME))
}

pub(crate) fn locale_dir() -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let exec_dir = exec_parent_dir()?;

        Ok(exec_dir.join("..\\share\\locale"))
    } else if cfg!(target_os = "macos") {
        let canonicalized_exec_dir = exec_parent_dir()?.canonicalize()?;

        if macos_is_in_app_bundle(&canonicalized_exec_dir) {
            Ok(canonicalized_exec_dir.join("../Resources/share/locale"))
        } else {
            Ok(PathBuf::from(config::LOCALEDIR))
        }
    } else {
        Ok(PathBuf::from(config::LOCALEDIR))
    }
}

pub(crate) fn cache_dir() -> anyhow::Result<PathBuf> {
    let cache_dir = match env::var("XDG_CACHE_HOME") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            let home_dir = env::home_dir().context("Unable to retrieve home dir")?;
            home_dir.join(".cache")
        }
    };
    let cache_dir = cache_dir.absolutize()?.to_path_buf();
    if !cache_dir.exists() {
        debug!(
            "Cache directory '{}' not yet present, creating..",
            cache_dir.display()
        );
        fs::create_dir(&cache_dir)?;
        debug!("Created cache directory '{}'", cache_dir.display());
    }
    Ok(cache_dir)
}

pub(crate) fn app_cache_dir() -> anyhow::Result<PathBuf> {
    let app_cache_dir = cache_dir()?.join(config::APP_NAME);
    if !app_cache_dir.exists() {
        debug!(
            "Application cache directory '{}' not yet present, creating..",
            app_cache_dir.display()
        );
        fs::create_dir(&app_cache_dir)?;
        debug!(
            "Created application cache directory '{}'",
            app_cache_dir.display()
        );
    }
    Ok(app_cache_dir)
}

pub(crate) fn drafts_dir() -> anyhow::Result<PathBuf> {
    let app_cache_dir = crate::env::app_cache_dir()?;
    let drafts_dir = app_cache_dir.join("drafts");
    if !drafts_dir.exists() {
        debug!(
            "Drafts directory '{}' not yet present, creating..",
            drafts_dir.display()
        );
        fs::create_dir(&drafts_dir)?;
        debug!("Created drafts directory '{}'", drafts_dir.display());
    }
    Ok(drafts_dir)
}

/// depending on the target platform we need to set some env vars on startup
pub(crate) fn setup_env() -> anyhow::Result<()> {
    if cfg!(target_os = "windows") {
        let data_dir = data_dir()?;
        let lib_dir = lib_dir()?;

        // SAFETY: this setup only happens while still being single-threaded
        unsafe {
            std::env::set_var("XDG_DATA_DIRS", data_dir);
            std::env::set_var(
                "GDK_PIXBUF_MODULEDIR",
                lib_dir.join("gdk-pixbuf-2.0\\2.10.0\\loaders"),
            );
        }

        // The OpenGL renderer has proven to be buggy and can even produce crashes on windows.
        // Stick to vulkan by default.
        // If it's initialization fails, Gtk4 falls back to OpenGL anyway.
        if std::env::var_os("GSK_RENDERER").is_none() {
            unsafe { std::env::set_var("GSK_RENDERER", "vulkan") };
        }

        // Without DirectComposition GSK falls back to the cairo software
        // renderer, where strokes and images render as flat coloured (pink) boxes.
        if std::env::var_os("GDK_DEBUG").is_none() {
            unsafe { std::env::set_var("GDK_DEBUG", "dcomp") };
        }
        // On Windows, pangocairo renders glyphs through cairo's win32 font backend, which goes to DirectWrite
        // and Direct2D.
        // On Windows that path is broken: pango logs "All font fallbacks failed" for every layout and rendering
        // text eventually dies with an access violation (0xc0000005) in d2d1.dll.
        // It reproducibly takes down the app when a document containing a text stroke is opened, or when the
        // typewriter is used.
        //
        // The fontconfig/freetype backend works correctly here, so select it explicitly.
        // Fontconfig is present and configured in the MSYS2 environment the app is built and shipped with.
        //
        // This deliberately does not use `std::env::set_var`: on Windows that only calls SetEnvironmentVariableW,
        // which updates the Win32 environment block but not the copy the C runtime builds at startup.
        // pangocairo reads the variable with plain `getenv` (unlike GDK, which uses `g_getenv` and therefore does
        // see `set_var`), so it would never observe the value. `_putenv_s` updates the CRT table that `getenv`
        // reads, and pangocairo resolves to the same UCRT as the app.
        if std::env::var_os("PANGOCAIRO_BACKEND").is_none() {
            unsafe extern "C" {
                fn _putenv_s(
                    name: *const std::ffi::c_char,
                    value: *const std::ffi::c_char,
                ) -> std::ffi::c_int;
            }

            unsafe { _putenv_s(c"PANGOCAIRO_BACKEND".as_ptr(), c"fc".as_ptr()) };
        }

        //unsafe { std::env::set_var("RUST_LOG", "rnote=debug,rnote-cli=debug,rnote-engine=debug,rnote-compose=debug") };
    } else if cfg!(target_os = "macos") {
        let canonicalized_exec_dir = exec_parent_dir()?.canonicalize()?;

        if macos_is_in_app_bundle(canonicalized_exec_dir) {
            let data_dir = data_dir()?;
            let resources_dir = exec_parent_dir()?.canonicalize()?.join("../Resources");

            // SAFETY: this setup only happens while still being single-threaded
            unsafe {
                std::env::set_var("XDG_DATA_DIRS", data_dir);
                std::env::set_var(
                    "GDK_PIXBUF_MODULE_FILE",
                    resources_dir.join("etc/loaders.cache"),
                );
            }
        }
    }
    Ok(())
}

fn exec_parent_dir() -> anyhow::Result<PathBuf> {
    Ok(std::env::current_exe()?
        .parent()
        .ok_or(anyhow::anyhow!(
            "could not get parent dir of executable path"
        ))?
        .to_path_buf())
}

// this returns true when the app is packaged as a relocatable application bundle
fn macos_is_in_app_bundle(canonicalized_exec_dir: impl AsRef<Path>) -> bool {
    canonicalized_exec_dir
        .as_ref()
        .components()
        .zip(canonicalized_exec_dir.as_ref().components().skip(1))
        .any(|(a, b)| {
            if let Component::Normal(a) = a
                && let Component::Normal(b) = b
            {
                a == OsStr::new("Contents") && b == OsStr::new("MacOS")
            } else {
                false
            }
        })
}
