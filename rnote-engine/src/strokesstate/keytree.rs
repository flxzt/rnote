use p2d::bounding_volume::AABB;
use rstar::primitives::GeomWithData;

use super::StrokeKey;

type KeyTreeObject = GeomWithData<rstar::primitives::Rectangle<[f64; 2]>, StrokeKey>;

fn new_keytree_object(key: StrokeKey, bounds: AABB) -> KeyTreeObject {
    KeyTreeObject::new(
        rstar::primitives::Rectangle::from_corners(
            [bounds.mins[0], bounds.mins[1]],
            [bounds.maxs[0], bounds.maxs[1]],
        ),
        key,
    )
}

#[derive(Debug, Default)]
pub(super) struct KeyTree(rstar::RTree<KeyTreeObject, rstar::DefaultParams>);

impl KeyTree {
    pub fn insert_with_key(&mut self, key: StrokeKey, bounds: AABB) {
        self.0.insert(new_keytree_object(key, bounds));
    }

    /// Slow removal, as has to iterate through the entire tree in no particular order
    pub fn remove_with_key(&mut self, key: StrokeKey) -> Option<KeyTreeObject> {
        let object_to_remove = self.0.iter().find(|&object| object.data == key)?.to_owned();

        self.0.remove(&object_to_remove)
    }

    pub fn update_with_key(&mut self, key: StrokeKey, new_bounds: AABB) {
        self.remove_with_key(key);
        self.insert_with_key(key, new_bounds);
    }

    pub fn keys_intersecting_bounds(&self, bounds: AABB) -> Vec<StrokeKey> {
        self.0
            .locate_in_envelope_intersecting(&rstar::AABB::from_corners(
                [bounds.mins[0], bounds.mins[1]],
                [bounds.maxs[0], bounds.maxs[1]],
            ))
            .map(|object| object.data)
            .collect()
    }

    pub fn reload_with_vec(&mut self, strokes: Vec<(StrokeKey, AABB)>) {
        let objects = strokes.into_iter().map(|(key, bounds)| {
            new_keytree_object(key, bounds)
        }).collect();

        self.0 = rstar::RTree::bulk_load(objects);
    }

    pub fn clear(&mut self) {
        *self = Self::default()
    }
}
