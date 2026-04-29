use crate::genarena::{GenArena, GenArenaKey};

pub struct Layout {
    nodes: GenArena<LayoutNode>,
    head: GenArenaKey,
}

impl Layout {
    pub fn new(width: u32, height: u32) -> Self {
        let root = LayoutNode {
            relatives: LayoutNodeRelatives::Root {
                child: None
            },
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

        Self {
            nodes,
            head,
        }
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

    pub fn add_node(&mut self, parent: GenArenaKey, ty: LayoutNodeType, position: Position) -> GenArenaKey {
        let relatives = match ty {
            LayoutNodeType::Leaf => LayoutNodeRelatives::Leaf { parent },
            LayoutNodeType::Container => LayoutNodeRelatives::Container { parent, child: None },
            LayoutNodeType::HorizontalGroup { spacing } => LayoutNodeRelatives::HorizontalGroup { parent, children: Vec::new(), spacing },
            LayoutNodeType::VerticalGroup { spacing } => LayoutNodeRelatives::VerticalGroup { parent, children: Vec::new(), spacing },
        };

        let node = LayoutNode {
            relatives,
            position,
            coords: None,
        };

        let node_key = self.nodes.push(node);

        let parent_node = self.nodes.get_mut(parent).unwrap();
        
        match &mut parent_node.relatives {
            LayoutNodeRelatives::Leaf { parent: _ } => panic!(),
            LayoutNodeRelatives::Root { child } => {
                *child = Some(node_key);
            },
            LayoutNodeRelatives::Container { parent: _, child } => {
                *child = Some(node_key);
            },
            LayoutNodeRelatives::HorizontalGroup { parent: _, children, spacing: _ } => {
                children.push(node_key);
            },
            LayoutNodeRelatives::VerticalGroup { parent: _, children, spacing: _ } => {
                children.push(node_key);
            },
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

        self.calculate_node_coords(root_node_key, bounds);
    }

    fn calculate_node_coords(&mut self, node_key: GenArenaKey, bounds: Coords) -> Coords {
        let node = self.nodes.get(node_key).unwrap();

        // calculate our base bounds for this branch (this is our maximum allowed size)
        let mut coords = bounds;

        coords.x += node.position.h_padding as i32;
        coords.w -= node.position.h_padding * 2;

        coords.y += node.position.v_padding as i32;
        coords.h -= node.position.v_padding * 2;

        // TODO: calculate coords here!!!
        let is_exact_width = match node.position.w {
            Size::Fill => {
                true
            },
            Size::Min => {
                false
            },
            Size::Exact(size) => {
                coords.w = size;

                true
            },
        };

        let is_exact_height = match node.position.h {
            Size::Fill => {
                true
            },
            Size::Min => {
                false
            },
            Size::Exact(size) => {
                coords.h = size;

                true
            },
        };

        match node.position.x {
            HorizontalOffset::AlignLeft(position) => {
                coords.x += position as i32;
            },
            HorizontalOffset::Center => {
                coords.x += ((bounds.w / 2) - (coords.w / 2)) as i32;
            },
            HorizontalOffset::AlignRight(position) => {
                coords.x += (bounds.w - (coords.w + position)) as i32;
            },
        }

        match node.position.y {
            VerticalOffset::AlignTop(position) => {
                coords.y += position as i32;
            },
            VerticalOffset::Center => {
                coords.y += ((bounds.h / 2) - (coords.h / 2)) as i32;
            },
            VerticalOffset::AlignBottom(position) => {
                coords.y += (bounds.h - (coords.h + position)) as i32;
            },
        }

        // TODO: only minimize size if not Size::Exact!
        match &node.relatives {
            LayoutNodeRelatives::Root { child } => {
                if let Some(child_key) = child {
                    // ignore child coords, we don't need minimum size
                    self.calculate_node_coords(*child_key, coords);
                }
            },
            LayoutNodeRelatives::Leaf { parent: _ } => {},
            LayoutNodeRelatives::Container { parent: _, child } => {
                if let Some(child_key) = child {
                    let child_coords = self.calculate_node_coords(*child_key, coords);

                    if !is_exact_width {
                        coords.w = ((child_coords.x - coords.x) + child_coords.w as i32) as u32;
                    }

                    if !is_exact_height {
                        coords.h = ((child_coords.y - coords.y) + child_coords.h as i32) as u32;
                    }
                }
            },
            LayoutNodeRelatives::HorizontalGroup { parent: _, children, spacing } => {
                let mut bounds = coords;
                let mut max_h = 0;
                let space = *spacing;

                // borrow checker gets mad if we use children directly
                // ...so clone it. really bad stuff
                for child_key in &children.clone() {
                    let child_coords = self.calculate_node_coords(*child_key, bounds);

                    let shift_x = (child_coords.x - coords.x) + (child_coords.w + space) as i32;
                    bounds.x += shift_x;
                    bounds.w = bounds.w.saturating_sub(shift_x as u32);

                    let height = (child_coords.y - coords.y) + child_coords.h as i32;

                    max_h = max_h.max(height as u32);
                }

                // now that we know the size of our children, adjust coords
                if !is_exact_width {
                    coords.w = bounds.x as u32;
                }

                if !is_exact_height {
                    coords.h = max_h;
                }
            },
            LayoutNodeRelatives::VerticalGroup { parent: _, children, spacing } => {
                let mut bounds = coords;
                let mut max_w = 0;
                let space = *spacing;

                // borrow checker gets mad if we use children directly
                // ...so clone it. really bad stuff
                for child_key in &children.clone() {
                    let child_coords = self.calculate_node_coords(*child_key, bounds);

                    let shift_y = (child_coords.y - coords.y) + (child_coords.h + space) as i32;
                    bounds.y += shift_y;
                    bounds.h = bounds.h.saturating_sub(shift_y as u32);

                    let width = (child_coords.x - coords.x) + child_coords.w as i32;

                    max_w = max_w.max(width as u32);
                }

                // now that we know the size of our children, adjust coords
                if !is_exact_height {
                    coords.h = bounds.y as u32;
                }

                if !is_exact_width {
                    coords.w = max_w;
                }
            },
        }

        // get a mutable ref to this node to actually set its coords
        // we couldn't hold it before because we needed to calculate children
        let node_mut = self.nodes.get_mut(node_key).unwrap();
        node_mut.coords = Some(coords);

        println!("Coords: {}, {}, {}, {}", coords.x, coords.y, coords.w, coords.h);

        coords
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
    HorizontalGroup { spacing: u32 },
    VerticalGroup { spacing: u32 },
}

enum LayoutNodeRelatives {
    Root {
        child: Option<GenArenaKey>,
    },
    Leaf {
        parent: GenArenaKey,
    },
    Container {
        parent: GenArenaKey,
        child: Option<GenArenaKey>,
    },
    HorizontalGroup {
        parent: GenArenaKey,
        children: Vec<GenArenaKey>,
        spacing: u32,
    },
    VerticalGroup {
        parent: GenArenaKey,
        children: Vec<GenArenaKey>,
        spacing: u32,
    },
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub enum HorizontalOffset {
    AlignLeft(u32),
    Center,
    AlignRight(u32),
}

#[derive(Clone, Copy)]
pub enum VerticalOffset {
    AlignTop(u32),
    Center,
    AlignBottom(u32),
}

#[derive(Clone, Copy)]
pub enum Size {
    Fill,
    Min,
    Exact(u32),
}