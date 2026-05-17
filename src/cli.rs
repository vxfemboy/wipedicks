//! CLI surface. Every doc comment in here becomes part of `--help`, so we treat
//! help text as marketing copy: technically correct, narratively unhinged.

use std::path::PathBuf;

use clap::Parser;

/// Wipe files and devices by face-fucking them with ASCII penises.
///
/// By default, refuses to plow SSDs and NVMes — flash storage doesn't enjoy
/// repeat performances. The Flash Translation Layer remaps your writes to fresh
/// NAND, so your "secure overwrite" leaves the original data sitting smug in
/// over-provisioned cells, and every pass burns program/erase cycles off the
/// drive's lifespan. Use --secure-erase to delegate to the OS's native tool
/// (the drive's own factory reset), or --rape to override and proceed anyway,
/// you absolute degenerate.
#[derive(Parser, Debug)]
#[command(
    name = "wipedicks",
    version,
    about = "Wipe files/devices with dicks.",
    long_about = None,
)]
pub struct Args {
    /// Files, directories, or block devices to wipe. Pass as many as you like —
    /// we'll service them all in parallel like the bukkake bottom we are.
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,

    /// Go deep — recurse into directories and shred every file inside.
    #[arg(short = 'r', long)]
    pub recursive: bool,

    /// How many rounds of overwriting per target. 1 by default (basic
    /// missionary). Bump it for ceremonial multi-pass plowing — note: extra
    /// rounds on SSDs are pure self-harm; see --rape.
    #[arg(short = 'n', long = "numrounds", default_value_t = 1, value_name = "N")]
    pub rounds: usize,

    /// After the targets are gone, fill the filesystem's free space with dicks
    /// until ENOSPC, then delete the temp file. Erases the ghosts of files past.
    #[arg(short = 'w', long)]
    pub wipefree: bool,

    /// Take it slow, big boy. One penis per write(2). Useful for sanity
    /// comparison or terminals with a kink for tiny syscalls. ~5000x more
    /// syscalls than the buffered path.
    #[arg(short = 's', long)]
    pub slow: bool,

    /// Override every safety check (system paths, swap, mounted devices, the
    /// SSD refusal). The nuclear safe word — if you pull this, we trust you
    /// know exactly which hole you're aiming at. There is no afterward.
    #[arg(long)]
    pub rape: bool,

    /// On a block-device target, delegate to the drive's native secure-erase
    /// tool: `nvme format -s 1` for NVMe, `hdparm --security-erase` for SATA.
    /// Actually secure, doesn't burn PE cycles, gives the drive a happy ending.
    #[arg(long)]
    pub secure_erase: bool,

    /// Minimum '=' (shaft) count per penis. Default 1 — micro-dick energy.
    #[arg(long = "shaft-min", default_value_t = 1, value_name = "N")]
    pub shaft_min: usize,

    /// Maximum '=' (shaft) count per penis. Default 12 — well-endowed but
    /// believable. Crank it for absolute schlong content.
    #[arg(long = "shaft-max", default_value_t = 12, value_name = "N")]
    pub shaft_max: usize,

    /// Minimum '~' (jizz) drip count per penis. 0 is "pulled out in time".
    #[arg(long = "jizz-min", default_value_t = 0, value_name = "N")]
    pub jizz_min: usize,

    /// Maximum '~' (jizz) drip count per penis. 3 is the default — modest pearl
    /// necklace. Raise it for serious cumshot energy.
    #[arg(long = "jizz-max", default_value_t = 3, value_name = "N")]
    pub jizz_max: usize,

    /// Probability that a given penis gets a '#' (balls) at the base, in
    /// [0.0, 1.0]. 0.5 is the default — half your dicks are circumcised.
    #[arg(long = "balls-chance", default_value_t = 0.5, value_name = "P")]
    pub balls_chance: f32,

    /// Write batch size in bytes. Larger buffer = fewer syscalls = faster
    /// stroke rate. Default 1 MiB. Don't go below 64.
    #[arg(long = "buffer-size", default_value_t = 1 << 20, value_name = "BYTES")]
    pub buffer_size: usize,

    /// Width of each progress bar in characters. Default 16. Make it bigger
    /// if you like long bars.
    #[arg(long = "bar-width", default_value_t = 16, value_name = "N")]
    pub bar_width: usize,

    /// Disable progress bars entirely. Auto-disabled when stderr isn't a TTY
    /// (piped output stays clean — no escaped dicks bleeding into your logs).
    #[arg(long = "no-progress")]
    pub no_progress: bool,

    /// Print one buffer of generated dicks to stdout and exit, without
    /// destroying anything. Use to preview your --shaft-* / --jizz-* /
    /// --balls-chance configuration. Visual masturbation only.
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// After every round, read the file back and confirm every byte is a
    /// valid penis character (8, =, #, D, ~, space). Catches bad sectors and
    /// filesystem corruption. For the paranoid bottom.
    #[arg(long)]
    pub verify: bool,

    /// Cycle the progress bar through the rainbow. Pride parade for the
    /// shaft; jizz stays bright white (cum is cum regardless of orientation).
    #[arg(long)]
    pub gay: bool,
}

impl Args {
    /// Sanity-check cross-flag combinations. Returns the user-facing reason
    /// the args don't make sense, if any. Good safe-words save lives.
    pub fn validate(&self) -> Result<(), String> {
        if self.shaft_min > self.shaft_max {
            return Err("--shaft-min must be <= --shaft-max (your max can't be smaller than your min, big guy)".into());
        }
        if self.jizz_min > self.jizz_max {
            return Err("--jizz-min must be <= --jizz-max (drip range got crossed)".into());
        }
        if !(0.0..=1.0).contains(&self.balls_chance) {
            return Err(
                "--balls-chance must be in [0.0, 1.0] (it's a probability, not a body count)"
                    .into(),
            );
        }
        if self.buffer_size < 64 {
            return Err(
                "--buffer-size must be >= 64 bytes (too small to fit a single dick)".into(),
            );
        }
        if self.bar_width < 4 {
            return Err("--bar-width must be >= 4 (need room for [8D] at minimum)".into());
        }
        Ok(())
    }
}

pub fn parse() -> Args {
    Args::parse()
}
