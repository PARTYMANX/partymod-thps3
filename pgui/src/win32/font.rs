use std::ffi::c_void;

use windows::{
    Win32::{
        Graphics::Gdi::{CreateFontW, HFONT},
        UI::{
            HiDpi::SystemParametersInfoForDpi,
            WindowsAndMessaging::{NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS},
        },
    },
    core::PCWSTR,
};

pub struct Fonts {
    /// Unscaled copy of the default GUI font, for layout calculation purposes.
    pub default_font: HFONT,
    pub default_font_height: u32,
    /// Scaled copy of the default GUI font, for actual use.
    pub default_font_scaled: HFONT,
}

unsafe impl Sync for Fonts {}

impl Fonts {
    pub fn new() -> Self {
        unsafe {
            let mut ncm = NONCLIENTMETRICSW::default();
            ncm.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
            let _ = SystemParametersInfoForDpi(
                SPI_GETNONCLIENTMETRICS.0,
                ncm.cbSize,
                Some(&raw mut ncm as *mut c_void),
                0,
                96,
            );

            let lf = ncm.lfMessageFont;
            let default_font = CreateFontW(
                lf.lfHeight,
                lf.lfWidth,
                lf.lfEscapement,
                lf.lfOrientation,
                lf.lfWeight,
                lf.lfItalic as u32,
                lf.lfUnderline as u32,
                lf.lfStrikeOut as u32,
                lf.lfCharSet,
                lf.lfOutPrecision,
                lf.lfClipPrecision,
                lf.lfQuality,
                lf.lfPitchAndFamily as u32,
                PCWSTR(lf.lfFaceName.as_ptr()),
            );
            let default_font_height = lf.lfHeight.abs() as u32;

            let default_font_scaled = CreateFontW(
                lf.lfHeight,
                lf.lfWidth,
                lf.lfEscapement,
                lf.lfOrientation,
                lf.lfWeight,
                lf.lfItalic as u32,
                lf.lfUnderline as u32,
                lf.lfStrikeOut as u32,
                lf.lfCharSet,
                lf.lfOutPrecision,
                lf.lfClipPrecision,
                lf.lfQuality,
                lf.lfPitchAndFamily as u32,
                PCWSTR(lf.lfFaceName.as_ptr()),
            );

            Self {
                default_font,
                default_font_height,
                default_font_scaled,
            }
        }
    }

    pub fn update(&mut self, scale: f32) {
        let mut ncm = NONCLIENTMETRICSW::default();
        ncm.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
        unsafe {
            let dpi = (96.0 * scale) as u32;
            let _ = SystemParametersInfoForDpi(
                SPI_GETNONCLIENTMETRICS.0,
                ncm.cbSize,
                Some(&raw mut ncm as *mut c_void),
                0,
                dpi,
            );

            let lf = ncm.lfMessageFont;
            self.default_font_scaled = CreateFontW(
                lf.lfHeight,
                lf.lfWidth,
                lf.lfEscapement,
                lf.lfOrientation,
                lf.lfWeight,
                lf.lfItalic as u32,
                lf.lfUnderline as u32,
                lf.lfStrikeOut as u32,
                lf.lfCharSet,
                lf.lfOutPrecision,
                lf.lfClipPrecision,
                lf.lfQuality,
                lf.lfPitchAndFamily as u32,
                PCWSTR(lf.lfFaceName.as_ptr()),
            );
        }
    }
}
