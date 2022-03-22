use std::ops::Range;

use gtk4::glib;
use rand_distr::Distribution;
use serde::{Deserialize, Serialize};

/// The distribution for the spread of dots across the width of the textured stroke
#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "TexturedDotsDistribution")]
pub enum TexturedDotsDistribution {
    #[enum_value(name = "Uniform", nick = "uniform")]
    Uniform = 0,
    #[enum_value(name = "Normal", nick = "normal")]
    Normal,
    #[enum_value(name = "Exponential", nick = "exponential")]
    Exponential,
    #[enum_value(name = "ReverseExponential", nick = "reverse-exponential")]
    ReverseExponential,
}

impl Default for TexturedDotsDistribution {
    fn default() -> Self {
        Self::Normal
    }
}

impl TexturedDotsDistribution {
    /// Samples a value for the given range, symmetrical to the mid of the range. For distributions that are open ended, samples are clipped to the range
    pub fn sample_for_range_symmetrical_clipped<G: rand::Rng + ?Sized>(
        &self,
        rng: &mut G,
        range: Range<f64>,
    ) -> f64 {
        let sample = match self {
            Self::Uniform => rand_distr::Uniform::from(range.clone()).sample(rng),
            Self::Normal => {
                // setting the mean to the mid of the range
                let mean = (range.end + range.start) / 2.0;
                // the standard deviation
                let std_dev = ((range.end - range.start) / 2.0) / 3.0;

                rand_distr::Normal::new(mean, std_dev).unwrap().sample(rng)
            }
            Self::Exponential => {
                let mid = (range.end + range.start) / 2.0;
                let width = (range.end - range.start) / 4.0;
                // The lambda
                let lambda = 1.0;

                let sign: f64 = if rand_distr::Standard.sample(rng) {
                    1.0
                } else {
                    -1.0
                };

                mid + sign * width * rand_distr::Exp::new(lambda).unwrap().sample(rng)
            }
            Self::ReverseExponential => {
                let width = (range.end - range.start) / 4.0;
                // The lambda
                let lambda = 1.0;

                let positive: bool = rand_distr::Standard.sample(rng);
                let sign = if positive { 1.0 } else { -1.0 };
                let offset = if positive { range.start } else { range.end };

                offset + (sign * width * rand_distr::Exp::new(lambda).unwrap().sample(rng))
            }
        };

        if !range.contains(&sample) {
            // Do a uniform distribution as fallback if sample is out of range
            rand_distr::Uniform::from(range.clone()).sample(rng)
        } else {
            sample
        }
    }
}
