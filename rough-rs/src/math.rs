use rand::{Rng, SeedableRng};

/// Random i32 with full number range
#[allow(dead_code)]
pub(crate) fn random_i32_full(seed: Option<u64>) -> i32 {
    let mut rng = if let Some(seed) = seed {
        rand::rngs::StdRng::seed_from_u64(seed)
    } else {
        rand::rngs::StdRng::from_entropy()
    };
    rng.gen()
}

/// Random f64 between 0.0 and 1.0
pub(crate) fn random_f64_0to1(seed: Option<u64>) -> f64 {
    let mut rng = if let Some(seed) = seed {
        rand::rngs::StdRng::seed_from_u64(seed)
    } else {
        rand::rngs::StdRng::from_entropy()
    };
    rng.gen_range(0.0..1.0)
}
