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

/// (De)Serialize a [`Vector2`] rounded to 3 decimal places
pub mod glam_vector2_dp3 {
    use p2d::math::Vector2;
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    /// Serialize a [`Vector2`] rounded to 3 decimal places
    pub fn serialize<S: Serializer>(v: &Vector2, s: S) -> Result<S::Ok, S::Error> {
        const SCALE: f64 = (10_u32.pow(3)) as f64;
        let r: p2d::math::Vec2 = (v * SCALE).round() / SCALE;
        r.serialize(s)
    }

    /// Deserialize a [`Vector2`]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vector2, D::Error> {
        Vector2::deserialize(d)
    }
}

/// (De)Serialize a [`p2d::glamx::DAffine2`] rounded to 3 decimal places
pub mod glam_daffine2_f64_dp3 {
    use p2d::glamx::DAffine2;
    use p2d::math::Vector2;
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    /// Serialize a [`DAffine2`] rounded to 3 decimal places
    pub fn serialize<S: Serializer>(v: &DAffine2, serializer: S) -> Result<S::Ok, S::Error> {
        const SCALE: Vector2 = Vector2::splat(10_u32.pow(3) as f64);
        let x_axis = (v.x_axis * SCALE).round() / SCALE;
        let y_axis = (v.y_axis * SCALE).round() / SCALE;
        let z_axis = (v.z_axis * SCALE).round() / SCALE;
        DAffine2::from_cols(x_axis, y_axis, z_axis).serialize(serializer)
    }

    /// Deserialize a [`DAffine2`]
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DAffine2, D::Error> {
        DAffine2::deserialize(deserializer)
    }
}

/// (De)Serialize a [Cuboid] rounded to 3 decimal places
pub mod p2d_cuboid_dp3 {
    use p2d::shape::Cuboid;
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    /// Serialize a [Cuboid] rounded to 3 decimal places
    pub fn serialize<S: Serializer>(v: &Cuboid, s: S) -> Result<S::Ok, S::Error> {
        const D: f64 = (10_u32.pow(3)) as f64;
        let a = (v.half_extents * D).round();
        Cuboid::new(a / D).serialize(s)
    }

    /// Deserialize a [Cuboid]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Cuboid, D::Error> {
        Cuboid::deserialize(d)
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
