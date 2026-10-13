use std::path::PathBuf;

use anyhow::{Context, Result};
use host::{copy_tree, HostLayout};

pub(crate) fn sync(layout: &HostLayout) -> Result<()> {
    let source = source_dir()
        .context("cannot locate the templates directory; set SHELL_TEMPLATES to its path")?;
    let destination = layout.templates_dir();
    copy_tree(&source, &destination).with_context(|| {
        format!(
            "cannot install templates from {} into {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn source_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("SHELL_TEMPLATES") {
        return Some(PathBuf::from(dir));
    }
    beside_exe().or_else(dev_fallback)
}

fn beside_exe() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.join("templates");
    dir.is_dir().then_some(dir)
}

#[cfg(debug_assertions)]
fn dev_fallback() -> Option<PathBuf> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates");
    dir.is_dir().then_some(dir)
}

#[cfg(not(debug_assertions))]
fn dev_fallback() -> Option<PathBuf> {
    None
}
