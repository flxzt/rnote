// TODO: a macro would be perfect here to reduce code repetition

/// (De)Serialize a [f64] rounded to 3 decimal places
pub mod f64_dp3 {
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    /// Serialize a [f64] rounded to 3 decimal places
    pub fn serialize<S: Serializer>(v: &f64, s: S) -> Result<S::Ok, S::Error> {
        const D: f64 = (10_u32.pow(3)) as f64;
        ((v * D).round() / D).serialize(s)
    }

    /// Deserialize a [f64]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
        f64::deserialize(d)
    }
}

/// (De)Serialize a [`na::Vector2<f64>`] rounded to 3 decimal places
pub mod na_vector2_f64_dp3 {
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    /// Serialize a [`na::Vector2<f64>`] rounded to 3 decimal places
    pub fn serialize<S: Serializer>(v: &na::Vector2<f64>, s: S) -> Result<S::Ok, S::Error> {
        const D: f64 = (10_u32.pow(3)) as f64;
        let mut a = v * D;
        a = na::vector![a[0].round(), a[1].round()];
        (a / D).serialize(s)
    }

    /// Deserialize a [`na::Vector2<f64>`]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<na::Vector2<f64>, D::Error> {
        na::Vector2::<f64>::deserialize(d)
    }
}

/// (De)Serialize a [`na::Affine2<f64>`] rounded to 3 decimal places
pub mod na_affine2_f64_dp3 {
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    /// Serialize a [`na::Vector2<f64>`] rounded to 3 decimal places
    pub fn serialize<S: Serializer>(v: &na::Affine2<f64>, s: S) -> Result<S::Ok, S::Error> {
        const D: f64 = (10_u32.pow(3)) as f64;
        let mut a = v.into_inner() * D;
        a = na::matrix![
        a[(0, 0)].round(), a[(0, 1)].round(), a[(0, 2)].round();
        a[(1, 0)].round(), a[(1, 1)].round(), a[(1, 2)].round();
        a[(2, 0)].round(), a[(2, 1)].round(), a[(2, 2)].round();
        ];
        na::Affine2::<f64>::from_matrix_unchecked(a / D).serialize(s)
    }

    /// Deserialize a [`na::Vector2<f64>`]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<na::Affine2<f64>, D::Error> {
        na::Affine2::<f64>::deserialize(d)
    }
}

/// (De)Serialize a [p2d::shape::Cuboid] rounded to 3 decimal places
pub mod p2d_cuboid_dp3 {
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    /// Serialize a [p2d::shape::Cuboid] rounded to 3 decimal places
    pub fn serialize<S: Serializer>(v: &p2d::shape::Cuboid, s: S) -> Result<S::Ok, S::Error> {
        const D: f64 = (10_u32.pow(3)) as f64;
        let mut a = v.half_extents * D;
        a = na::vector![a[0].round(), a[1].round()];
        p2d::shape::Cuboid::new(a / D).serialize(s)
    }

    /// Deserialize a [p2d::shape::Cuboid]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<p2d::shape::Cuboid, D::Error> {
        p2d::shape::Cuboid::deserialize(d)
    }
}

/// (De)Serialize bytes with base64 encoding
pub mod sliceu8_base64 {
    use base64::Engine;
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    /// Serialize bytes as base64 encoded
    pub fn serialize<S: Serializer>(v: impl AsRef<[u8]>, s: S) -> Result<S::Ok, S::Error> {
        String::serialize(&base64::engine::general_purpose::STANDARD.encode(v), s)
    }

    /// Deserialize base64 encoded bytes
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        base64::engine::general_purpose::STANDARD
            .decode(String::deserialize(d)?.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}
