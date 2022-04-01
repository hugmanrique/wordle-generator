mod feistel;

use crate::feistel::FeistelNetwork;
use bytemuck::bytes_of_mut;
use sha2::{Digest, Sha512};

pub type Seed = [u8; 32];

/// A cryptographically secure random wordle generator.
pub struct Wordle<'a, T: AsRef<str>> {
    words: &'a [T],
    window_len: u64,
    seed: Seed,
    hasher: Sha512,
    // The Luby-Rackoff theorem shows that 4 rounds are enough to resist all
    // adaptive chosen plaintext and chosen ciphertext attacks, for sufficiently
    // large block sizes. However, we support arbitrarily small domains.
    network: FeistelNetwork<8>,
}

impl<'a, T: AsRef<str>> Wordle<'a, T> {
    /// Creates a new generator seeded via [`rand::random`].
    pub fn new(words: &'a [T], window_len: u64) -> Self {
        Self::from_seed(words, window_len, rand::random())
    }

    /// Creates a new generator using a seed.
    pub fn from_seed(words: &'a [T], window_len: u64, seed: Seed) -> Self {
        Self {
            words,
            window_len,
            seed,
            hasher: Sha512::new(),
            network: FeistelNetwork::for_domain(words.len()),
        }
    }

    fn update_window(&mut self, window: u64) {
        // SHA512 produces enough output for 8 64-bit round keys
        let key_bytes = bytes_of_mut(self.network.keys_mut());
        self.hasher.update(&self.seed);
        self.hasher.update(&window.to_ne_bytes());
        Digest::finalize_into_reset(&mut self.hasher, key_bytes.into());
    }

    /// Returns the word for the given day.
    pub fn get(&mut self, day: u64) -> &'a str {
        let window = day / self.window_len;
        self.update_window(window);

        let day = day % self.window_len;
        let mut idx = self.network.permute(day);
        while idx >= self.words.len() as u64 {
            idx = self.network.permute(idx);
        }
        self.words[idx as usize].as_ref()
    }
}
