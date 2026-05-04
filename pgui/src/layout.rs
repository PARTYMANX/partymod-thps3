use crate::genarena::{GenArena, GenArenaKey};

pub struct Layout {
    nodes: GenArena<LayoutNode>,
    head: GenArenaKey,
}

impl Layout {
    pub fn new(width: u32, height: u32) -> Self {
        let root = LayoutNode {
            relatives: LayoutNodeRelatives::Root { child: None },
            position: Position {
                w: Size::Exact(width),
                h: Size::Exact(height),
                x: HorizontalOffset::AlignLeft(0),
                y: VerticalOffset::AlignTop(0),
                h_padding: 0,
                v_padding: 0,
            },
            coords: None,
        };

        let mut nodes = GenArena::new();
        let head = nodes.push(root);

        Self { nodes, head }
    }

    pub fn get_root_node(&self) -> GenArenaKey {
        self.head
    }

    pub fn get_node_coords(&self, key: GenArenaKey) -> Option<Coords> {
        match self.nodes.get(key) {
            Some(v) => v.coords,
            None => None,
        }
    }

    pub fn add_node(
        &mut self,
        parent: GenArenaKey,
        ty: LayoutNodeType,
        position: Position,
    ) -> GenArenaKey {
        let relatives = match ty {
            LayoutNodeType::Leaf => LayoutNodeRelatives::Leaf { _parent: parent },
            LayoutNodeType::Container => LayoutNodeRelatives::Container {
                _parent: parent,
                child: None,
            },
            LayoutNodeType::MultiContainer => LayoutNodeRelatives::MultiContainer {
                _parent: parent,
                children: Vec::new(),
            },
            LayoutNodeType::HorizontalGroup { spacing } => LayoutNodeRelatives::HorizontalGroup {
                _parent: parent,
                children: Vec::new(),
                spacing,
            },
            LayoutNodeType::VerticalGroup { spacing } => LayoutNodeRelatives::VerticalGroup {
                _parent: parent,
                children: Vec::new(),
                spacing,
            },
        };

        let node = LayoutNode {
            relatives,
            position,
            coords: None,
        };

        let node_key = self.nodes.push(node);

        let parent_node = self.nodes.get_mut(parent).unwrap();

        match &mut parent_node.relatives {
            LayoutNodeRelatives::Leaf { _parent: _ } => panic!(),
            LayoutNodeRelatives::Root { child } => {
                *child = Some(node_key);
            }
            LayoutNodeRelatives::Container { _parent: _, child } => {
                *child = Some(node_key);
            }
            LayoutNodeRelatives::MultiContainer {
                _parent: _,
                children,
            } => {
                children.push(node_key);
            }
            LayoutNodeRelatives::HorizontalGroup {
                _parent: _,
                children,
                spacing: _,
            } => {
                children.push(node_key);
            }
            LayoutNodeRelatives::VerticalGroup {
                _parent: _,
                children,
                spacing: _,
            } => {
                children.push(node_key);
            }
        }

        node_key
    }

    // TODO: two pass coords: first calculate node offsets and sizes, then calculate absolute positions
    // TODO: figure out how to get proportional sizing to coexist with absolute sizing (i.e. two proportionally sized groups above one absolute sized item)
    pub fn calculate_coords(&mut self) {
        let root_node_key = self.get_root_node();

        let root_node = self.nodes.get(root_node_key).unwrap();
        let bounds = Coords {
            w: if let Size::Exact(w) = root_node.position.w {
                w
            } else {
                panic!();
            },
            h: if let Size::Exact(h) = root_node.position.w {
                h
            } else {
                panic!();
            },
            x: 0,
            y: 0,
        };

        // calculate relative coords for each node
        self.calculate_relative_node_coords(root_node_key, bounds, true);

        // finally, calculate final coords for each node
        self.calculate_node_positions(root_node_key, bounds);
    }

    // calculates the size and relative positioning of a node given its parent's bounds. returns its own bounds
    fn calculate_relative_node_coords(
        &mut self,
        node_key: GenArenaKey,
        parent_bounds: Coords,
        greedy: bool,
    ) -> Coords {
        let node = self.nodes.get(node_key).unwrap();

        // amount to subtract from bounds
        let offset_x = match node.position.x {
            HorizontalOffset::AlignLeft(v) => v,
            HorizontalOffset::Center => 0,
            HorizontalOffset::AlignRight(v) => v,
        };
        // size of bounds
        let (width, is_exact_width) = match node.position.w {
            Size::Fill => {
                if greedy {
                    (
                        parent_bounds.w - offset_x - (node.position.h_padding * 2),
                        true,
                    )
                } else {
                    (0, false)
                }
            }
            Size::Min => (0, false),
            Size::Exact(v) => (v, true),
        };

        // amount to subtract from bounds
        let offset_y = match node.position.y {
            VerticalOffset::AlignTop(v) => v,
            VerticalOffset::Center => 0,
            VerticalOffset::AlignBottom(v) => v,
        };
        // size of bounds
        let (height, is_exact_height) = match node.position.h {
            Size::Fill => {
                if greedy {
                    (
                        parent_bounds.h - offset_y - (node.position.v_padding * 2),
                        true,
                    )
                } else {
                    (0, false)
                }
            }
            Size::Min => (0, false),
            Size::Exact(v) => (v, true),
        };

        let mut coords = Coords {
            w: width,
            h: height,
            x: parent_bounds.x + node.position.h_padding as i32,
            y: parent_bounds.y + node.position.v_padding as i32,
        };

        // space available to this node
        let available_bounds = Coords {
            w: if is_exact_width {
                coords.w
            } else {
                (parent_bounds.w - offset_x) - (node.position.h_padding * 2)
            },
            h: if is_exact_height {
                coords.h
            } else {
                (parent_bounds.h - offset_y) - (node.position.v_padding * 2)
            },
            x: 0,
            y: 0,
        };

        // space taken by this node
        let mut self_bounds = Coords {
            w: offset_x + (node.position.h_padding * 2),
            h: offset_y + (node.position.v_padding * 2),
            x: 0,
            y: 0,
        };

        match &node.relatives {
            LayoutNodeRelatives::Root { child } => {
                if let Some(child_key) = child {
                    // ignore child coords, we don't need minimum size
                    self.calculate_relative_node_coords(*child_key, available_bounds, true);
                }
            }
            LayoutNodeRelatives::Leaf { _parent: _ } => {}
            LayoutNodeRelatives::Container { _parent: _, child } => {
                if let Some(child_key) = child {
                    let child_coords =
                        self.calculate_relative_node_coords(*child_key, available_bounds, true);

                    if !is_exact_width {
                        coords.w += child_coords.w;
                    }
                    if !is_exact_height {
                        coords.h += child_coords.h;
                    }
                }
            }
            LayoutNodeRelatives::MultiContainer {
                _parent: _,
                children,
            } => {
                // borrow checker gets mad if we use children directly
                // ...so clone it. really bad stuff
                for child_key in &children.clone() {
                    let mut component_coords = coords;

                    let child_coords =
                        self.calculate_relative_node_coords(*child_key, available_bounds, true);

                    if !is_exact_width {
                        component_coords.w += child_coords.w;

                        coords.w = coords.w.max(component_coords.w);
                    }
                    if !is_exact_height {
                        component_coords.h += child_coords.h;

                        coords.h = coords.h.max(component_coords.h);
                    }
                }
            }
            LayoutNodeRelatives::HorizontalGroup {
                _parent: _,
                children,
                spacing,
            } => {
                let mut children_width = 0;
                let mut max_h = 0;
                let mut remaining_bounds = available_bounds;
                let space = *spacing;

                // borrow checker gets mad if we use children directly
                // ...so clone it. really bad stuff
                let children_list = children.clone();

                for (i, child_key) in children_list.iter().enumerate() {
                    let is_last = i == children_list.len() - 1;

                    let child_bounds =
                        self.calculate_relative_node_coords(*child_key, remaining_bounds, is_last);

                    if !is_last {
                        children_width += child_bounds.w + space;
                    } else {
                        children_width += child_bounds.w;
                    }

                    remaining_bounds.w = available_bounds.w.saturating_sub(children_width);
                    remaining_bounds.x = children_width as i32;

                    max_h = max_h.max(child_bounds.h as u32);
                }

                if !is_exact_width {
                    coords.w += children_width;
                }
                if !is_exact_height {
                    coords.h += max_h;
                }
            }
            LayoutNodeRelatives::VerticalGroup {
                _parent: _,
                children,
                spacing,
            } => {
                let mut children_height = 0;
                let mut max_w = 0;
                let mut remaining_bounds = available_bounds;
                let space = *spacing;

                // borrow checker gets mad if we use children directly
                // ...so clone it. really bad stuff
                let children_list = children.clone();

                for (i, child_key) in children_list.iter().enumerate() {
                    let is_last = i == children_list.len() - 1;

                    let child_bounds =
                        self.calculate_relative_node_coords(*child_key, remaining_bounds, is_last);

                    if !is_last {
                        children_height += child_bounds.h + space;
                    } else {
                        children_height += child_bounds.h;
                    }

                    remaining_bounds.h = available_bounds.h.saturating_sub(children_height);
                    remaining_bounds.y = children_height as i32;

                    max_w = max_w.max(child_bounds.w as u32);
                }

                if !is_exact_width {
                    coords.w += max_w;
                }
                if !is_exact_height {
                    coords.h += children_height;
                }
            }
        }

        let node_mut = self.nodes.get_mut(node_key).unwrap();

        // now that we know this node's size, we can align it
        match node_mut.position.x {
            HorizontalOffset::AlignLeft(v) => coords.x += v as i32,
            HorizontalOffset::Center => {
                if greedy {
                    let node_width = coords.w + (node_mut.position.h_padding * 2);
                    coords.x += ((parent_bounds.w.saturating_sub(node_width)) / 2) as i32;
                }
            }
            HorizontalOffset::AlignRight(v) => {
                if greedy {
                    let node_width = coords.w + (node_mut.position.h_padding * 2);
                    coords.x +=
                        ((parent_bounds.w.saturating_sub(node_width)).saturating_sub(v)) as i32;
                }
            }
        }

        match node_mut.position.y {
            VerticalOffset::AlignTop(v) => coords.y += v as i32,
            VerticalOffset::Center => {
                if greedy {
                    let node_height = coords.h + (node_mut.position.v_padding * 2);
                    coords.y += ((parent_bounds.h.saturating_sub(node_height)) / 2) as i32;
                }
            }
            VerticalOffset::AlignBottom(v) => {
                if greedy {
                    let node_height = coords.h + (node_mut.position.v_padding * 2);
                    coords.y +=
                        ((parent_bounds.h.saturating_sub(node_height)).saturating_sub(v)) as i32;
                }
            }
        }

        node_mut.coords = Some(coords);

        self_bounds.w += coords.w;
        self_bounds.h += coords.h;

        self_bounds
    }

    fn calculate_node_positions(&mut self, node_key: GenArenaKey, parent_coords: Coords) {
        let node = self.nodes.get(node_key).unwrap();

        let mut coords = node.coords.unwrap();

        coords.x += parent_coords.x;
        coords.y += parent_coords.y;

        match &node.relatives {
            LayoutNodeRelatives::Root { child } => {
                if let Some(child_key) = child {
                    // ignore child coords, we don't need minimum size
                    self.calculate_node_positions(*child_key, coords);
                }
            }
            LayoutNodeRelatives::Leaf { _parent: _ } => {}
            LayoutNodeRelatives::Container { _parent: _, child } => {
                if let Some(child_key) = child {
                    self.calculate_node_positions(*child_key, coords);
                }
            }
            LayoutNodeRelatives::MultiContainer {
                _parent: _,
                children,
            } => {
                // borrow checker gets mad if we use children directly
                // ...so clone it. really bad stuff
                for child_key in &children.clone() {
                    self.calculate_node_positions(*child_key, coords);
                }
            }
            LayoutNodeRelatives::HorizontalGroup {
                _parent: _,
                children,
                spacing: _,
            } => {
                // borrow checker gets mad if we use children directly
                // ...so clone it. really bad stuff
                for child_key in &children.clone() {
                    self.calculate_node_positions(*child_key, coords);
                }
            }
            LayoutNodeRelatives::VerticalGroup {
                _parent: _,
                children,
                spacing: _,
            } => {
                // borrow checker gets mad if we use children directly
                // ...so clone it. really bad stuff
                for child_key in &children.clone() {
                    self.calculate_node_positions(*child_key, coords);
                }
            }
        }

        let node_mut = self.nodes.get_mut(node_key).unwrap();
        node_mut.coords = Some(coords);
    }

    pub fn set_node_position(&mut self, node_key: GenArenaKey, position: Position) {
        if let Some(node) = self.nodes.get_mut(node_key) {
            node.position = position;
        }
    }
}

struct LayoutNode {
    relatives: LayoutNodeRelatives,
    position: Position,
    coords: Option<Coords>,
}

pub enum LayoutNodeType {
    Leaf,
    Container,
    MultiContainer,
    HorizontalGroup { spacing: u32 },
    VerticalGroup { spacing: u32 },
}

enum LayoutNodeRelatives {
    Root {
        child: Option<GenArenaKey>,
    },
    Leaf {
        _parent: GenArenaKey,
    },
    Container {
        _parent: GenArenaKey,
        child: Option<GenArenaKey>,
    },
    MultiContainer {
        _parent: GenArenaKey,
        children: Vec<GenArenaKey>,
    },
    HorizontalGroup {
        _parent: GenArenaKey,
        children: Vec<GenArenaKey>,
        spacing: u32,
    },
    VerticalGroup {
        _parent: GenArenaKey,
        children: Vec<GenArenaKey>,
        spacing: u32,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub w: Size,
    pub h: Size,

    pub x: HorizontalOffset,
    pub y: VerticalOffset,

    pub h_padding: u32,
    pub v_padding: u32,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            w: Size::Min,
            h: Size::Min,
            x: HorizontalOffset::AlignLeft(0),
            y: VerticalOffset::AlignTop(0),
            h_padding: 0,
            v_padding: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Coords {
    pub w: u32,
    pub h: u32,

    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HorizontalOffset {
    AlignLeft(u32),
    Center,
    AlignRight(u32),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VerticalOffset {
    AlignTop(u32),
    Center,
    AlignBottom(u32),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Fill,
    Min,
    Exact(u32),
}
