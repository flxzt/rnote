// Imports
use crate::config;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

pub(crate) fn lib_dir() -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let exec_dir = exec_parent_dir()?;

        Ok(exec_dir.join("..\\lib"))
    } else if cfg!(target_os = "macos") {
        let canonicalized_exec_dir = exec_parent_dir()?.canonicalize()?;

        if macos_is_in_app_bundle(&canonicalized_exec_dir) {
            Ok(canonicalized_exec_dir.join("../Resources/lib"))
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

            //std::env::set_var("RUST_LOG", "rnote=debug,rnote-cli=debug,rnote-engine=debug,rnote-compose=debug");
        }
    } else if cfg!(target_os = "macos") {
        let canonicalized_exec_dir = exec_parent_dir()?.canonicalize()?;

        if macos_is_in_app_bundle(canonicalized_exec_dir) {
            let data_dir = data_dir()?;
            let lib_dir = lib_dir()?;

            // SAFETY: this setup only happens while still being single-threaded
            unsafe {
                std::env::set_var("XDG_DATA_DIRS", data_dir);
                std::env::set_var(
                    "GDK_PIXBUF_MODULE_FILE",
                    lib_dir.join("gdk-pixbuf-2.0/2.10.0/loaders.cache"),
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
            if let (Component::Normal(a), Component::Normal(b)) = (a, b) {
                a == OsStr::new("Contents") && b == OsStr::new("MacOS")
            } else {
                false
            }
        })
}

#[cfg(target_os = "windows")]
/// Workaround for windows for shadow that intercept mouse events outside of the
/// actual window. See https://github.com/flxzt/rnote/issues/1372
///
/// Taken from gaphor
/// See comment from https://github.com/gaphor/gaphor/blob/a7b35712b166a38b78933a79613eab330f7bd885/gaphor/ui/styling-windows.css
/// and https://gitlab.gnome.org/GNOME/gtk/-/issues/6255#note_1952796
pub fn window_styling_workaround() -> anyhow::Result<()> {
    use gtk4::{gdk, style_context_add_provider_for_display};

    // gtk needs to be initialized for the style provider to work
    gtk4::init()?;

    let default_display = gdk::Display::default();
    let style_provider = gtk4::CssProvider::new();
    style_provider.load_from_string(
        "
            .csd {
          box-shadow: 0 3px 9px 1px alpha(black, 0.35),
                      0 0 0 1px alpha(black, 0.18);
        }
        
        .csd:backdrop {
          box-shadow: 0 3px 9px 1px transparent,
                      0 2px 6px 2px alpha(black, 1),
                      0 0 0 1px alpha(black, 0.06);
        }",
    );

    match default_display {
        Some(display) => {
            style_context_add_provider_for_display(
                &display,
                &style_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            Ok(())
        }
        None => Err(anyhow::anyhow!("Could not find a default display")),
    }
}
