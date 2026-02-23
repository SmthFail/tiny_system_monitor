use super::engine::{Element, VNode, mount, reconcile, layout_equal_split, update_tree, paint, Key, Direction};
use super::buffer::Buffer;
use super::widget::Rect;
use super::app_error::AppError;

/// Represents a layer in the layer stack
/// Each layer has its own VNode tree and renders on top of previous layers
pub struct Layer {
    /// Unique identifier for this layer
    pub key: Key,
    /// The virtual node tree for this layer
    pub vnode: VNode,
    /// The live element tree (maintained between frames)
    pub element: Option<Element>,
    /// Whether this layer is visible
    pub visible: bool,
    /// Whether this layer blocks input to layers below
    pub modal: bool,
}

impl Layer {
    pub fn new(key: &str, vnode: VNode) -> Self {
        Self {
            key: Key(key.to_string()),
            vnode,
            element: None,
            visible: true,
            modal: false,
        }
    }

    pub fn modal(mut self) -> Self {
        self.modal = true;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Update the vnode and reconcile with existing element
    pub fn update_vnode(&mut self, new_vnode: VNode) {
        if let Some(elem) = &mut self.element {
            reconcile(elem, &new_vnode);
        }
        self.vnode = new_vnode;
    }

    /// Initialize the element tree if not already done
    pub fn ensure_element(&mut self) {
        if self.element.is_none() {
            self.element = Some(mount(&self.vnode));
        }
    }

    /// Get mutable reference to the element
    pub fn element_mut(&mut self) -> Option<&mut Element> {
        self.element.as_mut()
    }
}

/// Layer stack - manages multiple layers of UI
pub struct LayerStack {
    layers: Vec<Layer>,
}

impl LayerStack {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add a new layer on top of the stack
    pub fn push(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// Remove a layer by key
    pub fn remove(&mut self, key: &str) -> Option<Layer> {
        if let Some(pos) = self.layers.iter().position(|l| l.key.0 == key) {
            Some(self.layers.remove(pos))
        } else {
            None
        }
    }

    /// Get mutable reference to a layer by key
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.key.0 == key)
    }

    /// Get reference to a layer by key
    pub fn get(&self, key: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.key.0 == key)
    }

    /// Check if a layer exists
    pub fn has_layer(&self, key: &str) -> bool {
        self.layers.iter().any(|l| l.key.0 == key)
    }

    /// Get the topmost visible layer
    pub fn top_visible(&mut self) -> Option<&mut Layer> {
        self.layers.iter_mut().rev().find(|l| l.visible)
    }

    /// Get the topmost modal layer (blocks input)
    pub fn top_modal(&mut self) -> Option<&mut Layer> {
        self.layers.iter_mut().rev().find(|l| l.visible && l.modal)
    }

    /// Render all visible layers in order (bottom to top)
    pub fn render_all(&mut self, buffer: &mut Buffer, area: Rect) -> Result<(), AppError> {
        for layer in &mut self.layers {
            if !layer.visible {
                continue;
            }

            layer.ensure_element();
            if let Some(elem) = layer.element_mut() {
                // Each layer gets the full area (they overlay)
                layout_equal_split(elem, area);

                if let Err(e) = update_tree(elem) {
                    // Continue rendering other layers even if one fails
                    eprintln!("Layer {} update error: {}", layer.key.0, e.message);
                }

                paint(buffer, elem)?;
            }
        }
        Ok(())
    }

    /// Update all visible layers
    pub fn update_all(&mut self) -> Result<(), AppError> {
        for layer in &mut self.layers {
            if !layer.visible {
                continue;
            }

            if let Some(elem) = layer.element_mut() {
                update_tree(elem)?;
            }
        }
        Ok(())
    }

    /// Hide a layer
    pub fn hide(&mut self, key: &str) {
        if let Some(layer) = self.get_mut(key) {
            layer.visible = false;
        }
    }

    /// Show a layer
    pub fn show(&mut self, key: &str) {
        if let Some(layer) = self.get_mut(key) {
            layer.visible = true;
        }
    }

    /// Toggle layer visibility
    pub fn toggle(&mut self, key: &str) {
        if let Some(layer) = self.get_mut(key) {
            layer.visible = !layer.visible;
        }
    }

    /// Clear all layers
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Get number of layers
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Check if stack is empty
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Iterate over layers
    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.layers.iter()
    }
}

impl Default for LayerStack {
    fn default() -> Self {
        Self::new()
    }
}
