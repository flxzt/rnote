use rand::{Rng, SeedableRng};

use super::roughoptions::Options;

pub(super) fn merge<T>(this: Option<T>, other: Option<T>) -> Option<T> {
    match this {
        Some(t) => Some(t),
        None => other,
    }
}

/// Random u64 with full number range
pub(super) fn random_u64_full(seed: Option<u64>) -> u64 {
    let mut rng = if let Some(seed) = seed {
        rand_pcg::Pcg64::seed_from_u64(seed)
    } else {
        rand_pcg::Pcg64::from_entropy()
    };
    rng.gen()
}

/// Random i32 with full number range
pub(super) fn random_i32_full(seed: Option<u64>) -> i32 {
    let mut rng = if let Some(seed) = seed {
        rand_pcg::Pcg64::seed_from_u64(seed)
    } else {
        rand_pcg::Pcg64::from_entropy()
    };
    rng.gen()
}

/// Random f64 between 0.0 and 1.0
pub(super) fn random_f64_0to1(seed: Option<u64>) -> f64 {
    let mut rng = if let Some(seed) = seed {
        rand_pcg::Pcg64::seed_from_u64(seed)
    } else {
        rand_pcg::Pcg64::from_entropy()
    };
    rng.gen_range(0.0..1.0)
}

/// returning random f64 from 0.0 to 1.0 created from seed and advancing it
pub(super) fn rand_f64_0to1_next(options: &mut Options) -> f64 {
    if let Some(ref mut seed) = options.seed {
        *seed = random_u64_full(Some(*seed));
    };
    random_f64_0to1(options.seed)
}
