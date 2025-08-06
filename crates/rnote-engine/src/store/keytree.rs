// Imports
use super::StrokeKey;
use p2d::bounding_volume::Aabb;
use rstar::primitives::GeomWithData;
use std::collections::HashMap;

/// The rtree object that holds the bounds and [StrokeKey].
type KeyTreeObject = GeomWithData<rstar::primitives::Rectangle<[f64; 2]>, StrokeKey>;

#[derive(Debug, Default)]
/// A Rtree with [StrokeKey]'s as associated data.
///
/// Used for faster spatial queries.
pub(super) struct KeyTree(rstar::RTree<KeyTreeObject, rstar::DefaultParams>);

impl KeyTree {
    /// Insert a new tree object with the given [StrokeKey] and bounds.
    pub(crate) fn insert_with_key(&mut self, key: StrokeKey, bounds: Aabb) {
        self.0.insert(new_keytree_object(key, bounds));
    }

    /// Removes the [KeyTreeObject] for the given key.
    pub(crate) fn remove_with_key(&mut self, key: StrokeKey) -> Option<(StrokeKey, Aabb)> {
        let object_to_remove = self.0.iter().find(|&object| object.data == key)?.to_owned();

        self.0
            .remove(&object_to_remove)
            .and_then(|key_object| Some(keytree_to_store(key_object)))
    }

    /// Update the Tree with new bounds for the given key.
    ///
    /// Has to be called when the geometry of the stroke has changed.
    pub(crate) fn update_with_key(&mut self, key: StrokeKey, new_bounds: Aabb) {
        self.remove_with_key(key);
        self.insert_with_key(key, new_bounds);
    }

    /// Return the keys that intersect with the given bounds.
    pub(crate) fn keys_intersecting_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        self.0
            .locate_in_envelope_intersecting(&rstar::AABB::from_corners(
                [bounds.mins[0], bounds.mins[1]],
                [bounds.maxs[0], bounds.maxs[1]],
            ))
            .map(|object| object.data)
            .collect()
    }

    /// Return the keys that intersect with the given bounds.
    pub(crate) fn keys_intersecting_bounds_hashset(&self, bounds: Aabb) -> HashMap<StrokeKey, ()> {
        self.0
            .locate_in_envelope_intersecting(&rstar::AABB::from_corners(
                [bounds.mins[0], bounds.mins[1]],
                [bounds.maxs[0], bounds.maxs[1]],
            ))
            .map(|object| (object.data, ()))
            .collect()
    }

    /// Return the keys that are completely contained in the given bounds.
    pub(crate) fn keys_in_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        self.0
            .locate_in_envelope(&rstar::AABB::from_corners(
                [bounds.mins[0], bounds.mins[1]],
                [bounds.maxs[0], bounds.maxs[1]],
            ))
            .map(|object| object.data)
            .collect()
    }

    /// Rebuild the entire rtree from the given Vec of (key, bounds).
    pub(crate) fn rebuild_from_vec(&mut self, strokes: Vec<(StrokeKey, Aabb)>) {
        let objects = strokes
            .into_iter()
            .map(|(key, bounds)| new_keytree_object(key, bounds))
            .collect();

        self.0 = rstar::RTree::bulk_load(objects);
    }

    ///  Clear the entire tree.
    pub(crate) fn clear(&mut self) {
        *self = Self::default()
    }

    pub fn get_bounds(&self) -> Aabb {
        let aabb_enveloppe = self.0.root().envelope();
        Aabb::new(
            na::point![aabb_enveloppe.lower()[0], aabb_enveloppe.lower()[1]],
            na::point![aabb_enveloppe.upper()[0], aabb_enveloppe.upper()[1]],
        )
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.size() == 0
    }
}

fn new_keytree_object(key: StrokeKey, bounds: Aabb) -> KeyTreeObject {
    KeyTreeObject::new(
        rstar::primitives::Rectangle::from_corners(
            [bounds.mins[0], bounds.mins[1]],
            [bounds.maxs[0], bounds.maxs[1]],
        ),
        key,
    )
}

fn keytree_to_store(key_object: KeyTreeObject) -> (StrokeKey, Aabb) {
    (
        key_object.data,
        Aabb::new(
            na::point![key_object.geom().lower()[0], key_object.geom().lower()[1]],
            na::point![key_object.geom().upper()[0], key_object.geom().upper()[1]],
        ),
    )
}
