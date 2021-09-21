use rand::Rng;

/// Random i32 with full number range
pub fn random_i32_full() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

/// Random f64 between 0.0 and 1.0
pub fn random_f64_0to1() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..1.0)
}
