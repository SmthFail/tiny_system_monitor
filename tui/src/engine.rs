use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::app_error::AppError;
use crate::buffer::Buffer;
use crate::widget::{Rect, Widget};

// ---------------- keys & layout direction ----------------

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Key(pub String);

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Horizontal,
    Vertical,
}

// один «живой» виджет: общее владение + interior mutability
type WidgetCell = Rc<RefCell<Box<dyn Widget>>>;

// виртуальное дерево кадра
#[derive(Clone)]
pub enum VNode {
    Container {
        key: Option<Key>,
        dir: Direction,
        children: Vec<VNode>,
    },
    // лист — уже созданный экземпляр виджета
    LeafInstance {
        key: Key,
        cell: WidgetCell,
    },
}

fn short_type<T>() -> String {
    std::any::type_name::<T>()
        .rsplit("::")
        .next()
        .unwrap_or("Widget")
        .to_string()
}

// уникальные стабильные ключи для инстансов
static NODE_ID: AtomicU64 = AtomicU64::new(1);

// Позволяет писать: app.add_child(my_widget);
pub trait IntoNode {
    fn into_node(self) -> VNode;
}

impl<W: Widget + 'static> IntoNode for W {
    fn into_node(self) -> VNode {
        let id = NODE_ID.fetch_add(1, Ordering::Relaxed);
        let key = Key(format!("{}#{}", short_type::<W>(), id));
        let cell: WidgetCell = Rc::new(RefCell::new(Box::new(self)));
        VNode::LeafInstance { key, cell }
    }
}

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

// ---------------- live tree ----------------

pub enum ElementKind {
    Container { dir: Direction }, // только layout
    Leaf(WidgetCell),             // ссылка на живой виджет
}

pub struct Element {
    pub key: Option<Key>,
    pub kind: ElementKind,
    pub rect: Rect,
    pub children: Vec<Element>,
}

impl Element {
    fn new_container(key: Option<Key>, dir: Direction) -> Self {
        Self {
            key,
            kind: ElementKind::Container { dir },
            rect: Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            children: vec![],
        }
    }
    fn new_leaf(key: Key, cell: WidgetCell) -> Self {
        Self {
            key: Some(key),
            kind: ElementKind::Leaf(cell),
            rect: Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            children: vec![],
        }
    }
}

// ---------------- mount / layout / update / paint ----------------

pub fn mount(v: &VNode) -> Element {
    match v {
        VNode::Container { key, dir, children } => {
            let mut e = Element::new_container(key.clone(), *dir);
            e.children = children.iter().map(mount).collect();
            e
        }
        VNode::LeafInstance { key, cell } => Element::new_leaf(key.clone(), cell.clone()),
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

pub fn update_tree(e: &mut Element) -> Result<(), AppError> {
    match &mut e.kind {
        ElementKind::Leaf(cell) => cell.borrow_mut().update(),
        ElementKind::Container { .. } => {
            for ch in &mut e.children {
                update_tree(ch)?;
            }
            Ok(())
        }
    }
}

pub fn paint(buf: &mut Buffer, e: &mut Element) -> Result<(), AppError> {
    match &mut e.kind {
        ElementKind::Leaf(cell) => cell.borrow_mut().render(buf, e.rect),
        ElementKind::Container { .. } => {
            for ch in &mut e.children {
                paint(buf, ch)?;
            }
            Ok(())
        }
    }
}

// ---------------- reconciliation (по ключам) ----------------

fn node_key(v: &VNode) -> Option<&Key> {
    match v {
        VNode::Container { key, .. } => key.as_ref(),
        VNode::LeafInstance { key, .. } => Some(key),
    }
}

pub fn reconcile(prev: &mut Element, next: &VNode) {
    match (&mut prev.kind, next) {
        // контейнер -> контейнер
        (ElementKind::Container { dir: prev_dir }, VNode::Container { key, dir, children }) => {
            prev.key = key.clone();
            *prev_dir = *dir;

            let mut old_children = std::mem::take(&mut prev.children);
            let mut keyed_old: HashMap<String, Element> = HashMap::new();
            let mut unkeyed_old: Vec<Element> = Vec::new();

            for c in old_children.drain(..) {
                if let Some(k) = &c.key {
                    keyed_old.insert(k.0.clone(), c);
                } else {
                    unkeyed_old.push(c);
                }
            }

            let mut new_children: Vec<Element> = Vec::with_capacity(children.len());

            for n in children {
                let k_opt = node_key(n).cloned();
                let mut ch = if let Some(k) = &k_opt {
                    if let Some(old) = keyed_old.remove(&k.0) {
                        old
                    } else {
                        mount(n)
                    }
                } else if !unkeyed_old.is_empty() {
                    unkeyed_old.remove(0)
                } else {
                    mount(n)
                };

                reconcile(&mut ch, n);
                new_children.push(ch);
            }

            // оставшиеся old считаем удалёнными
            prev.children = new_children;
        }

        // контейнер -> лист: полная замена
        (ElementKind::Container { .. }, VNode::LeafInstance { key, cell }) => {
            prev.key = Some(key.clone());
            prev.kind = ElementKind::Leaf(cell.clone());
            prev.children.clear();
        }

        // лист -> контейнер: полная замена
        (ElementKind::Leaf(_), VNode::Container { key, dir, children }) => {
            prev.key = key.clone();
            prev.kind = ElementKind::Container { dir: *dir };
            prev.children = children.iter().map(mount).collect();
        }

        // лист -> лист
        (ElementKind::Leaf(old_cell), VNode::LeafInstance { key, cell: new_cell }) => {
            match &prev.key {
                Some(k) if k == key => {
                    // ключ совпал — сохраняем состояние (ничего не делаем)
                }
                _ => {
                    prev.key = Some(key.clone());
                    *old_cell = new_cell.clone();
                }
            }
        }
    }
}
