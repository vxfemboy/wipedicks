//! Tells you whether the thing you're about to plow is a spinning slut
//! (HDD — go nuts) or a delicate flash queen (SSD/NVMe — easy, tiger).
//!
//! Multi-pass overwrites on flash are a double L: ineffective because the
//! FTL remaps your writes to fresh NAND (the original data sits smug in
//! over-provisioned cells), and harmful because every pass eats program/erase
//! cycles off the drive's lifespan. So we refuse flash by default and point
//! you at the drive's native secure-erase, which actually gives the drive a
//! happy ending instead of premature death.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};

/// What flavor of storage we're dealing with. Determines whether it can take
/// a multi-pass railing or whether we should treat it like fine china.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Only Linux's sysfs path distinguishes all four variants. macOS lumps every
// flash device into `Sata` (so `Nvme` is dead there) and Windows always
// returns `Unknown` (so Nvme/Sata/Rotational are dead). Keep the lint useful
// where it actually catches things — i.e. Linux — and silence it elsewhere.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub enum DriveKind {
    Nvme,
    Sata,       // SSD on SATA bus, rotational==0
    Rotational, // Spinning rust, fair game for overwriting
    Unknown,    // Probably non-Linux or unmounted; treat as safe-to-overwrite
}

impl DriveKind {
    pub fn is_flash(self) -> bool {
        matches!(self, DriveKind::Nvme | DriveKind::Sata)
    }
}

/// Whether the path itself is a block device file (e.g. /dev/sda).
pub fn is_block_device(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        std::fs::metadata(path)
            .map(|m| m.file_type().is_block_device())
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        false
    }
}

/// For a path on a mounted filesystem, find the underlying block device.
/// For a path that already IS a block device, return it as-is.
///
/// Resolves via `/proc/self/mountinfo` rather than `st_dev`, because some
/// filesystems (notably btrfs subvolumes, bind mounts, overlay) use anonymous
/// block devices with major=0 that don't exist under `/sys/dev/block/`.
#[cfg(target_os = "linux")]
pub fn resolve_block_device(path: &Path) -> Option<PathBuf> {
    if is_block_device(path) {
        return Some(path.to_path_buf());
    }
    let canonical = std::fs::canonicalize(path).ok()?;
    let mountinfo = std::fs::read_to_string("/proc/self/mountinfo").ok()?;

    let mut best: Option<(usize, PathBuf)> = None;
    for line in mountinfo.lines() {
        // Format: id parent maj:min root mountpoint opts ... - fstype source super_opts
        let mut parts = line.split_whitespace();
        let _id = parts.next();
        let _parent = parts.next();
        let _devno = parts.next();
        let _root = parts.next();
        let mountpoint = match parts.next() {
            Some(m) => m,
            None => continue,
        };
        // Skip optional fields up to the " - " separator
        let after_dash = match line.split(" - ").nth(1) {
            Some(s) => s,
            None => continue,
        };
        let mut fields = after_dash.split_whitespace();
        let _fstype = fields.next();
        let source = match fields.next() {
            Some(s) => s,
            None => continue,
        };

        let mp = PathBuf::from(decode_mountinfo(mountpoint));
        if !canonical.starts_with(&mp) {
            continue;
        }
        let len = mp.as_os_str().len();
        if best.as_ref().is_none_or(|(l, _)| len > *l) {
            best = Some((len, PathBuf::from(decode_mountinfo(source))));
        }
    }
    let (_, src) = best?;
    // Pseudo filesystems give sources like "tmpfs", "proc" with no leading slash.
    if !src.is_absolute() {
        return None;
    }
    std::fs::canonicalize(&src).ok().or(Some(src))
}

/// mountinfo encodes spaces/tabs as octal escapes (\040, \011). Decode them.
#[cfg(target_os = "linux")]
fn decode_mountinfo(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 3 < bytes.len() {
            if let Ok(n) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 4]).unwrap_or("000"),
                8,
            ) {
                out.push(n as char);
                i += 4;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// macOS resolution: walk `df -P` output to find the device for a path.
#[cfg(target_os = "macos")]
pub fn resolve_block_device(path: &Path) -> Option<PathBuf> {
    if is_block_device(path) {
        return Some(path.to_path_buf());
    }
    let out = Command::new("/bin/df").arg("-P").arg(path).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = std::str::from_utf8(&out.stdout).ok()?;
    // Skip header; take first column of second line.
    let line = stdout.lines().nth(1)?;
    let dev = line.split_whitespace().next()?;
    Some(PathBuf::from(dev))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn resolve_block_device(path: &Path) -> Option<PathBuf> {
    if is_block_device(path) {
        Some(path.to_path_buf())
    } else {
        // TODO: Windows storage classification.
        None
    }
}

/// Classify a block device (e.g. /dev/nvme0n1, /dev/sda, /dev/sda1).
#[cfg(target_os = "linux")]
pub fn classify(dev_path: &Path) -> DriveKind {
    let name = match dev_path.file_name().and_then(|s| s.to_str()) {
        Some(n) => n,
        None => return DriveKind::Unknown,
    };
    if name.starts_with("nvme") {
        return DriveKind::Nvme;
    }
    // For partitions like sda1, sysfs queue/ lives on the parent (sda).
    let direct = PathBuf::from("/sys/class/block").join(name);
    let queue_dir = if direct.join("queue/rotational").exists() {
        direct
    } else if let Ok(real) = std::fs::canonicalize(&direct) {
        match real.parent() {
            Some(p) if p.join("queue/rotational").exists() => p.to_path_buf(),
            _ => return DriveKind::Unknown,
        }
    } else {
        return DriveKind::Unknown;
    };
    match std::fs::read_to_string(queue_dir.join("queue/rotational"))
        .ok()
        .map(|s| s.trim().to_string())
        .as_deref()
    {
        Some("0") => DriveKind::Sata,
        Some("1") => DriveKind::Rotational,
        _ => DriveKind::Unknown,
    }
}

#[cfg(target_os = "macos")]
pub fn classify(dev_path: &Path) -> DriveKind {
    // diskutil info -plist <dev> emits XML containing <key>SolidState</key>
    // followed by <true/> or <false/>. Heuristic — doesn't distinguish NVMe
    // from SATA — but the user-facing advice (use the OS-native secure-erase)
    // is universally correct for both, so we classify all flash as `Sata`.
    let out = match Command::new("/usr/sbin/diskutil")
        .arg("info")
        .arg("-plist")
        .arg(dev_path)
        .output()
    {
        Ok(o) if o.status.success() => o.stdout,
        _ => return DriveKind::Unknown,
    };
    let s = match std::str::from_utf8(&out) {
        Ok(s) => s,
        Err(_) => return DriveKind::Unknown,
    };
    if let Some(pos) = s.find("<key>SolidState</key>") {
        let tail = &s[pos..];
        if tail.contains("<true/>")
            && tail.find("<true/>") < tail.find("<key>").or(Some(usize::MAX))
        {
            return DriveKind::Sata;
        }
        if tail.contains("<false/>") {
            return DriveKind::Rotational;
        }
    }
    DriveKind::Unknown
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn classify(_dev_path: &Path) -> DriveKind {
    // TODO: Windows storage classification via Get-PhysicalDisk / IOCTL_STORAGE_QUERY_PROPERTY.
    DriveKind::Unknown
}

/// The bouncer for block-device targets. Returns `Ok(true)` to let the
/// caller plow ahead, `Ok(false)` if we already handled it via secure-erase
/// (drive's own factory reset — clean and consensual), `Err` to slam the
/// brakes because something's flash and the user didn't say --rape.
pub fn handle_block_device(dev: &Path, rape: bool, secure_erase: bool) -> Result<bool> {
    let kind = classify(dev);

    if secure_erase {
        run_secure_erase(dev, kind)?;
        return Ok(false);
    }

    if kind.is_flash() && !rape {
        print_refusal(dev, kind);
        return Err(anyhow!(
            "refusing to overwrite flash device {:?} (use --rape to override, or --secure-erase)",
            dev
        ));
    }

    Ok(true)
}

/// Heads-up for the user when their wipe target lives on flash. File-level
/// overwrite on an SSD is theatrical edging — feels good, accomplishes
/// nothing at the NAND level. We still delete the inode (that part works).
pub fn warn_file_on_flash(path: &Path, dev: &Path) {
    eprintln!("WARNING: {:?} sits on flash device {:?}.", path, dev);
    eprintln!("         File-level overwrite is theatrical on SSD/NVMe: the FTL won't");
    eprintln!("         necessarily clobber the physical NAND page that held this file.");
    eprintln!("         The file will still be deleted, but treat the wipe as advisory.");
}

fn print_refusal(dev: &Path, kind: DriveKind) {
    eprintln!();
    eprintln!("8==X  Refusing to overwrite {:?}", dev);
    eprintln!();
    eprintln!(
        "This is flash storage ({:?}). Multi-pass overwrite is:",
        kind
    );
    eprintln!(
        "  - INEFFECTIVE: the FTL remaps writes, leaving stale copies in over-provisioned NAND"
    );
    eprintln!("  - HARMFUL:     each pass consumes finite program/erase cycles");
    eprintln!();
    eprintln!("Use the drive's native secure-erase instead:");
    match kind {
        DriveKind::Nvme => {
            eprintln!(
                "    sudo nvme format {} -s 1        # user-data erase",
                dev.display()
            );
            eprintln!(
                "    sudo nvme format {} -s 2        # cryptographic erase (SED only)",
                dev.display()
            );
            eprintln!(
                "    sudo nvme sanitize {} --sanact=2 # block erase (if supported)",
                dev.display()
            );
        }
        DriveKind::Sata => {
            eprintln!(
                "    sudo hdparm --user-master u --security-set-pass p {}",
                dev.display()
            );
            eprintln!(
                "    sudo hdparm --user-master u --security-erase p {}",
                dev.display()
            );
            eprintln!();
            eprintln!("Or, for a fast non-cryptographic discard:");
            eprintln!("    sudo blkdiscard {}", dev.display());
        }
        _ => {}
    }
    eprintln!();
    eprintln!("For self-encrypting drives (TCG Opal), use `sedutil-cli --revertNoErase`.");
    eprintln!();
    eprintln!("To run those for you: pass --secure-erase.");
    eprintln!("To bypass this check anyway: pass --rape.");
    eprintln!();
}

fn run_secure_erase(dev: &Path, kind: DriveKind) -> Result<()> {
    match kind {
        DriveKind::Nvme => nvme_format(dev),
        DriveKind::Sata => hdparm_secure_erase(dev),
        DriveKind::Rotational => {
            eprintln!(
                "NOTE: {:?} is rotational; native secure-erase still applies via hdparm.",
                dev
            );
            hdparm_secure_erase(dev)
        }
        DriveKind::Unknown => Err(anyhow!(
            "cannot classify {:?} as NVMe or SATA; refuse to guess",
            dev
        )),
    }
}

fn nvme_format(dev: &Path) -> Result<()> {
    eprintln!("Running: nvme format {} -s 1", dev.display());
    let status = Command::new("nvme")
        .arg("format")
        .arg(dev)
        .arg("-s")
        .arg("1")
        .status()
        .context("failed to spawn `nvme` (install nvme-cli)")?;
    if !status.success() {
        return Err(anyhow!("nvme format failed: {}", status));
    }
    Ok(())
}

fn hdparm_secure_erase(dev: &Path) -> Result<()> {
    // Use a dummy password; the drive doesn't care, it just needs to be set
    // to unlock the SECURITY ERASE UNIT command.
    let pass = "p";
    eprintln!(
        "Running: hdparm --user-master u --security-set-pass {} {}",
        pass,
        dev.display()
    );
    let s1 = Command::new("hdparm")
        .args(["--user-master", "u", "--security-set-pass"])
        .arg(pass)
        .arg(dev)
        .status()
        .context("failed to spawn `hdparm`")?;
    if !s1.success() {
        return Err(anyhow!("hdparm --security-set-pass failed: {}", s1));
    }
    eprintln!(
        "Running: hdparm --user-master u --security-erase {} {}",
        pass,
        dev.display()
    );
    let s2 = Command::new("hdparm")
        .args(["--user-master", "u", "--security-erase"])
        .arg(pass)
        .arg(dev)
        .status()
        .context("failed to spawn `hdparm`")?;
    if !s2.success() {
        return Err(anyhow!("hdparm --security-erase failed: {}", s2));
    }
    Ok(())
}
