//! Randomness for "confused stonks" and "perfectly balanced as all things should be".

use std::hash::{BuildHasher, RandomState};

/// A small, non-cryptographic random number generator that needs nothing but std: a
/// [`RandomState`] holds hash keys that the standard library seeds from the operating system's
/// randomness, and hashing an incrementing counter with them yields a random sequence.
#[derive(Default)]
pub struct Rng {
    keys: RandomState,
    counter: u64,
}

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.keys.hash_one(self.counter)
    }

    /// A random number in `0..n` (`n` must not be 0).
    pub fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "Rng::below(0)");
        // Lemire's multiply-shift reduction: maps the 64-bit value onto 0..n (bias < n / 2^64).
        ((u128::from(self.next_u64()) * n as u128) >> 64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_stays_in_range_and_varies() {
        let mut rng = Rng::default();
        let mut seen = [false; 7];
        for _ in 0..1000 {
            seen[rng.below(7)] = true;
        }
        assert!(seen.iter().all(|&s| s));
        assert_eq!(rng.below(1), 0);
    }
}
