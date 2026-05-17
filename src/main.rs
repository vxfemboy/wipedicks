//! WipeDicks: the only file shredder that finishes with a happy ending.
//!
//! Pointy end of the codebase. Parses args, runs the safety + drive policy
//! gauntlet, then unleashes a bounded thread pool of horny workers on the
//! file list. Each worker takes a target, opens it, fills it with dicks,
//! removes its corpse, and reports back. Ctrl-C drains the in-flight cleanup
//! registry so we don't leave temp files lying around like used condoms.

mod cleanup;
mod cli;
mod dicks;
mod drive;
mod progress;
mod safety;
mod wipe;

use std::collections::HashSet;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

use anyhow::Result;

use crate::cli::Args;
use crate::dicks::{push_one, DickConfig};
use crate::progress::Progress;

fn main() {
    let args = cli::parse();
    if let Err(msg) = args.validate() {
        eprintln!("error: {msg}");
        std::process::exit(2);
    }
    let cfg = DickConfig {
        shaft_min: args.shaft_min,
        shaft_max: args.shaft_max,
        jizz_min: args.jizz_min,
        jizz_max: args.jizz_max,
        balls_chance: args.balls_chance,
    };

    if args.dry_run {
        run_dry(&cfg, args.buffer_size);
        return;
    }

    if let Err(e) = cleanup::install_handler() {
        eprintln!("warning: failed to install signal handler: {e:#}");
    }

    match run(args, cfg) {
        Ok(0) => {}
        Ok(n) => {
            eprintln!("{n} target(s) failed");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }
}

/// Squirt one buffer of generated dicks to stdout, then bail. For previewing
/// your --shaft-* / --jizz-* / --balls-chance settings without actually
/// ruining anyone's day. Visual edging only — no penetration.
fn run_dry(cfg: &DickConfig, buf_size: usize) {
    let mut rng = dicks::new_rng();
    let mut out = Vec::with_capacity(buf_size);
    while out.len() < buf_size {
        push_one(&mut rng, cfg, &mut out);
    }
    out.truncate(buf_size);
    let _ = std::io::stdout().write_all(&out);
}

/// The main fuckening. Returns the number of files that failed to take a
/// proper dicking (caller sets the exit code based on that).
fn run(args: Args, cfg: DickConfig) -> Result<u32> {
    // Cock-block check on the raw inputs. Catches `wipedicks /` or
    // `wipedicks -r /etc` BEFORE we walk a huge tree and refuse one file at a
    // time. Nobody wants to find out their hand slipped on /usr/.
    if !args.rape {
        for f in &args.files {
            let is_block = drive::is_block_device(f);
            if let Err(hazard) = safety::check_path(f, is_block) {
                safety::explain(f, hazard);
                return Ok(1);
            }
        }
    }

    let mut files = wipe::parse_filelist(&args.files, args.recursive)?;
    if files.is_empty() && !args.wipefree {
        eprintln!("nothing to wipe");
        return Ok(0);
    }

    // Path safety: refuse system paths, swap, mounted block devices.
    // Drive safety: refuse / delegate for block-device targets up front,
    // warn once per unique flash device for regular files.
    let mut warned_devices: HashSet<PathBuf> = HashSet::new();
    let mut keep: Vec<PathBuf> = Vec::with_capacity(files.len());
    for f in files.drain(..) {
        let is_block = drive::is_block_device(&f);
        if !args.rape {
            if let Err(hazard) = safety::check_path(&f, is_block) {
                safety::explain(&f, hazard);
                return Ok(1);
            }
        }
        if is_block {
            match drive::handle_block_device(&f, args.rape, args.secure_erase) {
                Ok(true) => keep.push(f),
                Ok(false) => { /* secure_erase already ran; skip overwrite */ }
                Err(e) => {
                    eprintln!("{e:#}");
                    return Ok(1);
                }
            }
        } else {
            if let Some(dev) = drive::resolve_block_device(&f) {
                if drive::classify(&dev).is_flash()
                    && warned_devices.insert(dev.clone())
                    && !args.rape
                {
                    drive::warn_file_on_flash(&f, &dev);
                }
            }
            keep.push(f);
        }
    }
    let files = keep;

    let progress_enabled = !args.no_progress && std::io::stderr().is_terminal();
    let progress = Progress::new(progress_enabled, args.bar_width, args.gay);

    let workers = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(files.len().max(1));

    let queue: Mutex<Vec<PathBuf>> = Mutex::new(files);
    let failures = std::sync::atomic::AtomicU32::new(0);

    thread::scope(|s| {
        for _ in 0..workers {
            let queue = &queue;
            let cfg = &cfg;
            let progress = &progress;
            let failures = &failures;
            let args = &args;
            s.spawn(move || loop {
                let path = match queue.lock().unwrap().pop() {
                    Some(p) => p,
                    None => break,
                };
                let total = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let pb = progress.add_file(&path, total);
                let res = wipe::wipe_file(
                    &path,
                    args.rounds,
                    cfg,
                    args.buffer_size,
                    args.slow,
                    args.verify,
                    pb.as_ref(),
                );
                if let Err(e) = res {
                    eprintln!("ERROR wiping {:?}: {:#}", path, e);
                    failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            });
        }
    });

    if args.wipefree {
        // Use the first input as a mount hint; fall back to CWD.
        let hint = args.files.first().cloned().unwrap_or(PathBuf::from("."));
        let pb = progress.add_file(&PathBuf::from("free space"), 0);
        if let Err(e) = wipe::wipe_freespace(&hint, &cfg, args.buffer_size, pb.as_ref()) {
            eprintln!("ERROR wiping free space: {e:#}");
            failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    Ok(failures.load(std::sync::atomic::Ordering::Relaxed))
}
