# Contributing (Sword Fighters Wanted)

![Sword Fighting](https://media.giphy.com/media/l1ugaivowDSqyE3KM/giphy.gif)

Want to cross streams? Pull requests are welcome. Before you start whittling, read this end to end — the project has a load-bearing tone and a strict tooling bar, and most rejected PRs would have been merges if the contributor had skimmed the rules first.

## The vibe (three rules)

1. **Truth wraps in jokes, not the other way around.** Every comment, doc-string, error message, and `--help` line is a dick joke as the wrapper around the technical payload. If the joke obscures the truth, the joke loses. If the truth has no joke, that's also fine — comments earn their place by being load-bearing.

2. **Default to no comments.** Rust's identifiers and types do most of the documenting. Only comment when the *why* is non-obvious (a hidden invariant, a workaround, a perf detail, a platform quirk). Then dress that *why* in the project voice.

3. **The joke is about dicks and sex acts. Not about people.** Anything that punches down — slurs, bigotry, harassment of named individuals — gets rejected on sight regardless of how clever the wrapper is. `--gay` (rainbow mode) and `--rape` (override safe-word) are project-internal *action-shape* names, not pejoratives aimed at anyone. If you can't tell the difference, this isn't the project for you.

## Voice cheat-sheet

Examples pulled verbatim from the codebase. Read these before you write your own.

### Good — `--help` doc string (lives in `src/cli.rs`)

```rust
/// Take it slow, big boy. One penis per write(2). Useful for sanity
/// comparison or terminals with a kink for tiny syscalls. ~5000x more
/// syscalls than the buffered path.
#[arg(short = 's', long)]
pub slow: bool,
```

The joke (basic missionary / one-penis-per-write) sits *on top of* the technical claim (one `write(2)` per generated dick, useful for syscall-count comparison, ~5000× ratio against the buffered path). The reader gets both.

### Good — module header (lives in `src/safety.rs`)

```rust
//! Cock-block department. Stops you from face-fucking your /etc/ when your
//! hand slipped, your /boot/ when you meant /tmp/, or the swap file your
//! kernel is currently using to remember things.
```

Names the module's job, then lists the actual hazards it catches.

### Good — invariant comment (lives in `src/wipe.rs`)

```rust
// Refill ONCE per round. Within a round we rewrite the same dicks for
// every chunk: the content is non-cryptographic ASCII art, so reusing
// the buffer is invisible. Across rounds we get fresh variety.
buf.fill();
```

A load-bearing perf note. Tells future-you (or future-contributor-you) why this isn't an accidental optimization to undo.

### Bad — sterile rewrite

```rust
/// Disables buffered writing for compatibility purposes.
pub slow: bool,
```

Accurate, but kills the voice. PRs that "professionalize" existing copy get closed.

### Bad — joke with no payload

```rust
// nut nut nut
buf.fill();
```

This is the joke without the why. Either explain the perf reason or delete the comment.

### Bad — punching at people

```rust
// only <slur> would name a variable this badly
```

Hard no, regardless of how true it feels in the moment.

## Code style + tooling

Reproduce locally before pushing. CI on `ubuntu-latest`, `macos-latest`, and `windows-latest` runs all four:

```bash
cargo fmt --all -- --check                           # exit 0
cargo clippy --release --all-targets -- -D warnings  # exit 0, no warnings
cargo build --release                                # exit 0
cargo test --release                                 # all tests pass
```

- **Rust 2021, snake_case, no `unsafe`.** If you genuinely need `unsafe`, justify it in the PR description.
- **Errors via `anyhow`.** Bubble to `main` and print with `{:#}` so the context chain shows up.
- **Per-platform code uses `#[cfg(target_os = "...")]`.** The fallback branch always compiles cleanly — silence dead-code warnings on non-supported platforms via `#[cfg_attr(not(target_os = "linux"), allow(dead_code))]` on the enum, not blanket `#[allow(dead_code)]`.
- **New deps need a one-line justification in the PR description.** We prefer leaning on existing crates (`indicatif`, `wide`, `console`, `anyhow`, `clap`, `ctrlc`, `rand`) over pulling in new ones for marginal wins.
- **Public items** (`pub fn` / `pub struct` / `pub enum`) get a doc-string in the project voice. **Private items** don't, unless the *why* is non-obvious.

## Module layout (so your PR lands in the right file)

| Module | What it does |
|---|---|
| `main.rs` | Arg parse → safety check → drive policy → bounded thread pool → exit code. |
| `cli.rs` | clap derive `Args` + per-flag `--help` copy. **Touch this and you change user-visible help text.** Match the existing voice. |
| `dicks.rs` | The actual penis generator. SIMD batched fill via `wide::u8x32`, per-thread `SmallRng`. |
| `wipe.rs` | `wipe_file`, `wipe_freespace`, `verify_file`. Cleanup pass always runs. |
| `drive.rs` | SSD/NVMe detection + refusal + `--secure-erase` delegation. Linux + macOS supported; Windows is a stub. |
| `safety.rs` | Path-safety guard. System dirs, swap, mounted devices. `--rape` bypasses. |
| `cleanup.rs` | Process-wide ctrl-c cleanup registry. |
| `progress.rs` | indicatif bars + the `VERBS` table for completion messages. |

If your PR touches more than two of these, it should probably be two PRs.

## PR checklist

- [ ] One concern per PR. Refactors stay separate from feature work.
- [ ] PR description matches the project voice. Bullet points are fine; corpo-speak is not.
- [ ] CI green on Linux, macOS, Windows.
- [ ] Tests added if you introduced a public function with non-trivial behavior.
- [ ] If you added or removed a CLI flag, the README's `<details>All options</details>` block stays in sync.
- [ ] For big changes (>200 LOC or a new module): open an issue first to align on the approach. We don't want anyone burning a weekend on a PR we'd close.

## What we want

- **Pick up `help wanted` issues.**
- **Bug reports with reproduction steps.** What you ran, what happened, what you expected, what OS + Rust version.
- **New completion verbs** for the `VERBS` table in `src/progress.rs`. Submit a PR adding one or three, each with color / bold / italic / emoji choices. Keep them on-brand.
- **More `--shaft-*` / `--jizz-*` knobs** that make the generator more configurable without exploding the flag list.

## What we don't want

- **"Modernization" PRs that swap our tone for boilerplate corporate copy.** The voice is the project.
- **Heavy new dependencies.** No `tokio`, no async runtimes, no web servers, no GUI toolkits. This is a single-binary CLI.
- **New override flags layered on top of `--rape`.** One nuclear safe-word is enough.
- **Anything that breaks the SSD-refusal default.** SSD-safety is the load-bearing feature; if you have a real reason to bypass it on a specific run, `--rape` already exists.

## The actual social contract

- Be a person to other contributors. Disagreement is fine; ad hominem is not.
- Maintainer reviews when maintainer reviews. Pinging "any update?" after 7+ days is fine; pinging hourly is not.
- If your PR sits unmerged for 2+ weeks with no maintainer response, ping in the PR thread. If it sits for 4+ weeks, escalate by tagging in the issue tracker.

8====D~~~
