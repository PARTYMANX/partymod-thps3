use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, COLOR_WINDOW, EndPaint, FillRect, HBRUSH, InvalidateRect, PAINTSTRUCT,
        },
        UI::{
            HiDpi::GetDpiForWindow,
            WindowsAndMessaging::{
                CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, GetClientRect, GetWindowRect, MoveWindow, PostQuitMessage, WINDOW_EX_STYLE, WM_COMMAND, WM_DESTROY, WM_DPICHANGED, WM_NOTIFY, WM_PAINT, WS_CAPTION, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE
            },
        },
    },
    core::{HSTRING, PCWSTR},
};

use crate::{
    button::ButtonState, checkbox::CheckboxState, genarena::{GenArena, GenArenaKey}, groupbox::GroupboxState, layout::Layout, tabs::TabsState, text::TextState, win32::{button::Button, checkbox::Checkbox, font::Fonts, groupbox::Groupbox, tabs::Tabs, text::Text}
};

pub enum Component<T> {
    Button(Button<T>),
    Checkbox(Checkbox<T>),
    Text(Text<T>),
    Groupbox(Groupbox<T>),
    Tabs(Tabs<T>),
}

pub struct Window<T> {
    hwnd: HWND,

    scale: f32,
    width: u32,
    height: u32,
    _title: HSTRING,

    post_update: Option<fn(&mut T)>,

    components: Vec<Component<T>>,
    component_tree: GenArena<ComponentTreeNode>,
    component_nodes: Vec<GenArenaKey>,
    layout: Layout,

    fonts: Fonts,
}

impl<T> Window<T> {
    pub fn new(
        window_class: PCWSTR,
        component: crate::component::Component<T>,
        width: u32,
        height: u32,
        title: String,
        post_update: Option<fn(&mut T)>,
    ) -> Self {
        let title = HSTRING::from(title);

        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class,
                &title,
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width as i32,
                height as i32,
                None,
                None,
                None,
                None,
            )
            .unwrap()
        };

        let scale = 1.0;

        let fonts = Fonts::new();

        // resize window to actually desired size
        unsafe {
            let mut client_rect = RECT::default();
            let _ = GetClientRect(window, &mut client_rect);

            let mut window_rect = RECT::default();
            let _ = GetWindowRect(window, &mut window_rect);

            let deco_width = (window_rect.right - window_rect.left) - client_rect.right;
            let deco_height = (window_rect.bottom - window_rect.top) - client_rect.bottom;

            let _ = MoveWindow(
                window,
                window_rect.left,
                window_rect.top,
                width as i32 + deco_width,
                height as i32 + deco_height,
                true,
            );
        }

        let mut native_components = Vec::new();
        let mut layout = Layout::new(width, height);

        let root_node = layout.get_root_node();
        let mut component_tree = GenArena::new();
        let mut component_nodes = Vec::new();
        Self::create_components(
            window,
            component,
            &mut native_components,
            &mut layout,
            root_node,
            &mut component_tree,
            None,
            &mut component_nodes,
            &fonts,
        );

        layout.calculate_coords();

        for native_component in &mut native_components {
            match native_component {
                Component::Button(button) => button.update(&mut layout, &fonts, 1.0),
                Component::Checkbox(checkbox) => checkbox.update(&mut layout, &fonts, 1.0),
                Component::Text(text) => text.update(&mut layout, &fonts, 1.0),
                Component::Groupbox(groupbox) => groupbox.update(&mut layout, &fonts, 1.0),
                Component::Tabs(tabs) => tabs.update(&mut layout, &fonts, 1.0),
            }
        }

        let mut result = Self {
            hwnd: window,

            scale,
            width,
            height,
            _title: title,

            post_update,

            components: native_components,
            component_tree,
            component_nodes,
            layout,

            fonts,
        };

        result.rescale(None);

        result
    }

    fn create_components(
        window: HWND,
        component: crate::component::Component<T>,
        native_components: &mut Vec<Component<T>>,
        layout: &mut Layout,
        layout_parent: GenArenaKey,
        component_tree: &mut GenArena<ComponentTreeNode>,
        component_parent: Option<GenArenaKey>,
        component_nodes: &mut Vec<GenArenaKey>,
        fonts: &Fonts,
    ) {
        match component {
            crate::component::Component::Button(button) => {
                let initial_state = ButtonState {
                    label: button.label,
                    enabled: button.enabled,
                    position: button.position,
                };

                let id = (native_components.len() + 1) as u16;

                let button = Component::Button(Button::new(
                    window,
                    id,
                    initial_state,
                    button.on_press,
                    button.state_hook,
                    layout_parent,
                    layout,
                    fonts,
                ));

                let component_node = component_tree.push(ComponentTreeNode {
                    component_id: Some(id as usize),
                    ty: ComponentTreeNodeType::Leaf,
                });

                if let Some(parent_key) = component_parent {
                    let p = component_tree.get_mut(parent_key).unwrap();

                    match &mut p.ty {
                        ComponentTreeNodeType::Node { children } => children.push(component_node),
                        ComponentTreeNodeType::Tabs { children: _ } => unreachable!(),
                        ComponentTreeNodeType::Leaf => unreachable!(),
                    }
                }

                native_components.push(button);
                component_nodes.push(component_node);
            }
            crate::component::Component::Checkbox(checkbox) => {
                let initial_state = CheckboxState {
                    label: checkbox.label,
                    checked: checkbox.checked,
                    enabled: checkbox.enabled,
                    position: checkbox.position,
                };

                let id = (native_components.len() + 1) as u16;

                let checkbox = Component::Checkbox(Checkbox::new(
                    window,
                    id,
                    initial_state,
                    checkbox.on_toggle,
                    checkbox.state_hook,
                    layout_parent,
                    layout,
                    fonts,
                ));

                let component_node = component_tree.push(ComponentTreeNode {
                    component_id: Some(id as usize),
                    ty: ComponentTreeNodeType::Leaf,
                });

                if let Some(parent_key) = component_parent {
                    let p = component_tree.get_mut(parent_key).unwrap();

                    match &mut p.ty {
                        ComponentTreeNodeType::Node { children } => children.push(component_node),
                        ComponentTreeNodeType::Tabs { children: _ } => unreachable!(),
                        ComponentTreeNodeType::Leaf => unreachable!(),
                    }
                }

                native_components.push(checkbox);
                component_nodes.push(component_node);
            }
            crate::component::Component::Text(text) => {
                let initial_state = TextState {
                    text: text.text,
                    enabled: text.enabled,
                    position: text.position,
                };

                let id = (native_components.len() + 1) as u16;

                let text = Component::Text(Text::new(
                    window,
                    id,
                    initial_state,
                    text.state_hook,
                    layout_parent,
                    layout,
                    fonts,
                ));

                let component_node = component_tree.push(ComponentTreeNode {
                    component_id: Some(id as usize),
                    ty: ComponentTreeNodeType::Leaf,
                });

                if let Some(parent_key) = component_parent {
                    let p = component_tree.get_mut(parent_key).unwrap();

                    match &mut p.ty {
                        ComponentTreeNodeType::Node { children } => children.push(component_node),
                        ComponentTreeNodeType::Tabs { children: _ } => unreachable!(),
                        ComponentTreeNodeType::Leaf => unreachable!(),
                    }
                }

                native_components.push(text);
                component_nodes.push(component_node);
            }
            crate::component::Component::Groupbox(groupbox) => {
                let initial_state = GroupboxState {
                    label: groupbox.label,
                    enabled: groupbox.enabled,
                    position: groupbox.position,
                };

                let id = (native_components.len() + 1) as u16;

                let groupbox_native = Groupbox::new(
                    window,
                    id,
                    initial_state,
                    groupbox.state_hook,
                    layout_parent,
                    layout,
                    fonts,
                );

                let node = groupbox_native.get_layout_node();

                let component_node = component_tree.push(ComponentTreeNode {
                    component_id: Some(id as usize),
                    ty: ComponentTreeNodeType::Node {
                        children: Vec::new()
                    },
                });

                if let Some(parent_key) = component_parent {
                    let p = component_tree.get_mut(parent_key).unwrap();

                    match &mut p.ty {
                        ComponentTreeNodeType::Node { children } => children.push(component_node),
                        ComponentTreeNodeType::Tabs { children: _ } => unreachable!(),
                        ComponentTreeNodeType::Leaf => unreachable!(),
                    }
                }

                native_components.push(Component::Groupbox(groupbox_native));
                component_nodes.push(component_node);

                Self::create_components(
                    window,
                    *groupbox.child,
                    native_components,
                    layout,
                    node,
                    component_tree,
                    Some(component_node),
                    component_nodes,
                    fonts,
                );
            }
            crate::component::Component::Tabs(tabs) => {
                let initial_state = TabsState {
                    enabled: tabs.enabled,
                    position: tabs.position,
                };

                let id = (native_components.len() + 1) as u16;

                let tabs_native = Tabs::new(
                    window,
                    id,
                    &tabs.tabs,
                    initial_state,
                    tabs.state_hook,
                    layout_parent,
                    layout,
                    fonts,
                );

                let node = tabs_native.get_layout_node();
                let mut children = Vec::new();
                for tab in tabs.tabs {
                    children.push(tab.child);
                }

                let component_node = component_tree.push(ComponentTreeNode {
                    component_id: Some(id as usize),
                    ty: ComponentTreeNodeType::Tabs {
                        children: Vec::new()
                    },
                });

                native_components.push(Component::Tabs(tabs_native));
                component_nodes.push(component_node);

                for child in children {
                    let tab_component_node = component_tree.push(ComponentTreeNode {
                        component_id: None,
                        ty: ComponentTreeNodeType::Node {
                            children: Vec::new()
                        },
                    });

                    let tab_parent = component_tree.get_mut(component_node).unwrap();

                    match &mut tab_parent.ty {
                        ComponentTreeNodeType::Node { children: _ } => unreachable!(),
                        ComponentTreeNodeType::Tabs { children } => children.push(vec![tab_component_node]),
                        ComponentTreeNodeType::Leaf => unreachable!(),
                    }

                    Self::create_components(
                        window,
                        child,
                        native_components,
                        layout,
                        node,
                        component_tree,
                        Some(tab_component_node),
                        component_nodes,
                        fonts,
                    );
                }

                let tab_node = component_tree.get(component_node).unwrap();
                match &tab_node.ty {
                    ComponentTreeNodeType::Node { children: _ } => unreachable!(),
                    ComponentTreeNodeType::Tabs { children } => {
                        for tab_idx in 1..children.len() {
                            for child in &children[tab_idx] {
                                Self::hide_children(native_components, component_tree, *child, true);
                            }
                        }
                    },
                    ComponentTreeNodeType::Leaf => unreachable!(),
                }
            }
            crate::component::Component::Container(container) => {
                let node = layout.add_node(
                    layout_parent,
                    crate::layout::LayoutNodeType::Container,
                    container.position,
                );

                let component_node = component_tree.push(ComponentTreeNode {
                    component_id: None,
                    ty: ComponentTreeNodeType::Node {
                        children: Vec::new()
                    },
                });

                Self::create_components(
                    window,
                    *container.child,
                    native_components,
                    layout,
                    node,
                    component_tree,
                    Some(component_node),
                    component_nodes,
                    fonts,
                );
            }
            crate::component::Component::Horizontal(horizontal) => {
                let node = layout.add_node(
                    layout_parent,
                    crate::layout::LayoutNodeType::HorizontalGroup {
                        spacing: horizontal.spacing,
                    },
                    horizontal.position,
                );

                let component_node = component_tree.push(ComponentTreeNode {
                    component_id: None,
                    ty: ComponentTreeNodeType::Node {
                        children: Vec::new()
                    },
                });

                for child in horizontal.children {
                    Self::create_components(
                        window, 
                        child, 
                        native_components, 
                        layout, 
                        node, 
                        component_tree,
                        Some(component_node),
                        component_nodes,
                        fonts,
                    );
                }
            }
            crate::component::Component::Vertical(vertical) => {
                let node = layout.add_node(
                    layout_parent,
                    crate::layout::LayoutNodeType::VerticalGroup {
                        spacing: vertical.spacing,
                    },
                    vertical.position,
                );

                let component_node = component_tree.push(ComponentTreeNode {
                    component_id: None,
                    ty: ComponentTreeNodeType::Node {
                        children: Vec::new()
                    },
                });

                for child in vertical.children {
                    Self::create_components(
                        window, 
                        child, 
                        native_components, 
                        layout, 
                        node, 
                        component_tree,
                        Some(component_node),
                        component_nodes,
                        fonts,
                    );
                }
            }
        }
    }

    fn hide_children(
        native_components: &Vec<Component<T>>,
        component_tree: &GenArena<ComponentTreeNode>,
        node_key: GenArenaKey,
        hidden: bool
    ) {
        let node = component_tree.get(node_key).unwrap();
        
        if let Some(id) = node.component_id {
            match &native_components[id - 1] {
                Component::Button(button) => button.hide(hidden),
                Component::Checkbox(checkbox) => checkbox.hide(hidden),
                Component::Text(text) => text.hide(hidden),
                Component::Groupbox(groupbox) => groupbox.hide(hidden),
                Component::Tabs(tabs) => tabs.hide(hidden),
            }
        }
        
        match &node.ty {
            ComponentTreeNodeType::Leaf => {},
            ComponentTreeNodeType::Node { children } => {
                for child in children {
                    Self::hide_children(native_components, component_tree, *child, hidden);
                }
            },
            ComponentTreeNodeType::Tabs { children } => {
                // FIXME: when unhiding child tabs, this will unhide all elements
                for tab in children {
                    for child in tab {
                        Self::hide_children(native_components, component_tree, *child, hidden);
                    }
                }
            },
        }
    }

    fn rescale(&mut self, suggested_pos: Option<(i32, i32)>) {
        self.scale = unsafe {
            let dpi = GetDpiForWindow(self.hwnd);
            dpi as f32 / 96.0
        };

        unsafe {
            let mut client_rect = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut client_rect);

            let mut window_rect = RECT::default();
            let _ = GetWindowRect(self.hwnd, &mut window_rect);

            let deco_width = (window_rect.right - window_rect.left) - client_rect.right;
            let deco_height = (window_rect.bottom - window_rect.top) - client_rect.bottom;

            let width = (self.width as f32 * self.scale) as i32;
            let height = (self.height as f32 * self.scale) as i32;

            let (x, y) = match suggested_pos {
                Some(v) => v,
                None => (window_rect.left, window_rect.top),
            };

            let _ = MoveWindow(
                self.hwnd,
                x,
                y,
                width + deco_width,
                height + deco_height,
                true,
            );
        }

        // update font size
        self.fonts.update(self.scale);

        // update controls
        for component in &mut self.components {
            match component {
                Component::Button(button) => {
                    button.update(&mut self.layout, &self.fonts, self.scale)
                }
                Component::Checkbox(checkbox) => {
                    checkbox.update(&mut self.layout, &self.fonts, self.scale)
                }
                Component::Text(text) => {
                    text.update(&mut self.layout, &self.fonts, self.scale)
                }
                Component::Groupbox(groupbox) => {
                    groupbox.update(&mut self.layout, &self.fonts, self.scale)
                }
                Component::Tabs(tabs) => {
                    tabs.update(&mut self.layout, &self.fonts, self.scale)
                }
            }
        }

        // force repaint
        unsafe {
            let _ = InvalidateRect(Some(self.hwnd), None, true);
        }
    }

    pub fn run_state_hooks(&mut self, state: &T) {
        let mut updated = false;

        for c in &mut self.components {
            updated |= match c {
                Component::Button(button) => {
                    button.do_state_hook(state, &mut self.layout, &self.fonts)
                }
                Component::Checkbox(checkbox) => {
                    checkbox.do_state_hook(state, &mut self.layout, &self.fonts)
                }
                Component::Text(text) => {
                    text.do_state_hook(state, &mut self.layout, &self.fonts)
                }
                Component::Groupbox(groupbox) => {
                    groupbox.do_state_hook(state, &mut self.layout, &self.fonts)
                }
                Component::Tabs(tabs) => {
                    tabs.do_state_hook(state, &mut self.layout, &self.fonts)
                }
            }
        }

        if updated {
            self.layout.calculate_coords();

            for c in &mut self.components {
                match c {
                    Component::Button(button) => {
                        button.update(&mut self.layout, &self.fonts, self.scale)
                    }
                    Component::Checkbox(checkbox) => {
                        checkbox.update(&mut self.layout, &self.fonts, self.scale)
                    }
                    Component::Text(text) => {
                        text.update(&mut self.layout, &self.fonts, self.scale)
                    }
                    Component::Groupbox(groupbox) => {
                        groupbox.update(&mut self.layout, &self.fonts, self.scale)
                    }
                    Component::Tabs(tabs) => {
                        tabs.update(&mut self.layout, &self.fonts, self.scale)
                    }
                }
            }
        }
    }

    pub fn run_post_update(&mut self, state: &mut T) {
        if let Some(f) = self.post_update {
            (f)(state);

            self.run_state_hooks(state);
        }
    }

    pub fn wndproc(
        &mut self,
        state: &mut T,
        window: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match msg {
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                WM_PAINT => {
                    let mut ps = PAINTSTRUCT::default();
                    let hdc = BeginPaint(window, &mut ps);

                    FillRect(hdc, &ps.rcPaint, HBRUSH(COLOR_WINDOW.0 as *mut c_void));

                    _ = EndPaint(window, &ps);

                    LRESULT(0)
                }
                WM_DPICHANGED => {
                    let suggested_size = *(lparam.0 as *mut RECT);

                    self.rescale(Some((suggested_size.left, suggested_size.top)));

                    LRESULT(0)
                }
                WM_COMMAND => {
                    let id = (wparam.0 & 0xffff) as u16;

                    let mut result = None;

                    if let Some(c) = self.components.get_mut(id as usize - 1) {
                        match c {
                            Component::Button(button) => {
                                result =
                                    button.wndproc_on_pressed(state, window, msg, wparam, lparam)
                            }
                            Component::Checkbox(checkbox) => {
                                result =
                                    checkbox.wndproc_on_toggled(state, window, msg, wparam, lparam)
                            }
                            Component::Text(_) |
                            Component::Groupbox(_) |
                            Component::Tabs(_) => {}
                        }
                    }

                    match result {
                        Some(v) => {
                            self.run_state_hooks(state);

                            self.run_post_update(state);

                            v
                        }
                        None => DefWindowProcW(window, msg, wparam, lparam),
                    }
                }
                WM_NOTIFY => {
                    let id = (wparam.0 & 0xffff) as u16;

                    let mut result = None;

                    if let Some(c) = self.components.get_mut(id as usize - 1) {
                        match c {
                            Component::Tabs(tabs) => {
                                println!("Test!");
                            }
                            Component::Button(_) |
                            Component::Checkbox(_) |
                            Component::Text(_) |
                            Component::Groupbox(_) => {}
                        }
                    }

                    match result {
                        Some(v) => {
                            self.run_state_hooks(state);

                            self.run_post_update(state);

                            v
                        }
                        None => DefWindowProcW(window, msg, wparam, lparam),
                    }
                }
                _ => DefWindowProcW(window, msg, wparam, lparam),
            }
        }
    }
}

struct ComponentTreeNode {
    component_id: Option<usize>,

    ty: ComponentTreeNodeType,
}

enum ComponentTreeNodeType {
    Leaf,
    Node {
        children: Vec<GenArenaKey>,
    },
    Tabs {
        children: Vec<Vec<GenArenaKey>>,
    }
}
