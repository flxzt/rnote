// Imports
use super::{StrokeKey, StrokeStore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type DocumentLayerId = u32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename = "document_layer")]
pub struct DocumentLayer {
    #[serde(rename = "id")]
    pub id: DocumentLayerId,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "visible")]
    pub visible: bool,
    #[serde(rename = "locked")]
    pub locked: bool,
}

impl Default for DocumentLayer {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::from("Layer 1"),
            visible: true,
            locked: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "layer_component")]
pub struct LayerComponent {
    #[serde(rename = "layer_id")]
    pub layer_id: DocumentLayerId,
}

impl Default for LayerComponent {
    fn default() -> Self {
        Self { layer_id: 0 }
    }
}

impl StrokeStore {
    pub(crate) fn layers(&self) -> &[DocumentLayer] {
        self.layers.as_slice()
    }

    pub(crate) fn active_layer_id(&self) -> DocumentLayerId {
        self.active_layer_id
    }

    pub(crate) fn set_active_layer(&mut self, layer_id: DocumentLayerId) -> bool {
        if self
            .layers
            .iter()
            .any(|layer| layer.id == layer_id && layer.visible && !layer.locked)
        {
            self.active_layer_id = layer_id;
            true
        } else {
            false
        }
    }

    pub(crate) fn add_layer(&mut self, name: Option<String>) -> DocumentLayerId {
        self.layer_counter += 1;
        let id = self.layer_counter;
        let layer = DocumentLayer {
            id,
            name: name.unwrap_or_else(|| format!("Layer {}", self.layers.len() + 1)),
            visible: true,
            locked: false,
        };
        Arc::make_mut(&mut self.layers).push(layer);
        self.active_layer_id = id;
        id
    }

    pub(crate) fn delete_layer(&mut self, layer_id: DocumentLayerId) -> bool {
        if self.layers.len() <= 1 || !self.layers.iter().any(|layer| layer.id == layer_id) {
            return false;
        }

        let keys_to_remove = self
            .layer_components
            .iter()
            .filter_map(|(key, layer_comp)| {
                if layer_comp.layer_id == layer_id {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>();

        for key in keys_to_remove {
            self.remove_stroke(key);
        }

        Arc::make_mut(&mut self.layers).retain(|layer| layer.id != layer_id);
        self.ensure_active_layer_editable();

        true
    }

    pub(crate) fn rename_layer(&mut self, layer_id: DocumentLayerId, name: String) -> bool {
        if let Some(layer) = Arc::make_mut(&mut self.layers)
            .iter_mut()
            .find(|layer| layer.id == layer_id)
        {
            layer.name = name;
            true
        } else {
            false
        }
    }

    pub(crate) fn set_layer_visible(&mut self, layer_id: DocumentLayerId, visible: bool) -> bool {
        if !visible && !self.has_editable_layer_after(layer_id, Some(visible), None) {
            return false;
        }

        if let Some(layer) = Arc::make_mut(&mut self.layers)
            .iter_mut()
            .find(|layer| layer.id == layer_id)
        {
            layer.visible = visible;
            self.ensure_active_layer_editable();
            true
        } else {
            false
        }
    }

    pub(crate) fn set_layer_locked(&mut self, layer_id: DocumentLayerId, locked: bool) -> bool {
        if locked && !self.has_editable_layer_after(layer_id, None, Some(locked)) {
            return false;
        }

        if let Some(layer) = Arc::make_mut(&mut self.layers)
            .iter_mut()
            .find(|layer| layer.id == layer_id)
        {
            layer.locked = locked;
            self.ensure_active_layer_editable();
            true
        } else {
            false
        }
    }

    fn has_editable_layer_after(
        &self,
        layer_id: DocumentLayerId,
        visible_override: Option<bool>,
        locked_override: Option<bool>,
    ) -> bool {
        self.layers.iter().any(|layer| {
            let visible = if layer.id == layer_id {
                visible_override.unwrap_or(layer.visible)
            } else {
                layer.visible
            };
            let locked = if layer.id == layer_id {
                locked_override.unwrap_or(layer.locked)
            } else {
                layer.locked
            };
            visible && !locked
        })
    }

    fn ensure_active_layer_editable(&mut self) {
        if self
            .layers
            .iter()
            .any(|layer| layer.id == self.active_layer_id && layer.visible && !layer.locked)
        {
            return;
        }

        if let Some(layer) = self
            .layers
            .iter()
            .find(|layer| layer.visible && !layer.locked)
        {
            self.active_layer_id = layer.id;
        }
    }

    pub(crate) fn move_layer_up(&mut self, layer_id: DocumentLayerId) -> bool {
        self.move_layer(layer_id, true)
    }

    pub(crate) fn move_layer_down(&mut self, layer_id: DocumentLayerId) -> bool {
        self.move_layer(layer_id, false)
    }

    fn move_layer(&mut self, layer_id: DocumentLayerId, up: bool) -> bool {
        let Some(index) = self.layers.iter().position(|layer| layer.id == layer_id) else {
            return false;
        };
        let swap_index = if up {
            index.checked_add(1).filter(|&i| i < self.layers.len())
        } else {
            index.checked_sub(1)
        };

        if let Some(swap_index) = swap_index {
            Arc::make_mut(&mut self.layers).swap(index, swap_index);
            true
        } else {
            false
        }
    }

    pub(crate) fn stroke_document_layer_id(&self, key: StrokeKey) -> Option<DocumentLayerId> {
        self.layer_components.get(key).map(|comp| comp.layer_id)
    }

    #[allow(unused)]
    pub(crate) fn set_stroke_document_layer(
        &mut self,
        key: StrokeKey,
        layer_id: DocumentLayerId,
    ) -> bool {
        if !self.layers.iter().any(|layer| layer.id == layer_id) {
            return false;
        }
        if let Some(layer_comp) = Arc::make_mut(&mut self.layer_components)
            .get_mut(key)
            .map(Arc::make_mut)
        {
            layer_comp.layer_id = layer_id;
            true
        } else {
            false
        }
    }

    #[allow(unused)]
    pub(crate) fn stroke_layer_order(&self, key: StrokeKey) -> usize {
        self.stroke_document_layer_id(key)
            .and_then(|layer_id| self.layers.iter().position(|layer| layer.id == layer_id))
            .unwrap_or(0)
    }

    pub(crate) fn stroke_layer_visible(&self, key: StrokeKey) -> bool {
        self.stroke_document_layer_id(key)
            .and_then(|layer_id| self.layers.iter().find(|layer| layer.id == layer_id))
            .map(|layer| layer.visible)
            .unwrap_or(true)
    }

    pub(crate) fn stroke_layer_editable(&self, key: StrokeKey) -> bool {
        self.stroke_document_layer_id(key)
            .and_then(|layer_id| self.layers.iter().find(|layer| layer.id == layer_id))
            .map(|layer| layer.visible && !layer.locked)
            .unwrap_or(true)
    }

    pub(crate) fn stroke_in_active_layer(&self, key: StrokeKey) -> bool {
        self.stroke_document_layer_id(key)
            .map(|layer_id| layer_id == self.active_layer_id)
            .unwrap_or(true)
    }

    pub(crate) fn stroke_editable_in_active_layer(&self, key: StrokeKey) -> bool {
        self.stroke_in_active_layer(key) && self.stroke_layer_editable(key)
    }

    #[allow(unused)]
    pub(super) fn rebuild_layer_components_slotmap(&mut self) {
        self.layer_components = Arc::new(slotmap::SecondaryMap::new());
        self.stroke_components.keys().for_each(|key| {
            Arc::make_mut(&mut self.layer_components)
                .insert(key, Arc::new(LayerComponent::default()));
        });
    }

    pub(super) fn normalize_layers(&mut self) {
        if self.layers.is_empty() {
            self.layers = Arc::new(vec![DocumentLayer::default()]);
        }

        if !self
            .layers
            .iter()
            .any(|layer| layer.id == self.active_layer_id)
        {
            self.active_layer_id = self
                .layers
                .first()
                .map(|layer| layer.id)
                .unwrap_or_default();
        }

        self.layer_counter = self.layer_counter.max(
            self.layers
                .iter()
                .map(|layer| layer.id)
                .max()
                .unwrap_or_default(),
        );

        let default_layer_id = self
            .layers
            .first()
            .map(|layer| layer.id)
            .unwrap_or_default();
        for key in self.stroke_components.keys() {
            if !self.layer_components.contains_key(key) {
                Arc::make_mut(&mut self.layer_components).insert(
                    key,
                    Arc::new(LayerComponent {
                        layer_id: default_layer_id,
                    }),
                );
            }
        }

        let valid_layer_ids = self
            .layers
            .iter()
            .map(|layer| layer.id)
            .collect::<std::collections::HashSet<_>>();
        for (_, layer_comp) in Arc::make_mut(&mut self.layer_components).iter_mut() {
            if !valid_layer_ids.contains(&layer_comp.layer_id) {
                Arc::make_mut(layer_comp).layer_id = default_layer_id;
            }
        }
    }
}
