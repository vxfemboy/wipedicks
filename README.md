# 8====D~ WipeDicks

[![CI](https://github.com/vxfemboy/wipedicks/actions/workflows/ci.yml/badge.svg)](https://github.com/vxfemboy/wipedicks/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/wipedicks.svg)](https://crates.io/crates/wipedicks)

*The only file shredder that finishes with a happy ending.*

```
⠀⠀⠀⠀⠀⣠⠶⠚⠛⠛⠛⠲⢦⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⣴⠟⠁⠀⠀⠀⠀⠀⠀⠀⠻⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⣠⣾⣷⣄⠀⠀⠀⢀⣠⣤⣤⡀⠀⢿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⢸⣿⡿⢃⣸⡶⠂⢠⣿⣿⡿⠁⣱⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⢸⡏⠉⠩⣏⣐⣦⠀⠛⠦⠴⠚⠁⠀⣸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⣼⠧⠶⠶⠶⠿⠶⠶⠖⠚⠛⠉⠁⠀⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⣰⠶⠶⡄⠀⠀
⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠀⠀⠀⠀⠀⠀⠀⠀⢠⡟⠀⠀⢹⠀⠀
⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⢤⢠⡆⠀⢸⡄⠀⠀⠀⠀⠀⠀⢀⡿⠁⠀⠀⡾⠀⠀
⢹⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠈⡇⠀⠸⣧⣠⠴⠶⠖⠲⢶⡞⠁⠀⢈⡼⢃⠀⠀
⠸⡆⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⡇⠀⠀⢿⠁⠄⣲⡶⠶⠿⢤⣄⡀⠛⢛⠉⢻⠀
⠀⢿⡀⠀⠀⠀⠀⠀⠀⠀⢸⠠⣇⠀⠀⠀⠀⠊⠁⠀⠀⠀⠀⠀⠙⢦⠈⠙⠓⣆
⠀⠈⢷⡀⠀⠀⠀⠀⠀⢠⠏⡀⣬⣹⣦⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠈⡿⠶⠶⠋
⠀⠀⠈⢷⡀⠀⠀⠀⠀⠘⠛⠛⠋⠀⠀⠀⠀⠀⠀⠄⠀⠀⠀⠀⠀⣼⠃⠀⠀⠀
⠀⠀⠀⠀⠙⢦⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠄⠀⠀⣠⡞⠁⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠈⠛⣷⢶⣦⣤⣄⣀⣠⣤⣤⠀⣶⠶⠶⠶⠛⠁⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣀⡀⠀⣰⠇⣾⠀⠀⠈⣩⣥⣄⣿⠀⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢿⡉⠳⡟⣸⠃⠀⠀⠀⠘⢷⣌⠉⠀⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠙⢦⣴⠏⠀⠀⠀⠀⠀⠀⠉⠳⠶⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
```

<img width="720" height="405" alt="wipedicks plowing 6 HDDs in parallel on an R710" src="https://github.com/user-attachments/assets/eaaac1f1-ce9e-4938-a166-bd733b8f612e" />

Six 12 TB drives, parallel wipe via SSH, with live colorized hex dumps watching the targets in real time. Before-and-after speaks for itself:

```
❤  xxd /dev/sda1 | head
00000000: 383d 447e 7e20 383d 3d3d 3d3d 3d3d 447e  8=D~~ 8=======D~
00000010: 2038 233d 3d3d 3d3d 3d3d 3d44 7e7e 2038   8#========D~~ 8
00000020: 233d 3d44 7e7e 2038 3d3d 3d3d 3d3d 3d3d  #==D~~ 8========
00000030: 3d44 2038 3d3d 3d3d 447e 7e7e 2038 3d3d  =D 8====D~~~ 8==
00000040: 3d3d 3d3d 3d3d 3d3d 3d44 7e20 3823 3d3d  =========D~ 8#==
```

## What's the Big Deal? 8===D

WipeDicks is a high-performance, multi-threaded file and device wiping tool that overwrites your data with a veritable bukkake of ASCII penises. Now with progress bars, SSD-awareness, and 23 randomized completion verbs (creamed, milked, nuked, gangbanged, splooged on...). It's designed for secure data erasure with more dick jokes than you can shake a stick at.

## Features (or as we like to call them, "Dick Pics") 8=D

- **Live `[8=========D~~~]` progress bar** with yellow→green gradient and a randomized completion verb
- **SSD/NVMe-aware safety** — refuses to wear out your flash, suggests the right native tool, or delegates to it via `--secure-erase`
- **Path-safety guards** — won't wipe `/`, `/etc`, swap files, or mounted block devices without explicit consent
- **Configurable everything** — shaft length, jizz amount, balls chance, buffer size, bar width, verify-on-write
- **SIMD-batched I/O** via the `wide` crate, bounded thread pool, Ctrl-C cleanup, `--gay` mode for those who want their bukkake in technicolor

> [!Warning]
> ## Cock Block Alert! 8==X
> This tool is designed to permanently destroy data. Use with extreme caution, or you might end up with blue balls (i.e., regret).
>
> Always double-check your targets before unleashing our ASCII members upon them.
>
> <details>
> <summary><h3> SSDs, NVMes, and the Sad Truth About Wear Leveling 8==X </h3></summary>
> 
> **Overwriting flash storage with multi-pass writes is both useless and harmful.** Yes, even with our beautiful penises.
>
> - **Useless**: SSDs and NVMes use a Flash Translation Layer that remaps logical writes to fresh NAND pages. Overwriting LBA 0 doesn't necessarily clobber the physical NAND cell that holds your old data — that data lingers in over-provisioned NAND until the controller decides to recycle it.
> - **Harmful**: Every pass burns finite program/erase cycles. A 7-pass DoD-style wipe on a 1 TB QLC SSD can consume meaningful percentage points of its lifetime.
> 
> So wipedicks **refuses by default** when you point it at a flash block device, and prints the right native command instead:
> 
> | Drive | Right tool |
> |---|---|
> | NVMe | `sudo nvme format /dev/nvmeXnY -s 1` (user-data erase) or `-s 2` (crypto erase, SED only) |
> | SATA SSD | `sudo hdparm --user-master u --security-set-pass p /dev/sdX && sudo hdparm --user-master u --security-erase p /dev/sdX` |
> | Either (fast, non-cryptographic) | `sudo blkdiscard /dev/X` |
> | Self-encrypting (TCG Opal) | `sudo sedutil-cli --revertNoErase` |
>
> Use `--secure-erase` to have wipedicks run the right one for you. Use `--rape` to override the refusal and proceed with the (ineffective, damaging) overwrite anyway. Wiping individual *files* on a flash filesystem still works — wipedicks prints a one-time warning and continues, because at least `unlink(2)` actually deletes the inode.
> </details>

## Installation (Getting It Up and Running) 8===D~

```bash
# from crates.io (recommended)
cargo install wipedicks

# from source
git clone https://github.com/vxfemboy/wipedicks && cd wipedicks
cargo build --release    # binary lands at target/release/wipedicks
```

Prebuilt binaries for Linux (x86_64 + aarch64), macOS (x86_64 + aarch64), and Windows are also attached to each [GitHub release](https://github.com/vxfemboy/wipedicks/releases).

## Usage (How to Handle Your Tool) 8====D

```bash
# Give a single file the business
wipedicks /path/to/file

# Recursively shred a directory
wipedicks -r /path/to/directory

# Forcefully shove dicks into the disk 
sudo wipedicks --rape /dev/sdX 
```

<details>
<summary><b>All options</b> (or run <code>wipedicks --help</code>)</summary>

- `-r, --recursive`: Go deep into directories
- `-n, --numrounds <N>`: How many overwrite passes per target (default 1)
- `-w, --wipefree`: After the wipe, fill free space with dicks until ENOSPC
- `-s, --slow`: One-penis-per-write path (no buffering, no SIMD)
- `--rape`: Override every safety check. Nuclear safe-word.
- `--secure-erase`: Delegate to the drive's native secure-erase
- `--shaft-min <N>` / `--shaft-max <N>`: Range of `=` per penis (default 1..12)
- `--jizz-min <N>`  / `--jizz-max <N>`: Range of `~` per penis (default 0..3)
- `--balls-chance <P>`: Probability of a `#` base ring, 0.0..1.0 (default 0.5)
- `--buffer-size <BYTES>`: Write batch size (default 1 MiB)
- `--bar-width <N>`: Width of the progress bar (default 16)
- `--no-progress`: Disable progress bars (auto-disabled when stderr isn't a TTY)
- `--gay`: Cycle the bar across the rainbow with bright-white jizz
- `--verify`: After each round, read the file back and confirm every byte is a valid penis character
- `--dry-run`: Print one buffer of generated dicks to stdout and exit

</details>

> [!CAUTION]
> ## Wrap It Before You Tap It
> - Always check your target twice, or you might screw the wrong hole
> - Wiping entire devices is like unprotected data sex — there's no going back
> - Don't run this on system files unless you want your computer to go limp
> - This tool is for giggles and learning. For serious business, use protection (i.e., certified tools)

## Credits

This project was brought up to me by [Drewsif/wipedicks](https://github.com/Drewsif/wipedicks/) — all credits for the idea goes to the original creator. I just rewrote it in Rust.

## Disclaimer (Cover Your Ass-ets)

The authors aren't responsible for any data loss or damage. Use at your own risk, you kinky data destroyer!

Remember:
> With great power comes great responsibili-D.
> Use wipedicks wisely, and may your data always rest in pieces! 8====D~~~
