//! The plowing floor. Where files come to get railed and not come back.
//!
//! `wipe_file` takes one target, fills it with N rounds of dicks (fresh
//! variety per round), syncs to disk, and unlinks the corpse. Block devices
//! get the same treatment but skip the unlink — you don't `rm /dev/sda1`.
//! `wipe_freespace` fills the FS with dicks until ENOSPC, then deletes the
//! temp file. Both paths register with the cleanup module so a Ctrl-C
//! doesn't leave used files lying around.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use indicatif::ProgressBar;

use crate::cleanup;
use crate::dicks::{is_dick_byte, push_one, DickBuf, DickConfig};
use crate::drive;

/// Expand the user's input into a flat list of victims. Directories get
/// walked iff `--recursive`; block devices come through untouched.
pub fn parse_filelist(inputs: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for item in inputs {
        if drive::is_block_device(item) {
            out.push(item.clone());
        } else if item.is_dir() {
            if recursive {
                walk_dir(item, &mut out)?;
            } else {
                eprintln!(
                    "WARNING: {:?} is a directory and --recursive is off; skipping",
                    item
                );
            }
        } else if item.exists() {
            out.push(item.clone());
        } else {
            eprintln!("WARNING: {:?} does not exist; skipping", item);
        }
    }
    Ok(out)
}

fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("read_dir {:?}", dir))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

/// How much do we need to fill? For a regular file, the file size. For a
/// block device, `metadata().len()` lies (returns 0), so we measure via
/// `seek(End)` instead — the only way to find out how big a dick the
/// hardware can take.
fn target_size(path: &Path) -> Result<u64> {
    let mut f = OpenOptions::new()
        .read(true)
        .open(path)
        .with_context(|| format!("open(read) {:?}", path))?;
    let size = f.seek(SeekFrom::End(0))?;
    Ok(size)
}

/// Give one target the full treatment: overwrite `rounds` times, sync,
/// then unlink its corpse. `remove_file` always runs on regular files — a
/// half-pumped file in your directory is more embarrassing than no file at
/// all. Block devices keep their inode; we just bukkake the storage.
pub fn wipe_file(
    path: &Path,
    rounds: usize,
    cfg: &DickConfig,
    buf_size: usize,
    slow: bool,
    verify: bool,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let is_block = drive::is_block_device(path);
    let size = target_size(path)?;
    if let Some(pb) = progress {
        // Each round writes `size`; verify (if enabled) reads `size` per round.
        let per_round = if verify { size.saturating_mul(2) } else { size };
        pb.set_length(per_round.saturating_mul(rounds as u64));
    }

    if !is_block {
        cleanup::register(path);
    }

    let result: Result<()> = if slow {
        wipe_slow(path, rounds, cfg, size, verify, progress)
    } else {
        wipe_buffered(path, rounds, cfg, buf_size, size, verify, progress)
    };

    if !is_block {
        // Normal-exit cleanup. The Ctrl-C handler will hit this same path
        // if it fired first; remove_file is idempotent against ENOENT.
        if let Err(e) = fs::remove_file(path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("WARNING: failed to remove {:?}: {}", path, e);
            }
        }
        cleanup::unregister(path);
    }

    if let Some(pb) = progress {
        match &result {
            Ok(()) => crate::progress::celebrate(pb, path),
            Err(e) => crate::progress::lament(pb, path, &format!("{:#}", e)),
        }
    }

    result
}

fn wipe_buffered(
    path: &Path,
    rounds: usize,
    cfg: &DickConfig,
    buf_size: usize,
    size: u64,
    verify: bool,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let mut buf = DickBuf::new(cfg.clone(), buf_size);
    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .with_context(|| format!("open(write) {:?}", path))?;

    for _round in 0..rounds {
        // Refill ONCE per round. Within a round we rewrite the same dicks for
        // every chunk: the content is non-cryptographic ASCII art, so reusing
        // the buffer is invisible. Across rounds we get fresh variety.
        buf.fill();
        file.seek(SeekFrom::Start(0))?;
        let mut written: u64 = 0;
        while written < size {
            let remaining = size - written;
            let bytes = buf.as_bytes();
            let chunk = &bytes[..bytes.len().min(remaining as usize)];
            file.write_all(chunk)
                .with_context(|| format!("write_all {:?}", path))?;
            written += chunk.len() as u64;
            if let Some(pb) = progress {
                pb.inc(chunk.len() as u64);
            }
        }
        file.sync_data().ok();
        if verify {
            verify_file(path, size, progress)?;
        }
    }
    Ok(())
}

fn wipe_slow(
    path: &Path,
    rounds: usize,
    cfg: &DickConfig,
    size: u64,
    verify: bool,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let mut rng = crate::dicks::new_rng();
    let mut scratch: Vec<u8> = Vec::with_capacity(cfg.max_dick_len());
    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .with_context(|| format!("open(write) {:?}", path))?;

    for _round in 0..rounds {
        file.seek(SeekFrom::Start(0))?;
        let mut written: u64 = 0;
        while written < size {
            scratch.clear();
            push_one(&mut rng, cfg, &mut scratch);
            let remaining = size - written;
            let chunk = &scratch[..scratch.len().min(remaining as usize)];
            file.write_all(chunk)
                .with_context(|| format!("write_all {:?}", path))?;
            written += chunk.len() as u64;
            if let Some(pb) = progress {
                pb.inc(chunk.len() as u64);
            }
        }
        file.sync_data().ok();
        if verify {
            verify_file(path, size, progress)?;
        }
    }
    Ok(())
}

/// Read the freshly-wiped file back and make sure every single byte is in
/// the canonical penis alphabet. If something other than a dick shows up,
/// the drive cheated on us (bad sector, FS shenanigans, mid-pump interrupt).
fn verify_file(path: &Path, size: u64, progress: Option<&ProgressBar>) -> Result<()> {
    let mut file = File::open(path).with_context(|| format!("open(read) {:?}", path))?;
    let mut buf = vec![0u8; 64 * 1024];
    let mut offset: u64 = 0;
    while offset < size {
        let want = ((size - offset) as usize).min(buf.len());
        let n = file
            .read(&mut buf[..want])
            .with_context(|| format!("read {:?}", path))?;
        if n == 0 {
            return Err(anyhow!(
                "verify: file shorter than expected at offset {}",
                offset
            ));
        }
        if let Some(bad) = buf[..n].iter().position(|b| !is_dick_byte(*b)) {
            return Err(anyhow!(
                "verify: non-dick byte 0x{:02x} at offset {}",
                buf[bad],
                offset + bad as u64
            ));
        }
        offset += n as u64;
        if let Some(pb) = progress {
            pb.inc(n as u64);
        }
    }
    Ok(())
}

/// Cum-soak the filesystem's free space. Creates one big temp file under
/// the given mount, writes dicks until the FS taps out (ENOSPC), then
/// deletes the temp file so the disk's "free" again — but every previously
/// dead-but-not-overwritten block now holds a dick instead of your old data.
pub fn wipe_freespace(
    mount_hint: &Path,
    cfg: &DickConfig,
    buf_size: usize,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let dir = if mount_hint.is_dir() {
        mount_hint.to_path_buf()
    } else {
        mount_hint
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };
    let temp = dir.join("wipedicks-freespace.tmp");
    cleanup::register(&temp);

    let mut buf = DickBuf::new(cfg.clone(), buf_size);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp)
        .with_context(|| format!("create {:?}", temp))?;

    // Fill once; the freespace fill is a single pass — reuse the same buffer
    // for every chunk.
    buf.fill();
    let result = freespace_loop(&mut file, &buf, progress);

    drop(file);
    if let Err(e) = fs::remove_file(&temp) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!("WARNING: failed to remove {:?}: {}", temp, e);
        }
    }
    cleanup::unregister(&temp);
    if let Some(pb) = progress {
        match &result {
            Ok(()) => crate::progress::celebrate(pb, std::path::Path::new("free space")),
            Err(e) => {
                crate::progress::lament(pb, std::path::Path::new("free space"), &format!("{:#}", e))
            }
        }
    }
    result
}

fn freespace_loop(
    file: &mut std::fs::File,
    buf: &DickBuf,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    loop {
        match file.write_all(buf.as_bytes()) {
            Ok(()) => {
                if let Some(pb) = progress {
                    pb.inc(buf.as_bytes().len() as u64);
                }
            }
            // ENOSPC on Unix / ERROR_DISK_FULL on Windows — std maps both to
            // StorageFull, so we sidestep platform-specific errno crates.
            Err(e) if e.kind() == std::io::ErrorKind::StorageFull => break,
            Err(e) => return Err(e).context("freespace write"),
        }
    }
    file.sync_data().ok();
    Ok(())
}
