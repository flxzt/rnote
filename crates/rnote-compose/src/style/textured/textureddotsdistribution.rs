// Imports
use anyhow::Context;
use rand_distr::Distribution;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// The distribution for the spread of dots across the fill of a textured shape.
#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
pub enum TexturedDotsDistribution {
    /// Uniform distribution.
    Uniform = 0,
    /// Normal distribution.
    #[default]
    Normal,
    /// Exponential distribution distribution, from the outline increasing in probability symmetrical to the center.
    Exponential,
    /// Exponential distribution distribution, from the center increasing in probability symmetrical outwards to the outline.
    ReverseExponential,
}

impl TryFrom<u32> for TexturedDotsDistribution {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).with_context(|| {
            format!("TexturedDotsDistribution try_from::<u32>() for value {value} failed",)
        })
    }
}

impl TexturedDotsDistribution {
    /// Sample a value for the given range, symmetrical to the center of the range.
    ///
    /// For distributions that are open ended, samples are clipped to the range.
    pub fn sample_for_range_symmetrical_clipped<G: rand::Rng + ?Sized>(
        &self,
        rng: &mut G,
        range: Range<f64>,
    ) -> f64 {
        let sample = match self {
            Self::Uniform => rand_distr::Uniform::try_from(range.clone())
                .unwrap()
                .sample(rng),
            Self::Normal => {
                // the mean to the mid of the range
                let mean = (range.end + range.start) * 0.5;
                // the standard deviation
                let std_dev = ((range.end - range.start) * 0.5) / 3.0;

                rand_distr::Normal::new(mean, std_dev).unwrap().sample(rng)
            }
            Self::Exponential => {
                let mid = (range.end + range.start) * 0.5;
                let width = (range.end - range.start) / 4.0;
                // The lambda
                let lambda = 1.0;

                let sign: f64 = if rand_distr::StandardUniform.sample(rng) {
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

                let positive: bool = rand_distr::StandardUniform.sample(rng);
                let sign = if positive { 1.0 } else { -1.0 };
                let offset = if positive { range.start } else { range.end };

                offset + (sign * width * rand_distr::Exp::new(lambda).unwrap().sample(rng))
            }
        };

        if !range.contains(&sample) {
            // Do a uniform distribution as fallback if sample is out of range
            rand_distr::Uniform::try_from(range).unwrap().sample(rng)
        } else {
            sample
        }
    }
}
