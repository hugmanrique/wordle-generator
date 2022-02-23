use std::hash::Hasher;
use wyhash::WyHash;

// The Luby-Rackoff theorem shows that 4 rounds are enough to resist all
// adaptive chosen plaintext and chosen ciphertext attacks, for sufficiently
// large block sizes. However, we support arbitrarily small domains.
pub const ROUNDS: usize = 8;

// WyHash uses a 64-bit seed.
const SUBKEY_LEN: usize = std::mem::size_of::<u64>();

/// The length of a [Feistel network] key in bytes.
///
/// [Feistel network](`Feistel network`)
const KEY_LEN: usize = ROUNDS * SUBKEY_LEN;

/// A Feistel network of length `2n` provides a random permutation of
/// the set {0, 1, ..., 2^(2n - 1)}, based on the given round keys.
pub struct FeistelNetwork {
    keys: [u64; ROUNDS],
    upper_shift: u8,
    lower_mask: u64,
}

impl FeistelNetwork {
    /// Creates a Feistel network with a randomly-generated key to
    /// permute a domain with `2^bit_len` elements.
    ///
    /// # Panics
    ///
    /// Panics if `bit_len` is odd, equal to zero, or greater than `u64::BITS`.
    ///
    /// # Examples
    ///
    /// ```
    /// use wordle_generator::feistel::FeistelNetwork;
    ///
    /// let network = FeistelNetwork::new(4);
    ///
    /// assert_eq!(network.permute(1), network.permute(1));
    /// assert_ne!(network.permute(2), network.permute(3));
    /// ```
    ///
    /// ```should_panic
    /// use wordle_generator::feistel::FeistelNetwork;
    ///
    /// FeistelNetwork::new(23);
    /// ```
    ///
    /// ```should_panic
    /// use wordle_generator::feistel::FeistelNetwork;
    ///
    /// FeistelNetwork::new(u64::BITS as u8 + 1);
    /// ```
    pub fn new(bit_len: u8) -> Self {
        let keys = rand::random();
        Self::with_keys(bit_len, keys)
    }

    /// Creates a Feistel network with the given round keys to
    /// permute a domain with `2^bit_len` elements.
    ///
    /// # Panics
    ///
    /// Panics if `bit_len` is odd, equal to zero, or greater than `u64::BITS`.
    ///
    /// # Examples
    ///
    /// ```
    /// use wordle_generator::feistel::{FeistelNetwork, ROUNDS};
    ///
    /// let round_keys = [0xAB; ROUNDS];
    /// let network = FeistelNetwork::with_keys(12, round_keys);
    ///
    /// assert_eq!(network.permute(1234), 26);
    /// assert_eq!(network.permute(2134), 2827);
    /// assert_eq!(network.permute(0x0F00), 1964);
    /// assert_eq!(network.permute(1234), 26);
    /// ```
    pub fn with_keys(bit_len: u8, keys: [u64; ROUNDS]) -> Self {
        assert!(
            0 < bit_len && bit_len <= u64::BITS as u8,
            "bit_len (is {}) should be positive and < {}",
            bit_len,
            u64::BITS
        );
        assert_eq!(bit_len % 2, 0, "bit_len (is {}) should be even", bit_len);
        let upper_shift = bit_len / 2;
        Self {
            keys,
            upper_shift,
            lower_mask: (1u64 << upper_shift) - 1,
        }
    }

    pub fn permute(&self, input: u64) -> u64 {
        // TODO: assert input < max.
        let mut upper = input >> self.upper_shift;
        let mut lower = input & self.lower_mask;
        for i in 0..ROUNDS {
            let new_lower = upper ^ self.round(lower, self.keys[i]);
            upper = lower;
            lower = new_lower;
        }
        lower << self.upper_shift | upper
    }

    fn round(&self, lower: u64, key: u64) -> u64 {
        let mut hasher = WyHash::with_seed(key);
        // todo: key whitening
        hasher.write_u64(lower);
        hasher.finish() & self.lower_mask
    }

    /// Returns a mutable reference to the array of round keys
    /// used by the Feistel network. This can be used to rotate
    /// the keys.
    pub fn keys_mut(&mut self) -> &mut [u64; ROUNDS] {
        &mut self.keys
    }

    /// Computes the minimum bit length of a Feistel network that
    /// can permute `domain_size` elements.
    ///
    /// # Panics
    ///
    /// Panics if the domain size is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use wordle_generator::feistel::FeistelNetwork;
    ///
    /// assert_eq!(FeistelNetwork::bit_len_for(2), 2);
    /// assert_eq!(FeistelNetwork::bit_len_for(347), 10);
    /// assert_eq!(FeistelNetwork::bit_len_for(0x45_F58D), 24);
    /// assert_eq!(FeistelNetwork::bit_len_for(usize::MAX), usize::BITS as u8);
    /// ```
    pub fn bit_len_for(domain_size: usize) -> u8 {
        assert!(domain_size > 0, "domain cannot be empty");
        let mut len = usize::BITS - domain_size.leading_zeros();
        if len % 2 == 1 {
            len += 1; // balance
        }
        len.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{FeistelNetwork, ROUNDS};

    #[test]
    fn bijective() {
        let keys = [0xAB; ROUNDS];
        let network = FeistelNetwork::with_keys(12, keys);

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
        let keys = [0xCD; ROUNDS];
        let network = FeistelNetwork::with_keys(8, keys);

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
}
