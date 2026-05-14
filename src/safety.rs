//! Cock-block department. Stops you from face-fucking your /etc/ when your
//! hand slipped, your /boot/ when you meant /tmp/, or the swap file your
//! kernel is currently using to remember things.
//!
//! Refuses by default; the --rape safe-word bypasses every check. We don't
//! ass-blast your filesystem just because you fat-fingered a path.

use std::fmt;
use std::path::Path;
// PathBuf is only referenced by name in the Linux mountinfo/swaps parsers.
#[cfg(target_os = "linux")]
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// On non-Linux platforms `is_swap` and `is_mounted_block_device` are no-op
// fallbacks (no /proc/swaps, no /proc/self/mountinfo), so SwapFile and
// MountedDevice are never constructed there. Keep the lint useful on Linux.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub enum Hazard {
    /// Path is, or is inside, a system directory we shouldn't touch.
    SystemPath,
    /// Path is currently registered as a swap file/device.
    SwapFile,
    /// Path is a block device that's currently mounted somewhere.
    MountedDevice,
}

impl fmt::Display for Hazard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hazard::SystemPath => write!(f, "system path (refusing to wipe OS files)"),
            Hazard::SwapFile => write!(f, "active swap (listed in /proc/swaps)"),
            Hazard::MountedDevice => write!(f, "block device currently mounted"),
        }
    }
}

/// Hard-coded prefixes wipedicks refuses by default. The current working
/// directory and `/home` are deliberately NOT on this list — users routinely
/// wipe their own data there.
const SYSTEM_PREFIXES: &[&str] = &[
    "/boot", "/etc", "/usr", "/var", "/lib", "/lib64", "/lib32", "/bin", "/sbin", "/sys", "/proc",
    "/dev", "/root", "/srv", "/opt",
];

pub fn check_path(path: &Path, is_block: bool) -> Result<(), Hazard> {
    if is_dangerous_prefix(path) {
        return Err(Hazard::SystemPath);
    }
    if is_swap(path) {
        return Err(Hazard::SwapFile);
    }
    if is_block && is_mounted_block_device(path) {
        return Err(Hazard::MountedDevice);
    }
    Ok(())
}

fn is_dangerous_prefix(path: &Path) -> bool {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    // The root itself.
    if canonical == Path::new("/") {
        return true;
    }
    for prefix in SYSTEM_PREFIXES {
        let p = Path::new(prefix);
        if canonical == p || canonical.starts_with(p) {
            return true;
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn is_swap(path: &Path) -> bool {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let swaps = match std::fs::read_to_string("/proc/swaps") {
        Ok(s) => s,
        Err(_) => return false,
    };
    swaps
        .lines()
        .skip(1) // header
        .filter_map(|line| line.split_whitespace().next())
        .map(PathBuf::from)
        .any(|p| p == canonical)
}

#[cfg(not(target_os = "linux"))]
fn is_swap(_path: &Path) -> bool {
    false
}

#[cfg(target_os = "linux")]
fn is_mounted_block_device(dev: &Path) -> bool {
    let canonical = std::fs::canonicalize(dev).unwrap_or_else(|_| dev.to_path_buf());
    let mountinfo = match std::fs::read_to_string("/proc/self/mountinfo") {
        Ok(s) => s,
        Err(_) => return false,
    };
    for line in mountinfo.lines() {
        let after_dash = match line.split(" - ").nth(1) {
            Some(s) => s,
            None => continue,
        };
        let mut fields = after_dash.split_whitespace();
        let _fstype = fields.next();
        if let Some(src) = fields.next() {
            let src_path = PathBuf::from(src);
            if src_path == canonical {
                return true;
            }
            if let Ok(resolved) = std::fs::canonicalize(&src_path) {
                if resolved == canonical {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(not(target_os = "linux"))]
fn is_mounted_block_device(_dev: &Path) -> bool {
    // Without /proc/self/mountinfo we don't have a portable way to check.
    // Err on the side of letting the user proceed; --secure-erase delegation
    // and the OS-level "device busy" error provide a second line of defense.
    false
}

pub fn explain(path: &Path, hazard: Hazard) {
    eprintln!();
    eprintln!("8==X  Refusing to wipe {:?}", path);
    eprintln!("      Reason: {}", hazard);
    eprintln!();
    match hazard {
        Hazard::SystemPath => {
            eprintln!("      Wiping OS files breaks your machine. If you really mean to,");
            eprintln!("      pass --rape.");
        }
        Hazard::SwapFile => {
            eprintln!("      Disable swap first:  sudo swapoff {}", path.display());
            eprintln!("      Then re-run, or pass --rape to wipe anyway.");
        }
        Hazard::MountedDevice => {
            eprintln!("      Unmount first:  sudo umount {}", path.display());
            eprintln!("      Then re-run, or pass --rape to wipe anyway.");
        }
    }
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_is_refused() {
        assert!(is_dangerous_prefix(Path::new("/")));
    }

    #[test]
    fn etc_passwd_is_refused() {
        assert!(is_dangerous_prefix(Path::new("/etc/passwd")));
    }

    #[test]
    fn etc_lookalike_is_allowed() {
        // /home/etc-backup is not /etc.
        assert!(!is_dangerous_prefix(Path::new("/home/etc-backup")));
    }

    #[test]
    fn tmp_is_allowed() {
        assert!(!is_dangerous_prefix(Path::new("/tmp/anything")));
    }
}
