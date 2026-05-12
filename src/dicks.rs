//! Random-penis factory. SIMD-juiced where it counts. Per-thread,
//! no sword-crossing.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use wide::u8x32;

/// The anatomical knobs for our generated dicks.
#[derive(Clone, Debug)]
pub struct DickConfig {
    pub shaft_min: usize,
    pub shaft_max: usize,
    pub jizz_min: usize,
    pub jizz_max: usize,
    pub balls_chance: f32,
}

impl DickConfig {
    /// Worst-case length of a single dick in bytes. Used to pre-size the buffer
    /// so we never realloc mid-stroke.
    pub fn max_dick_len(&self) -> usize {
        // 8 + optional # + shaft + D + jizz + space
        1 + 1 + self.shaft_max + 1 + self.jizz_max + 1
    }
}

/// Per-thread reusable cock-buffer. One thread owns one — sharing dicks
/// across threads is how you end up with weird race conditions and bent
/// shafts. Call `fill()` to load it up, then `as_bytes()` for the goods.
pub struct DickBuf {
    cfg: DickConfig,
    buf: Vec<u8>,
    rng: SmallRng,
    capacity: usize,
}

impl DickBuf {
    pub fn new(cfg: DickConfig, capacity: usize) -> Self {
        let slack = cfg.max_dick_len();
        Self {
            cfg,
            buf: Vec::with_capacity(capacity + slack),
            rng: SmallRng::from_entropy(),
            capacity,
        }
    }

    /// Pump the buffer full of fresh dicks. Exactly `capacity` bytes when done.
    pub fn fill(&mut self) {
        self.buf.clear();
        while self.buf.len() < self.capacity {
            push_one(&mut self.rng, &self.cfg, &mut self.buf);
        }
        self.buf.truncate(self.capacity);
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }
}

/// Jerk out one random penis and append it to `out`. Shared between the
/// buffered fast path and the `--slow` one-dick-per-write masochism path.
pub fn push_one(rng: &mut SmallRng, cfg: &DickConfig, out: &mut Vec<u8>) {
    let balls = rng.gen::<f32>() < cfg.balls_chance;
    let shaft = if cfg.shaft_min == cfg.shaft_max {
        cfg.shaft_min
    } else {
        rng.gen_range(cfg.shaft_min..=cfg.shaft_max)
    };
    let jizz = if cfg.jizz_min == cfg.jizz_max {
        cfg.jizz_min
    } else {
        rng.gen_range(cfg.jizz_min..=cfg.jizz_max)
    };

    out.push(b'8');
    if balls {
        out.push(b'#');
    }
    fill_run(out, b'=', shaft);
    out.push(b'D');
    fill_run(out, b'~', jizz);
    out.push(b' ');
}

pub fn new_rng() -> SmallRng {
    SmallRng::from_entropy()
}

/// Is this byte part of the canonical penis alphabet? The doorman for
/// `--verify`: any byte that isn't on the list means a non-dick snuck through.
#[inline]
pub fn is_dick_byte(b: u8) -> bool {
    matches!(b, b'8' | b'=' | b'#' | b'D' | b'~' | b' ')
}

/// Splat `n` copies of `ch` into `out` 32 bytes at a time. The SIMD shaft
/// extender — turns a 1-byte char into long, hard rows of repeated symbols
/// with one vector instruction per chunk.
fn fill_run(out: &mut Vec<u8>, ch: u8, n: usize) {
    if n == 0 {
        return;
    }
    let start = out.len();
    out.resize(start + n, 0);
    let slice = &mut out[start..start + n];
    let chunk = u8x32::splat(ch).to_array();
    let mut i = 0;
    while i + 32 <= slice.len() {
        slice[i..i + 32].copy_from_slice(&chunk);
        i += 32;
    }
    while i < slice.len() {
        slice[i] = ch;
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cfg() -> DickConfig {
        DickConfig {
            shaft_min: 3,
            shaft_max: 8,
            jizz_min: 0,
            jizz_max: 3,
            balls_chance: 0.5,
        }
    }

    #[test]
    fn fill_yields_exact_capacity() {
        let mut buf = DickBuf::new(test_cfg(), 4096);
        buf.fill();
        assert_eq!(buf.as_bytes().len(), 4096);
    }

    #[test]
    fn output_starts_with_eight() {
        let mut buf = DickBuf::new(test_cfg(), 64);
        buf.fill();
        assert_eq!(buf.as_bytes()[0], b'8');
    }

    #[test]
    fn fill_run_vectorized_matches_scalar() {
        let mut a = Vec::new();
        fill_run(&mut a, b'=', 100);
        assert_eq!(a.len(), 100);
        assert!(a.iter().all(|&b| b == b'='));
    }
}
