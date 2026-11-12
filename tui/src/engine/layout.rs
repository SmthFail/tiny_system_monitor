use crate::widget::Rect;

use super::tree::{Direction, Element, ElementKind, IntoNode, Key, VNode};

// удобные «сборщики» контейнеров (если нужны)
#[derive(Default)]
pub struct Row {
    key: Option<Key>,
    children: Vec<VNode>,
}
impl Row {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn key<K: Into<String>>(mut self, k: K) -> Self {
        self.key = Some(Key(k.into()));
        self
    }
    pub fn child<N: IntoNode>(mut self, n: N) -> Self {
        self.children.push(n.into_node());
        self
    }
}
impl IntoNode for Row {
    fn into_node(self) -> VNode {
        VNode::Container {
            key: self.key,
            dir: Direction::Horizontal,
            children: self.children,
        }
    }
}

#[derive(Default)]
pub struct Column {
    key: Option<Key>,
    children: Vec<VNode>,
}
impl Column {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn key<K: Into<String>>(mut self, k: K) -> Self {
        self.key = Some(Key(k.into()));
        self
    }
    pub fn child<N: IntoNode>(mut self, n: N) -> Self {
        self.children.push(n.into_node());
        self
    }
}
impl IntoNode for Column {
    fn into_node(self) -> VNode {
        VNode::Container {
            key: self.key,
            dir: Direction::Vertical,
            children: self.children,
        }
    }
}

pub fn layout_equal_split(elem: &mut Element, area: Rect) {
    elem.rect = area;
    match &mut elem.kind {
        ElementKind::Leaf(_) => {}
        ElementKind::Container { dir } => {
            let n = elem.children.len().max(1) as u16;
            match dir {
                Direction::Horizontal => {
                    let w_each = area.width / n;
                    let mut x = area.x;
                    for ch in &mut elem.children {
                        layout_equal_split(
                            ch,
                            Rect {
                                x,
                                y: area.y,
                                width: w_each,
                                height: area.height,
                            },
                        );
                        x = x.saturating_add(w_each);
                    }
                }
                Direction::Vertical => {
                    let h_each = area.height / n;
                    let mut y = area.y;
                    for ch in &mut elem.children {
                        layout_equal_split(
                            ch,
                            Rect {
                                x: area.x,
                                y,
                                width: area.width,
                                height: h_each,
                            },
                        );
                        y = y.saturating_add(h_each);
                    }
                }
            }
        }
    }
}
