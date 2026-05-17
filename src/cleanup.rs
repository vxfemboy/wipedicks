//! Pull-out registry for graceful interrupts.
//!
//! Every wipe target and every freespace temp file checks in here before it
//! gets opened. If the user hits Ctrl-C / something kills us with SIGTERM,
//! the handler atomically drains the registry and `unlink(2)`s every entry
//! — no used files left dangling like a guilty walk of shame.
//!
//! The normal happy-ending path also unlinks files directly. Both paths are
//! idempotent (already-gone == fine), so they cooperate without coordination.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use anyhow::Result;

static REGISTRY: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();

fn registry() -> &'static Mutex<Vec<PathBuf>> {
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn install_handler() -> Result<()> {
    let _ = registry();
    ctrlc::set_handler(|| {
        eprintln!("\n8==X  interrupted; cleaning up...");
        let paths = std::mem::take(&mut *registry().lock().unwrap());
        for path in paths {
            let _ = std::fs::remove_file(&path);
        }
        std::process::exit(130); // 128 + SIGINT
    })?;
    Ok(())
}

pub fn register(path: &Path) {
    registry().lock().unwrap().push(path.to_path_buf());
}

pub fn unregister(path: &Path) {
    let mut q = registry().lock().unwrap();
    if let Some(idx) = q.iter().position(|p| p == path) {
        q.swap_remove(idx);
    }
}
