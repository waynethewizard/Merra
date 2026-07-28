//! Independently derived deterministic random streams.

use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;

const RNG_SCHEME: &[u8] = b"merra-rng-v1\0";

/// Stable built-in random domains.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RngDomain {
    /// Initial population demographics.
    Population,
    /// Birth and fertility decisions.
    Birth,
    /// Mortality decisions.
    Mortality,
    /// Weather generation.
    Weather,
    /// Political decisions.
    Politics,
    /// Cosmetic name generation.
    Names,
}

impl RngDomain {
    /// Returns the stable domain label used in seed derivation.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Population => "population",
            Self::Birth => "birth",
            Self::Mortality => "mortality",
            Self::Weather => "weather",
            Self::Politics => "politics",
            Self::Names => "names",
        }
    }
}

/// Derives a 256-bit stream seed without relying on platform hashers.
#[must_use]
pub fn seed_for_domain(root_seed: u64, domain: RngDomain) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(RNG_SCHEME);
    hasher.update(&root_seed.to_le_bytes());
    hasher.update(b"\0");
    hasher.update(domain.label().as_bytes());
    *hasher.finalize().as_bytes()
}

/// Creates the deterministic stream for a domain.
#[must_use]
pub fn rng_for_domain(root_seed: u64, domain: RngDomain) -> ChaCha12Rng {
    ChaCha12Rng::from_seed(seed_for_domain(root_seed, domain))
}

#[cfg(test)]
mod tests {
    use rand::RngExt;

    use super::{RngDomain, rng_for_domain, seed_for_domain};

    #[test]
    fn domain_seeds_are_stable_and_isolated() {
        assert_eq!(
            seed_for_domain(42, RngDomain::Mortality),
            seed_for_domain(42, RngDomain::Mortality)
        );
        assert_ne!(
            seed_for_domain(42, RngDomain::Mortality),
            seed_for_domain(42, RngDomain::Names)
        );
    }

    #[test]
    fn streams_repeat() {
        let mut first = rng_for_domain(42, RngDomain::Weather);
        let mut second = rng_for_domain(42, RngDomain::Weather);

        assert_eq!(first.random::<u64>(), second.random::<u64>());
    }
}
