use crate::feistel::{FeistelNetwork, KEY_LEN};
use std::iter::Skip;

pub mod feistel;

pub struct Wordle<'a, T: AsRef<str>> {
    day: usize,
    words: &'a [T],
    network: FeistelNetwork,
}

impl<'a, T: AsRef<str>> Wordle<'a, T> {
    pub fn new(words: &'a [T]) -> Self {
        let network_len = FeistelNetwork::next_len(words.len());
        Wordle {
            day: 0,
            words,
            network: FeistelNetwork::new(network_len),
        }
    }

    pub fn new_with_key(words: &'a [T], key: [u8; KEY_LEN]) -> Self {
        let network_len = FeistelNetwork::next_len(words.len());
        Wordle {
            day: 0,
            words,
            network: FeistelNetwork::new_with_key(network_len, key),
        }
    }
}

impl<'a, T: AsRef<str>> Iterator for Wordle<'a, T> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut idx = self.network.permute(self.day as u64);
        while idx >= self.words.len() as u64 {
            idx = self.network.permute(idx);
        }

        let word = self.words[idx as usize].as_ref();

        self.day += 1;
        if self.day == self.words.len() {
            self.day = 0;
        }
        Some(word)
    }

    /*fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        self.day = (self.day + n) % self.words.len();
        Ok(())
    }*/

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.day = (self.day + n) % self.words.len();
        self.next()
    }
}
