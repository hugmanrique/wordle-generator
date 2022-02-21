use rand::rngs::OsRng;
use rand::RngCore;
use std::hash::Hasher;
use wyhash::WyHash;

pub const KEY_LEN: usize = 32;
const DEFAULT_ROUNDS: u8 = 8;

pub struct FeistelNetwork {
    #[cfg(debug_assertions)]
    len: u8,
    left_shift: u8,
    right_mask: u64,
    key: [u8; KEY_LEN],
    rounds: u8,
}

impl FeistelNetwork {
    pub fn new(len: u8) -> Self {
        let mut key = [0u8; KEY_LEN];
        OsRng.fill_bytes(&mut key);
        Self::new_with_key(len, key)
    }

    pub fn new_with_key(len: u8, key: [u8; KEY_LEN]) -> Self {
        assert!(0 < len && len < u64::BITS as u8);
        // left and right substrings must have the same length
        assert_eq!(len % 2, 0, "unbalanced network, len {} should be even", len);

        let left_shift = len / 2;
        Self {
            #[cfg(debug_assertions)]
            len,
            left_shift,
            right_mask: (1u64 << left_shift) - 1,
            key,
            rounds: DEFAULT_ROUNDS,
        }
    }

    pub fn permute(&self, input: u64) -> u64 {
        #[cfg(debug_assertions)]
        debug_assert_eq!(
            input >> self.len,
            0,
            "input (is {}) bit length should be < len (is {})",
            input,
            self.len
        );

        let mut left = input >> self.left_shift;
        let mut right = input & self.right_mask;
        for i in 0..self.rounds {
            let f = self.round(i, right) & self.right_mask;
            let new_right = left ^ f;
            left = right;
            right = new_right;
        }

        (left << self.left_shift) | right
    }

    pub fn round(&self, round: u8, right: u64) -> u64 {
        let mut hasher = WyHash::default();
        hasher.write(&self.key);
        hasher.write_u8(round);
        hasher.write_u64(right);
        hasher.finish()
    }

    pub fn next_len(domain_size: usize) -> u8 {
        assert!(domain_size > 0, "element count must be positive");
        let mut len = usize::BITS - domain_size.leading_zeros();
        if len % 2 == 1 {
            len += 1;
        }
        len.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{FeistelNetwork, KEY_LEN};

    #[test]
    fn permute() {
        let key = [0; KEY_LEN];
        let network = FeistelNetwork::new_with_key(12, key);

        assert_eq!(network.permute(1234), 2458);
        assert_eq!(network.permute(2134), 3247);
        assert_eq!(network.permute(0x0F00), 2211);
        assert_eq!(network.permute(1234), 2458);
    }

    #[test]
    fn bijective() {
        let key = [0xAB; KEY_LEN];
        let network = FeistelNetwork::new_with_key(12, key);

        let mut seen = [false; 4096];
        for value in 0..4096 {
            let result = network.permute(value) as usize;
            assert!(!seen[result]);
            seen[result] = true;
        }
        assert!(seen.iter().all(|&x| x))
    }

    #[test]
    fn idempotent() {
        let key = [0xCD; KEY_LEN];
        let network = FeistelNetwork::new_with_key(8, key);

        for value in 0..256 {
            let expected = network.permute(value);
            for _ in 0..4 {
                assert_eq!(network.permute(value), expected);
            }
        }
    }

    #[test]
    #[should_panic]
    fn len_must_be_positive() {
        FeistelNetwork::new(0);
    }

    #[test]
    #[should_panic]
    fn unbalanced() {
        // len must be even
        FeistelNetwork::new(63);
    }

    #[test]
    #[should_panic]
    fn one_bit_is_unbalanced() {
        FeistelNetwork::new(1);
    }

    #[test]
    fn next_len() {
        assert_eq!(FeistelNetwork::next_len(2), 2);
        assert_eq!(FeistelNetwork::next_len(347), 10);
    }
}
