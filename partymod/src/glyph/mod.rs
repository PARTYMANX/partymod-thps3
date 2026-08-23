use crate::{config, file};

pub fn init() {
    let style = config::get_int("Miscellaneous", "ButtonGlyphStyle", 0);

    match style {
        1 => {
            file::register_patch(
                ".\\data\\fonts\\small.fnt",
                include_bytes!("patches/smallps2.bps"),
            );
        }
        2 => {
            file::register_patch(
                ".\\data\\fonts\\small.fnt",
                include_bytes!("patches/smallps2j.bps"),
            );
        }
        3 => {
            file::register_patch(
                ".\\data\\fonts\\small.fnt",
                include_bytes!("patches/smallxbx.bps"),
            );
        }
        _ => {}
    }
}
