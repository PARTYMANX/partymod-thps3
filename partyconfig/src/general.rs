use std::{collections::HashSet, fmt};

use pgui::{
    checkbox::checkbox,
    component::Component,
    container::{horizontal, vertical},
    dropdown::dropdown,
    groupbox::groupbox,
    layout::{HorizontalOffset, Size, VerticalOffset},
    text::text,
    textbox::textbox,
};
use windows::Win32::Graphics::Gdi::{DEVMODEW, ENUM_DISPLAY_SETTINGS_MODE, EnumDisplaySettingsW};

use crate::{AppState, ini::ConfigFile};

pub struct ResolutionInfo {
    display_options: Vec<String>,
    display_modes: Vec<DisplayMode>,
}

#[derive(PartialEq, Eq, Hash)]
pub struct DisplayMode {
    width: u32,
    height: u32,
}

const DEFAULT_DISPLAY_MODE: DisplayMode = DisplayMode {
    width: 0,
    height: 0,
};

impl Ord for DisplayMode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.height
            .cmp(&other.height)
            .then(self.width.cmp(&other.width))
    }
}

impl PartialOrd for DisplayMode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self == &DEFAULT_DISPLAY_MODE {
            write!(f, "Default Desktop Resolution")
        } else {
            write!(f, "{}x{}", self.width, self.height)
        }
    }
}

impl ResolutionInfo {
    pub fn init() -> Self {
        let mut device_mode = DEVMODEW::default();
        let mut i = 0;

        let mut display_mode_set = HashSet::new();

        while unsafe {
            EnumDisplaySettingsW(None, ENUM_DISPLAY_SETTINGS_MODE(i), &mut device_mode).as_bool()
        } {
            let display_mode = DisplayMode {
                width: device_mode.dmPelsWidth,
                height: device_mode.dmPelsHeight,
            };

            display_mode_set.insert(display_mode);

            i += 1;
        }

        let mut display_options = Vec::new();
        let mut display_modes = Vec::new();

        display_modes.push(DisplayMode {
            width: 0,
            height: 0,
        });

        for display_mode in display_mode_set {
            display_modes.push(display_mode);
        }

        display_modes.sort();

        for display_mode in &display_modes {
            display_options.push(format!("{}", display_mode))
        }

        Self {
            display_options,
            display_modes,
        }
    }

    pub fn get_display_option_list(&self) -> Vec<String> {
        self.display_options.clone()
    }
}

pub struct GeneralState {
    resolution: u32,
    using_custom_resolution: bool,
    editing_custom_width: bool,
    editing_custom_height: bool,
    custom_resolution_width: u32,
    custom_resolution_height: u32,
    windowed: bool,
    borderless: bool,
}

impl GeneralState {
    pub fn new(config_file: &ConfigFile, resolution_info: &ResolutionInfo) -> Self {
        let resolution_width = config_file.get_config_int("Graphics", "ResolutionX", 0);
        let resolution_height = config_file.get_config_int("Graphics", "ResolutionY", 0);
        let target_display_mode = DisplayMode {
            width: resolution_width as u32,
            height: resolution_height as u32,
        };

        let (resolution, using_custom_resolution) = match resolution_info
            .display_modes
            .binary_search(&target_display_mode)
        {
            Ok(v) => (v as u32, false),
            Err(_) => (0, true),
        };

        let (custom_resolution_width, custom_resolution_height) = if using_custom_resolution {
            (resolution_width as u32, resolution_height as u32)
        } else {
            (0, 0)
        };

        let windowed = config_file.get_config_bool("Graphics", "Windowed", false);
        let borderless = config_file.get_config_bool("Graphics", "Borderless", false);

        Self {
            resolution,
            using_custom_resolution,
            editing_custom_width: false,
            editing_custom_height: false,
            custom_resolution_width,
            custom_resolution_height,
            windowed,
            borderless,
        }
    }

    pub fn save(&self, config_file: &ConfigFile, resolution_info: &ResolutionInfo) {
        if self.using_custom_resolution {
            config_file.set_config_int(
                "Graphics",
                "ResolutionX",
                self.custom_resolution_width as i32,
            );
            config_file.set_config_int(
                "Graphics",
                "ResolutionY",
                self.custom_resolution_height as i32,
            );
        } else {
            let display_mode = &resolution_info.display_modes[self.resolution as usize];
            config_file.set_config_int("Graphics", "ResolutionX", display_mode.width as i32);
            config_file.set_config_int("Graphics", "ResolutionY", display_mode.height as i32);
        }

        config_file.set_config_bool("Graphics", "Windowed", self.windowed);
        config_file.set_config_bool("Graphics", "Borderless", self.borderless);
    }
}

impl Default for GeneralState {
    fn default() -> Self {
        Self {
            resolution: 0,
            using_custom_resolution: false,
            editing_custom_width: false,
            editing_custom_height: false,
            custom_resolution_width: 0,
            custom_resolution_height: 0,
            windowed: false,
            borderless: false,
        }
    }
}

pub fn general_page(width: u32, height: u32, resolution_list: Vec<String>) -> Component<AppState> {
    vertical(vec![
        groupbox(
            "Resolution".to_string(),
            vertical(vec![
                dropdown(resolution_list)
                    .on_select(|app_state: &mut AppState, selected| {
                        app_state.general_state.resolution = selected;
                    })
                    .state_hook(|app_state: &AppState, dropdown_state| {
                        dropdown_state.enabled = !app_state.general_state.using_custom_resolution;
                        dropdown_state.selected = app_state.general_state.resolution;
                    })
                    .v_position(VerticalOffset::AlignTop(8))
                    .into(),
                checkbox("Use Custom Resolution".to_string())
                    .on_toggle(|app_state: &mut AppState, checked| {
                        app_state.general_state.using_custom_resolution = checked;
                    })
                    .state_hook(|app_state: &AppState, checkbox_state| {
                        checkbox_state.checked = app_state.general_state.using_custom_resolution;
                    })
                    .into(),
                horizontal(vec![
                    text("Width:".to_string())
                        .state_hook(|app_state: &AppState, text_state| {
                            text_state.enabled = app_state.general_state.using_custom_resolution;
                        })
                        .v_position(VerticalOffset::AlignTop(2))
                        .height(Size::Exact(16))
                        .into(),
                    textbox("".to_string())
                        .on_focus(|app_state: &mut AppState, _text| {
                            app_state.general_state.editing_custom_width = true;
                        })
                        .on_unfocus(|app_state: &mut AppState, text| {
                            app_state.general_state.editing_custom_width = false;

                            app_state.general_state.custom_resolution_width =
                                match text.trim().parse::<u32>() {
                                    Ok(v) => v,
                                    Err(_) => 0,
                                };
                        })
                        .state_hook(|app_state: &AppState, text_state| {
                            text_state.enabled = app_state.general_state.using_custom_resolution;

                            if !app_state.general_state.editing_custom_width {
                                text_state.text = if app_state.general_state.custom_resolution_width
                                    == 0
                                {
                                    "".to_string()
                                } else {
                                    format!("{}", app_state.general_state.custom_resolution_width)
                                }
                            }
                        })
                        .width(Size::Exact(50))
                        .height(Size::Exact(20))
                        .into(),
                    text("Height:".to_string())
                        .state_hook(|app_state: &AppState, text_state| {
                            text_state.enabled = app_state.general_state.using_custom_resolution;
                        })
                        .v_position(VerticalOffset::AlignTop(2))
                        .height(Size::Exact(16))
                        .into(),
                    textbox("".to_string())
                        .on_focus(|app_state: &mut AppState, _text| {
                            app_state.general_state.editing_custom_height = true;
                        })
                        .on_unfocus(|app_state: &mut AppState, text| {
                            app_state.general_state.editing_custom_height = false;

                            app_state.general_state.custom_resolution_height =
                                match text.trim().parse::<u32>() {
                                    Ok(v) => v,
                                    Err(_) => 0,
                                };
                        })
                        .state_hook(|app_state: &AppState, text_state| {
                            text_state.enabled = app_state.general_state.using_custom_resolution;

                            if !app_state.general_state.editing_custom_height {
                                text_state.text = if app_state
                                    .general_state
                                    .custom_resolution_height
                                    == 0
                                {
                                    "".to_string()
                                } else {
                                    format!("{}", app_state.general_state.custom_resolution_height)
                                }
                            }
                        })
                        .width(Size::Exact(50))
                        .height(Size::Exact(20))
                        .into(),
                ])
                .h_position(HorizontalOffset::AlignLeft(16))
                .spacing(8)
                .into(),
                checkbox("Windowed".to_string())
                    .on_toggle(|app_state: &mut AppState, checked| {
                        app_state.general_state.windowed = checked;
                    })
                    .state_hook(|app_state: &AppState, checkbox_state| {
                        checkbox_state.checked = app_state.general_state.windowed;
                    })
                    .into(),
                checkbox("Borderless".to_string())
                    .on_toggle(|app_state: &mut AppState, checked| {
                        app_state.general_state.borderless = checked;
                    })
                    .state_hook(|app_state: &AppState, checkbox_state| {
                        checkbox_state.checked = app_state.general_state.borderless;
                    })
                    .into(),
            ])
            .spacing(8)
            .into(),
        )
        .width(Size::Exact(width))
        .height(Size::Exact(height / 2))
        .into(),
        horizontal(vec![
            groupbox(
                "Graphics".to_string(),
                text("well we need something here".to_string()).into(),
            )
            .width(Size::Exact(width / 2))
            .height(Size::Exact(height / 2))
            .into(),
            groupbox(
                "Miscellaneous".to_string(),
                text("well we need something here".to_string()).into(),
            )
            .width(Size::Exact(width / 2))
            .height(Size::Exact(height / 2))
            .into(),
        ])
        .into(),
    ])
    .into()
}
