use std::hash::Hasher;
use wyhash::WyHash;

type Keys<const ROUNDS: usize> = [u64; ROUNDS];

// The Luby-Rackoff theorem shows that 4 rounds are enough to resist all
// adaptive chosen plaintext and chosen ciphertext attacks, for sufficiently
// large block sizes. However, we support arbitrarily small domains.
pub(crate) struct FeistelNetwork<const ROUNDS: usize = 8> {
    keys: Keys<ROUNDS>,
    upper_shift: u8,
    lower_mask: u64,
}

impl<const ROUNDS: usize> FeistelNetwork<ROUNDS> {
    /// Creates a Feistel network to permute a domain of size at least `domain_len`.
    ///
    /// The round keys are initially set to zero. The network should be initialized
    /// by mutating `FeistelNetwork::keys_mut`.
    ///
    /// # Panics
    ///
    /// Panics if the domain size is zero.
    pub fn for_domain(domain_len: usize) -> Self {
        assert!(domain_len > 0, "domain cannot be empty");
        let mut len = usize::BITS - domain_len.leading_zeros(); // log2(domain_len) + 1
        if len % 2 == 1 {
            len += 1; // balance
        }
        // SAFETY: the maximum size 2^64 - 1 gives a 64-bit network.
        Self::new(len.try_into().unwrap())
    }

    /// Creates a Feistel network to permute a domain of size `2^bit_len`.
    ///
    /// The round keys are initially set to zero. The network should be initialized
    /// by mutating `FeistelNetwork::keys_mut`.
    ///
    /// # Panics
    ///
    /// Panics if `bit_len` is odd, equal to zero, or greater than `u64::BITS`.
    pub fn new(bit_len: u8) -> Self {
        assert!(bit_len > 0, "bit_len should be positive");
        assert!(
            bit_len <= u64::BITS as u8,
            "bit_len (is {}) should be < {}",
            bit_len,
            u64::BITS
        );
        assert_eq!(bit_len % 2, 0, "bit_len (is {}) should be even", bit_len);

        let upper_shift = bit_len / 2;
        Self {
            keys: [0; ROUNDS],
            upper_shift,
            lower_mask: (1u64 << upper_shift) - 1,
        }
    }

    /// Creates a Feistel network to permute a domain of size `2^bit_len`.
    ///
    /// # Panics
    ///
    /// Panics if `bit_len` is odd, equal to zero, or greater than `u64::BITS`.
    pub fn with_keys(bit_len: u8, keys: Keys<ROUNDS>) -> Self {
        let mut network = Self::new(bit_len);
        network.keys_mut().copy_from_slice(&keys);
        network
    }

    pub fn permute(&self, input: u64) -> u64 {
        // todo: assert input < max
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

    /// Returns a mutable reference to the array of round keys used by
    /// the Feistel network.
    ///
    /// This should be used to initialize and rotate the keys.
    pub fn keys_mut(&mut self) -> &mut Keys<ROUNDS> {
        &mut self.keys
    }
}

#[cfg(test)]
mod tests {
    use super::FeistelNetwork;

    #[test]
    fn small_domain() {
        let keys = [0x12, 8];
        let network = FeistelNetwork::with_keys(4, keys);
        assert_eq!(network.permute(1), network.permute(1));
        assert_ne!(network.permute(2), network.permute(3));
    }

    #[test]
    fn for_domain_rounds_up() {
        let mut network = FeistelNetwork::<1>::for_domain(347); // n = 10
        network.keys_mut()[0] = 0x34; // initialize

        let mut seen = [false; 1 << 10];
        for value in 0..1 << 10 {
            let result = network.permute(value) as usize;
            seen[result] = true;
        }
        assert!(seen.iter().all(|&x| x));
    }

    #[test]
    fn bijective() {
        let keys = [0xAB; 8];
        let network = FeistelNetwork::with_keys(12, keys);

        let mut seen = [false; 1 << 12];
        for value in 0..1 << 12 {
            let result = network.permute(value) as usize;
            assert!(!seen[result]);
            seen[result] = true;
        }
        assert!(seen.iter().all(|&x| x));
    }

    #[test]
    fn idempotent() {
        let keys = [0xCD; 8];
        let network = FeistelNetwork::with_keys(8, keys);
        for value in 0..1 << 8 {
            let expected = network.permute(value);
            for _ in 0..4 {
                assert_eq!(network.permute(value), expected);
            }
        }
    }

    #[test]
    #[should_panic]
    fn len_must_be_positive() {
        FeistelNetwork::<8>::new(0);
    }

    #[test]
    #[should_panic]
    fn must_be_balanced() {
        FeistelNetwork::<8>::new(3);
    }
}
