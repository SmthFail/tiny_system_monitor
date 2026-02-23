use super::engine::{Element, VNode, mount, reconcile, layout_equal_split, update_tree, paint, Key};

/// A route in the navigation stack
/// Can represent a full screen or an overlay (dialog)
pub struct Route {
    /// Unique identifier for debugging/reconciliation
    pub key: Key,
    /// Virtual nodes for this route
    pub vnodes: Vec<VNode>,
    /// Live element tree (maintained between frames)
    pub element: Option<Element>,
    /// If true, this route covers all routes below it
    pub fullscreen: bool,
    /// If true, this route blocks input to routes below
    pub modal: bool,
}

impl Route {
    /// Create a new route from vnodes
    pub fn new(vnodes: Vec<VNode>) -> Self {
        Self {
            key: Key(format!("route_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())),
            vnodes,
            element: None,
            fullscreen: false,
            modal: false,
        }
    }

    /// Mark this route as fullscreen (covers routes below)
    pub fn fullscreen(mut self) -> Self {
        self.fullscreen = true;
        self
    }

    /// Mark this route as modal (blocks input to routes below)
    pub fn modal(mut self) -> Self {
        self.modal = true;
        self
    }

    /// Initialize the element tree if not already done
    pub fn ensure_element(&mut self) {
        if self.element.is_none() {
            // Create root container for this route's vnodes
            let root_vnode = super::engine::VNode::Container {
                key: Some(self.key.clone()),
                dir: super::engine::Direction::Vertical,
                children: self.vnodes.clone(),
            };
            self.element = Some(mount(&root_vnode));
        }
    }

    /// Update vnodes and reconcile with existing element
    pub fn update_vnodes(&mut self, new_vnodes: Vec<VNode>) {
        let root_vnode = super::engine::VNode::Container {
            key: Some(self.key.clone()),
            dir: super::engine::Direction::Vertical,
            children: new_vnodes.clone(),
        };

        if let Some(elem) = &mut self.element {
            reconcile(elem, &root_vnode);
        }

        self.vnodes = new_vnodes;
    }

    /// Get mutable reference to the element
    pub fn element_mut(&mut self) -> Option<&mut Element> {
        self.element.as_mut()
    }

    /// Layout and render this route
    pub fn render(&mut self, buffer: &mut super::buffer::Buffer, area: super::widget::Rect) -> Result<(), super::app_error::AppError> {
        self.ensure_element();

        if let Some(elem) = self.element_mut() {
            layout_equal_split(elem, area);
            update_tree(elem)?;
            paint(buffer, elem)?;
        }

        Ok(())
    }
}
