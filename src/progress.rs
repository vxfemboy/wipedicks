//! Live ASCII-cock progress theatre.
//!
//! Each wipe target gets its own bar — penis grows from `8D` to `8==============D`
//! as the file dies. Jizz drips in bright white. Bytes go yellow → green like
//! a traffic light flipping to "ohyeah". The trailing filename is painted red
//! so you can see exactly whose data is currently getting plowed.
//!
//! On completion, the file gets a custom send-off: one of ~20 verbs (fucked,
//! creamed, milked, nuked...) chosen at random, each with its own color and
//! formatting. Because "wiped" was getting boring.
//!
//! Format (resolves GitHub issue #1):
//!     [8D~~~~~~~~~~~~~~~]   0.00 B/64.00 MiB   0%  /path/to/victim
//!     [8=======D~~~~~~~~]  32.00 MiB/64.00 MiB  50%  /path/to/victim
//!     [8===============D]  64.00 MiB/64.00 MiB 100%  /path/to/victim 💦 creamed
//!
//! `indicatif`'s built-in `progress_chars` treats the middle character as a
//! "tip" that disappears at 100%, so we render the bar ourselves via a custom
//! `with_key` callback to guarantee the `D` stays in frame all the way to the
//! right edge — wouldn't want to lose the head at the finish line.

use std::path::Path;

use console::Style;
use indicatif::{HumanBytes, MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use rand::Rng;

// xterm-256 indices for the gay rainbow (ROYGBIV) — applied to the shaft.
const RAINBOW: &[u8] = &[196, 208, 226, 46, 51, 21, 201];

// Yellow → green gradient for the wiped-bytes counter and percent.
const GRADIENT_FULL: u8 = 46; // bright green — finishes hard
const TOTAL_COLOR: u8 = 51; // cyan — the goalposts
const JIZZ_COLOR: u8 = 231; // bright white — pearl drip
const PATH_COLOR: u8 = 196; // red — the victim's name in lights

fn gradient_for(frac: f32) -> u8 {
    if frac >= 1.0 {
        return GRADIENT_FULL;
    }
    match (frac * 10.0) as u32 {
        0..=1 => 226, // bright yellow — just getting hard
        2..=3 => 227, // yellow
        4..=5 => 191, // yellow-green
        6..=7 => 154, // lime-yellow
        _ => 82,      // lime green — about to bust
    }
}

fn paint(s: &str, color: u8, enabled: bool) -> String {
    if enabled {
        Style::new()
            .force_styling(true)
            .color256(color)
            .apply_to(s)
            .to_string()
    } else {
        s.to_string()
    }
}

/// Wrap a path in red ANSI for the progress message tail.
pub fn red_path(path: &Path) -> String {
    if console::colors_enabled_stderr() {
        paint(&path.display().to_string(), PATH_COLOR, true)
    } else {
        path.display().to_string()
    }
}

/// One of the many ways a dick can finish off a file.
struct Verb {
    word: &'static str,
    color: u8,
    bold: bool,
    underline: bool,
    italic: bool,
    /// Optional emoji prefix. Goes BEFORE the styled word, with a space.
    emoji: Option<&'static str>,
}

/// The hall of fame. Every wipe ends with one of these, randomly picked.
const VERBS: &[Verb] = &[
    Verb {
        word: "fucked",
        color: 196,
        bold: true,
        underline: true,
        italic: false,
        emoji: None,
    },
    Verb {
        word: "wiped",
        color: 46,
        bold: true,
        underline: false,
        italic: false,
        emoji: None,
    },
    Verb {
        word: "deleted",
        color: 196,
        bold: true,
        underline: false,
        italic: false,
        emoji: None,
    },
    Verb {
        word: "destroyed",
        color: 196,
        bold: true,
        underline: false,
        italic: false,
        emoji: Some("💀"),
    },
    Verb {
        word: "obliterated",
        color: 208,
        bold: true,
        underline: true,
        italic: false,
        emoji: Some("💥"),
    },
    Verb {
        word: "purged",
        color: 226,
        bold: true,
        underline: false,
        italic: false,
        emoji: None,
    },
    Verb {
        word: "creamed",
        color: 231,
        bold: true,
        underline: false,
        italic: false,
        emoji: Some("💦"),
    },
    Verb {
        word: "milked",
        color: 250,
        bold: false,
        underline: false,
        italic: false,
        emoji: Some("🥛"),
    },
    Verb {
        word: "blasted",
        color: 196,
        bold: true,
        underline: false,
        italic: false,
        emoji: Some("💥"),
    },
    Verb {
        word: "pissed on",
        color: 226,
        bold: false,
        underline: false,
        italic: true,
        emoji: Some("💦"),
    },
    Verb {
        word: "annihilated",
        color: 196,
        bold: true,
        underline: true,
        italic: false,
        emoji: Some("☢"),
    },
    Verb {
        word: "demolished",
        color: 208,
        bold: true,
        underline: false,
        italic: false,
        emoji: None,
    },
    Verb {
        word: "ravaged",
        color: 197,
        bold: true,
        underline: false,
        italic: false,
        emoji: None,
    },
    Verb {
        word: "pounded",
        color: 201,
        bold: true,
        underline: false,
        italic: false,
        emoji: Some("🍆"),
    },
    Verb {
        word: "nuked",
        color: 226,
        bold: true,
        underline: false,
        italic: false,
        emoji: Some("☢"),
    },
    Verb {
        word: "gangbanged",
        color: 201,
        bold: true,
        underline: true,
        italic: false,
        emoji: None,
    },
    Verb {
        word: "railed",
        color: 165,
        bold: true,
        underline: false,
        italic: false,
        emoji: None,
    },
    Verb {
        word: "splooged on",
        color: 231,
        bold: false,
        underline: false,
        italic: true,
        emoji: Some("💦"),
    },
    Verb {
        word: "drilled",
        color: 208,
        bold: true,
        underline: false,
        italic: false,
        emoji: None,
    },
    Verb {
        word: "facialized",
        color: 231,
        bold: true,
        underline: true,
        italic: false,
        emoji: Some("💦"),
    },
    Verb {
        word: "jizzed on",
        color: 231,
        bold: true,
        underline: false,
        italic: false,
        emoji: Some("💦"),
    },
    Verb {
        word: "creampied",
        color: 199,
        bold: true,
        underline: false,
        italic: false,
        emoji: Some("🍑"),
    },
    Verb {
        word: "rawdogged",
        color: 197,
        bold: true,
        underline: true,
        italic: false,
        emoji: None,
    },
];

fn pick_verb() -> &'static Verb {
    let idx = rand::thread_rng().gen_range(0..VERBS.len());
    &VERBS[idx]
}

fn style_verb(v: &Verb) -> String {
    let mut style = Style::new().force_styling(true).color256(v.color);
    if v.bold {
        style = style.bold();
    }
    if v.underline {
        style = style.underlined();
    }
    if v.italic {
        style = style.italic();
    }
    style.apply_to(v.word).to_string()
}

/// Bury a file with a randomly chosen send-off. Used by `wipe::wipe_file`
/// when the deed is done.
pub fn celebrate(pb: &ProgressBar, path: &Path) {
    let colors = console::colors_enabled_stderr();
    let verb = pick_verb();
    let body = if colors {
        let path_str = paint(&path.display().to_string(), PATH_COLOR, true);
        let styled = style_verb(verb);
        match verb.emoji {
            Some(e) => format!("{} {} {}", path_str, e, styled),
            None => format!("{} {}", path_str, styled),
        }
    } else {
        match verb.emoji {
            Some(e) => format!("{} {} {}", path.display(), e, verb.word),
            None => format!("{} {}", path.display(), verb.word),
        }
    };
    pb.finish_with_message(body);
}

/// Mark a wipe that shit the bed. Bright-red, bold "FUCKED UP" so the user
/// notices something failed in a sea of green successes.
pub fn lament(pb: &ProgressBar, path: &Path, err: &str) {
    let colors = console::colors_enabled_stderr();
    let body = if colors {
        let path_str = paint(&path.display().to_string(), PATH_COLOR, true);
        let tag = Style::new()
            .force_styling(true)
            .color256(196)
            .bold()
            .apply_to("FUCKED UP")
            .to_string();
        format!("{} {}: {}", path_str, tag, err)
    } else {
        format!("{} FUCKED UP: {}", path.display(), err)
    };
    pb.abandon_with_message(body);
}

#[derive(Clone)]
pub struct Progress {
    multi: Option<MultiProgress>,
    bar_width: usize,
    gay: bool,
}

impl Progress {
    pub fn new(enabled: bool, bar_width: usize, gay: bool) -> Self {
        Self {
            multi: enabled.then(MultiProgress::new),
            bar_width,
            gay,
        }
    }

    pub fn add_file(&self, path: &Path, total_bytes: u64) -> Option<ProgressBar> {
        let multi = self.multi.as_ref()?;
        let pb = multi.add(ProgressBar::new(total_bytes));
        pb.set_style(self.style());
        pb.set_message(red_path(path));
        Some(pb)
    }

    fn style(&self) -> ProgressStyle {
        let inner = self.bar_width.saturating_sub(1).max(2);
        let gay = self.gay;
        let colors = console::colors_enabled_stderr();

        let template = "[{dickbar}] {bytes}/{total_bytes} {percent}%  {msg}";
        ProgressStyle::with_template(template)
            .expect("static template is valid")
            .with_key(
                "dickbar",
                move |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    let frac = state.fraction().clamp(0.0, 1.0);
                    let shaft = ((inner - 1) as f32 * frac).round() as usize;
                    let shaft = shaft.min(inner - 1);
                    let jizz = inner - 1 - shaft;
                    let bar: String = std::iter::once('8')
                        .chain((0..shaft).map(|_| '='))
                        .chain(std::iter::once('D'))
                        .chain((0..jizz).map(|_| '~'))
                        .collect();

                    if colors {
                        // Jizz is white in every mode — that's the whole point.
                        // The shaft only goes rainbow when --gay is on.
                        let mut color_idx = 0;
                        for ch in bar.chars() {
                            if ch == '~' {
                                let style = Style::new().force_styling(true).color256(JIZZ_COLOR);
                                let _ = write!(w, "{}", style.apply_to(ch));
                            } else if gay {
                                let style = Style::new()
                                    .force_styling(true)
                                    .color256(RAINBOW[color_idx % RAINBOW.len()]);
                                let _ = write!(w, "{}", style.apply_to(ch));
                                color_idx += 1;
                            } else {
                                let _ = w.write_char(ch);
                            }
                        }
                    } else {
                        let _ = w.write_str(&bar);
                    }
                },
            )
            .with_key(
                "bytes",
                move |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    let text = format!("{:>10}", HumanBytes(state.pos()).to_string());
                    let _ = w.write_str(&paint(&text, gradient_for(state.fraction()), colors));
                },
            )
            .with_key(
                "total_bytes",
                move |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    let total = state.len().unwrap_or(0);
                    let text = format!("{:<10}", HumanBytes(total).to_string());
                    let _ = w.write_str(&paint(&text, TOTAL_COLOR, colors));
                },
            )
            .with_key(
                "percent",
                move |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    let p = (state.fraction() * 100.0).round() as u32;
                    let text = format!("{:>3}", p);
                    let _ = w.write_str(&paint(&text, gradient_for(state.fraction()), colors));
                },
            )
    }
}
