use std::ffi::CStr;

use partymod_common::{patch, syncunsafecell::SyncUnsafeCell};

use crate::{config, file, window};

pub struct GfxContext {
    orig_create_device: *const (),
}

unsafe impl Sync for GfxContext {}
unsafe impl Send for GfxContext {}

pub static GFX_CONTEXT: SyncUnsafeCell<Option<GfxContext>> = SyncUnsafeCell::new(None);

pub fn init() {
    unsafe {
        let ctx = &mut *GFX_CONTEXT.get();
        *ctx = Some(GfxContext {
            orig_create_device: std::ptr::null(),
        });
    }

    // patch in bits.tdx containing correct shadow.png
    file::register_patch(
        ".\\data\\models\\bits\\bits.tdx",
        include_bytes!("patches/bits.bps"),
    );

    // fix all bsps which lack face cull mode data
    file::register_patch(
        ".\\data\\levels\\ap\\ap.bsp",
        include_bytes!("patches/ap.bps"),
    );
    /*file::register_patch(
        ".\\data\\levels\\burn\\burn.bsp",
        include_bytes!("patches/burn.bps"),
    );*/
    file::register_patch(
        ".\\data\\levels\\can\\can.bsp",
        include_bytes!("patches/can.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\foun\\foun.bsp",
        include_bytes!("patches/foun.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\la\\la.bsp",
        include_bytes!("patches/la.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\rio\\rio.bsp",
        include_bytes!("patches/rio.bps"),
    );
    /*file::register_patch(
        ".\\data\\levels\\ros\\ros.bsp",
        include_bytes!("patches/ros.bps"),
    );*/
    file::register_patch(
        ".\\data\\levels\\shp\\shp.bsp",
        include_bytes!("patches/shp.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\si\\si.bsp",
        include_bytes!("patches/si.bps"),
    );
    /*file::register_patch(
        ".\\data\\levels\\sk3ed_bch\\sk3ed_bch.bsp",
        include_bytes!("patches/sk3ed_bch.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\sk3ed_indr\\sk3ed_indr.bsp",
        include_bytes!("patches/sk3ed_indr.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\sk3ed_schl\\sk3ed_schl.bsp",
        include_bytes!("patches/sk3ed_schl.bps"),
    );*/
    file::register_patch(
        ".\\data\\levels\\skateshop\\skateshop.bsp",
        include_bytes!("patches/skateshop.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\sub\\sub.bsp",
        include_bytes!("patches/sub.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\tok\\tok.bsp",
        include_bytes!("patches/tok.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\tut\\tut.bsp",
        include_bytes!("patches/tut.bps"),
    );
    /*file::register_patch(
        ".\\data\\levels\\ware\\ware.bsp",
        include_bytes!("patches/ware.bps"),
    );*/

    // texture dictionary patches to reintroduce transparency
    // not confident they're right due to some inaccuracies
    // SI shine notably isn't quite right and suburbia wall dirt is maybe off
    // also airport scanline effect doesn't animate/is broken so it's hidden fully

    file::register_patch(
        ".\\data\\levels\\foun\\foun.tdx",
        include_bytes!("patches/fountdx.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\rio\\rio.tdx",
        include_bytes!("patches/riotdx.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\sub\\sub.tdx",
        include_bytes!("patches/subtdx.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\ap\\ap.tdx",
        include_bytes!("patches/aptdx.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\si\\si.tdx",
        include_bytes!("patches/sitdx.bps"),
    );
    file::register_patch(
        ".\\data\\levels\\la\\la.tdx",
        include_bytes!("patches/latdx.bps"),
    );

    // alt patches that remove pseudo-env map flags from materials
    // otherwise those materials don't render
    // TODO: make it so we don't need these
    //file::register_patch(".\\data\\levels\\la\\la.bsp", include_bytes!("patches/la-noreflect.bps"));
    //file::register_patch(".\\data\\levels\\shp\\shp.bsp", include_bytes!("patches/shp-noreflect.bps"));
    //file::register_patch(".\\data\\levels\\si\\si.bsp", include_bytes!("patches/si-noreflect.bps"));
}

pub fn init_settings() {
    unsafe {
        let ptr_custom_settings = 0x008510b1 as *mut bool;
        let ptr_distance_fog = 0x008510b2 as *mut bool;
        let ptr_low_detail_models = 0x008510b3 as *mut bool;
        let ptr_frame_cap = 0x008510b4 as *mut bool;
        let ptr_bit_depth = 0x0085108c as *mut u32;
        let ptr_shadows = 0x005b4e76 as *mut bool;
        let ptr_particles = 0x005b4e77 as *mut bool;
        let ptr_animating_textures = 0x005b4e78 as *mut bool;

        *ptr_animating_textures = config::get_bool("Graphics", "AnimatedTextures", true);
        *ptr_particles = config::get_bool("Graphics", "Particles", true);
        *ptr_shadows = config::get_bool("Graphics", "Shadows", true);
        *ptr_distance_fog = config::get_bool("Graphics", "DistanceFog", false);
        *ptr_low_detail_models = config::get_bool("Graphics", "LowDetailModels", false);

        *ptr_custom_settings = true;
        *ptr_frame_cap = true;

        *ptr_bit_depth = 32;
    }
}

fn get_shadow_setting() -> bool {
    unsafe {
        let ptr_shadows = 0x005b4e76 as *mut bool;

        *ptr_shadows
    }
}

unsafe fn skater_shadow_render_wrapper(unk: u32) {
    unsafe {
        let orig_skater_shadow_render: extern "C" fn(u32) = std::mem::transmute(0x00529170);
        let rw_get_render_state: extern "C" fn(u32, *mut i32) = std::mem::transmute(0x0055ce60);
        let rw_set_render_state: extern "C" fn(u32, i32) = std::mem::transmute(0x0055ce10);

        // going in, depth test/write have already been disabled.

        // grab our other current state and then set it for the skater
        let mut current_alpha_state = 0;
        rw_get_render_state(12, &mut current_alpha_state);

        let mut current_cull_state = 0;
        rw_get_render_state(20, &mut current_cull_state);

        let mut current_src_blend_state = 0;
        rw_get_render_state(10, &mut current_src_blend_state);

        let mut current_dst_blend_state = 0;
        rw_get_render_state(11, &mut current_dst_blend_state);

        // disable alpha blending
        rw_set_render_state(12, 0);
        // draw two-sided
        rw_set_render_state(20, 1);
        // set blend src to src alpha
        rw_set_render_state(10, 1);
        // set blend dst to 1 - src alpha
        rw_set_render_state(11, 1);

        // draw the shadow
        orig_skater_shadow_render(unk);

        // reset state
        rw_set_render_state(12, current_alpha_state);
        rw_set_render_state(20, current_cull_state);
        rw_set_render_state(10, current_src_blend_state);
        rw_set_render_state(11, current_dst_blend_state);
    }
}

#[repr(C)]
struct RW3DVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: u32,
    uv: [f32; 2],
}

unsafe fn draw_skater_shadow() {
    unsafe {
        let rw_im_3d_transform: extern "C" fn(*const RW3DVertex, u32, *const (), u32) =
            std::mem::transmute(0x00561010);
        let rw_im_3d_render_indexed_primitive: extern "C" fn(u32, *const u32, u32) =
            std::mem::transmute(0x00561110);
        let rw_get_render_state: extern "C" fn(u32, *mut i32) = std::mem::transmute(0x0055ce60);
        let rw_set_render_state: extern "C" fn(u32, i32) = std::mem::transmute(0x0055ce10);

        let vertex_count = *(0x00930bb8 as *const u32);
        let p_vertices = 0x0090b8c8 as *mut RW3DVertex;
        let p_indices = 0x00926948 as *const u32;

        // set transparency to half for each vertex
        let vertices = std::slice::from_raw_parts_mut(p_vertices, vertex_count as usize);

        for vertex in vertices {
            vertex.color = 0x80ffffff;
        }

        let mut current_alpha_state = 0;
        rw_get_render_state(12, &mut current_alpha_state);

        let mut current_filter_state = 0;
        rw_get_render_state(9, &mut current_filter_state);

        let mut current_src_blend_state = 0;
        rw_get_render_state(10, &mut current_src_blend_state);

        let mut current_dst_blend_state = 0;
        rw_get_render_state(11, &mut current_dst_blend_state);

        // load bearing state changes. no idea why these must be here but
        // if they're not, certain skaters won't have transparent shadows.
        // i think something is not resetting d3d state so i think this forces
        // a flush.

        // set blend src to src color
        rw_set_render_state(10, 3);
        // set blend dst to dst color
        rw_set_render_state(11, 9);

        rw_im_3d_transform(p_vertices, vertex_count, std::ptr::null(), 1);

        // enable alpha blending
        rw_set_render_state(12, 1);
        // set filter to linear
        rw_set_render_state(9, 2);
        // set blend src to src alpha
        rw_set_render_state(10, 5);
        // set blend src to 1 - src alpha
        rw_set_render_state(11, 6);

        rw_im_3d_render_indexed_primitive(3, p_indices, vertex_count);

        // reset state
        rw_set_render_state(12, current_alpha_state);
        rw_set_render_state(9, current_filter_state);
        rw_set_render_state(10, current_src_blend_state);
        rw_set_render_state(11, current_dst_blend_state);
    }
}

unsafe fn draw_blob_shadow() {
    unsafe {
        let rw_im_3d_transform: extern "C" fn(*const RW3DVertex, u32, *const (), u32) =
            std::mem::transmute(0x00561010);
        let rw_im_3d_render_indexed_primitive: extern "C" fn(u32, *const u32, u32) =
            std::mem::transmute(0x00561110);
        let rw_get_render_state: extern "C" fn(u32, *mut i32) = std::mem::transmute(0x0055ce60);
        let rw_set_render_state: extern "C" fn(u32, i32) = std::mem::transmute(0x0055ce10);

        let vertex_count = *(0x00930bbc as *const u32);
        let p_vertices = 0x00928778 as *mut RW3DVertex;
        let p_indices = 0x00928150 as *const u32;

        // set transparency to half for each vertex
        let vertices = std::slice::from_raw_parts_mut(p_vertices, vertex_count as usize);

        for vertex in vertices {
            vertex.color = 0xffffffff;
        }

        let mut current_alpha_state = 0;
        rw_get_render_state(12, &mut current_alpha_state);

        let mut current_filter_state = 0;
        rw_get_render_state(9, &mut current_filter_state);

        let mut current_src_blend_state = 0;
        rw_get_render_state(10, &mut current_src_blend_state);

        let mut current_dst_blend_state = 0;
        rw_get_render_state(11, &mut current_dst_blend_state);

        let mut current_cull_state = 0;
        rw_get_render_state(20, &mut current_cull_state);

        // enable alpha blending
        rw_set_render_state(12, 1);
        // set filter to linear
        rw_set_render_state(9, 2);
        // set blend src to src alpha
        rw_set_render_state(10, 1);
        // set blend dst to one
        rw_set_render_state(11, 6);
        // set blend dst to one
        rw_set_render_state(20, 1);

        rw_im_3d_transform(p_vertices, vertex_count * 4, std::ptr::null(), 1);
        rw_im_3d_render_indexed_primitive(3, p_indices, vertex_count * 6);

        rw_set_render_state(12, current_alpha_state);
        rw_set_render_state(9, current_filter_state);
        rw_set_render_state(10, current_src_blend_state);
        rw_set_render_state(11, current_dst_blend_state);
        rw_set_render_state(20, current_cull_state);
    }
}

extern "C" fn create_shadow_wrapper(
    obj: *const (),
    unk1: *const (),
    unk2: u32,
    unk3: u8,
    is_hq_shadow: bool,
    unk4: f32,
) {
    unsafe {
        let orig_func: extern "C" fn(*const (), *const (), u32, u8, bool, f32) =
            std::mem::transmute(0x005010e0);

        if !get_shadow_setting() {
            // always use lq shadow
            orig_func(obj, unk1, unk2, unk3, false, unk4);
        } else {
            orig_func(obj, unk1, unk2, unk3, is_hq_shadow, unk4);
        }
    }
}

unsafe fn patch_shadows() {
    unsafe {
        // skater shadow render
        patch::patch_call(
            0x00501548 as *mut (),
            skater_shadow_render_wrapper as *const (),
        );

        // fix skater shadow alpha
        patch::patch_nop(0x0050209b as *mut (), 37);
        patch::patch_call(0x0050209b as *mut (), draw_skater_shadow as *const ());

        // fix blob shadow alpha
        patch::patch_nop(0x00501839 as *mut (), 45);
        patch::patch_call(0x00501839 as *mut (), draw_blob_shadow as *const ());
        patch::patch_byte((0x00501866 + 2) as *mut (), 0x08); // only add 8 to ESP

        // always draw shadows
        patch::patch_nop(0x0042fa4f as *mut (), 2);
        patch::patch_nop(0x005010a6 as *mut (), 2);
        patch::patch_nop(0x00501102 as *mut (), 6);
        patch::patch_call(0x004ab35b as *mut (), create_shadow_wrapper as *const ());

        // don't skip drawing blob shadows
        patch::patch_nop(0x005017d3 as *mut (), 2);
        // increase blob shadow view distance
        patch::patch_f32((0x0049466d + 1) as *mut (), 2400.0);
        // reduce blob shadow size
        patch::patch_f32((0x005017d5 + 1) as *mut (), 12.0);
    }
}

unsafe extern "C" fn do_draw_side_wrapper(unk: *const ()) -> u32 {
    unsafe {
        let orig_func: extern "C" fn(*const ()) -> u32 = std::mem::transmute(0x00526fc0);
        let rw_set_render_state: extern "C" fn(u32, i32) = std::mem::transmute(0x0055ce10);

        // material dump

        let material = *(unk.byte_add(0x54) as *mut *mut ());

        let bytes = material as *mut [u8; 128];

        /*
        print!("MATERIAL:");
        for (i, byte) in (*bytes).iter().enumerate() {
            if i % 4 == 0 {
                print!(" ");
            }
            print!("{:02x}", byte);
        }
        */

        /*let texture = *(material as *mut *mut ());
        if !texture.is_null() {
            let bytes = (texture.byte_add(0x10)) as *mut [u8; 128];
            print!(" TEXTURE: ");
            let name = CStr::from_bytes_until_nul(&*bytes).unwrap();
            print!("{}", name.to_string_lossy());
            for (i, byte) in (*bytes).iter().enumerate() {
                if i % 4 == 0 {
                    print!(" ");
                }
                print!("{:02x}", byte);
            }
        }*/

        //print!("\n");

        let flag = **(unk.byte_add(0x54) as *const *const u32);

        if flag & 1 != 0 {
            rw_set_render_state(20, 1);
        } else {
            rw_set_render_state(20, 2);
        }

        orig_func(unk)
    }
}

unsafe extern "C" fn materialdump(sector: *mut (), out: *mut ()) {
    unsafe {
        let orig_func: extern "C" fn(*mut (), *mut ()) -> u32 = std::mem::transmute(0x004f9640);

        let material = *((sector as *mut *mut ()).byte_add(8));

        let bytes = material as *mut [u8; 128];

        print!("MATERIAL:");
        for (i, byte) in (*bytes).iter().enumerate() {
            if i % 4 == 0 {
                print!(" ");
            }
            print!("{:02x}", byte);
        }

        let texture = *(material as *mut *mut ());
        if !texture.is_null() {
            let bytes = (texture.byte_add(0x10)) as *mut [u8; 128];
            print!(" TEXTURE: ");
            let name = CStr::from_bytes_until_nul(&*bytes).unwrap();
            print!("{}", name.to_string_lossy());
            for (i, byte) in (*bytes).iter().enumerate() {
                if i % 4 == 0 {
                    print!(" ");
                }
                print!("{:02x}", byte);
            }
        }

        print!("\n");

        orig_func(sector, out);

        /*if (*bytes)[0x1a] & 0x20 != 0 {
            (*bytes)[0x1a] &= !0x20;
        }*/
    }
}

unsafe extern "C" fn texturedump(material: *mut ()) {
    unsafe {
        let orig_func: extern "C" fn(*mut ()) -> u32 = std::mem::transmute(0x004f4320);

        if material.is_null() || (*(material as *mut *mut ())).is_null() {
            orig_func(material);
            return;
        }

        let texture = *((*(material as *mut *mut ())) as *mut *mut ());
        if !texture.is_null() {
            let name_bytes = (texture.byte_add(0x10)) as *mut [u8; 128];
            print!(" TEXTURE: ");
            let name = CStr::from_bytes_until_nul(&*name_bytes).unwrap();
            print!("{}", name.to_string_lossy());

            let bytes = texture as *mut [u8; 128];
            for (i, byte) in (*bytes).iter().enumerate() {
                if i % 4 == 0 {
                    print!(" ");
                }
                print!("{:02x}", byte);
            }

            print!("\n");
        }

        orig_func(material);
    }
}

unsafe fn patch_draw_side() {
    unsafe {
        patch::patch_call(0x004f4278 as *mut (), do_draw_side_wrapper as *const ());
        patch::patch_call(0x00522192 as *mut (), do_draw_side_wrapper as *const ());

        // all of this stuff was research for the pseudo env map patch lol
        //patch::patch_jmp(0x00550eec as *mut (), 0x00550f29 as *const ());
        //patch::patch_nop(0x004f48c5 as *mut (), 2);
        //patch::patch_nop(0x004f9bff as *mut (), 5);
        //patch::patch_nop(0x004f498b as *mut (), 2);

        // disables animated textures
        //patch::patch_byte(0x004010bd as *mut (), 0xeb);
        //patch::patch_nop(0x004010bd as *mut (), 2);

        // ditto
        //patch::patch_byte(0x0040114c as *mut (), 0xeb);

        //patch::patch_byte(0x004f964e as *mut (), 0xeb);
        //patch::patch_byte(0x0040123f as *mut (), 0xeb);

        //patch::patch_nop(0x0040172b as *mut (), 6);

        //patch::patch_byte(0x004f43f7 as *mut (), 0xeb);

        //patch::patch_nop(0x004f964b as *mut (), 15);

        //patch::patch_u32((0x004fa235 + 1) as *mut (), materialdump as u32);
        //patch::patch_call(0x004f4992 as *mut (), texturedump as *const ());
        //patch::patch_call(0x004f48cc as *mut (), texturedump as *const ());

        // test if this disables textures
        //patch::patch_byte(0x004f498f as *mut (), 0xeb);

        // test if this disables rendering
        //patch::patch_nop(0x004fa244 as *mut (), 5);
        //patch::patch_byte(0x0055bada as *mut (), 0xeb);
        //patch::patch_byte(0x0055bb2a as *mut (), 0xeb);
        //patch::patch_nop(0x0055bb22 as *mut (), 3);
        //patch::patch_nop(0x0055bb2a as *mut (), 2);
        //patch::patch_nop(0x0055bb36 as *mut (), 2);

        // reflection goes to 00534be0

        //patch::patch_byte(0x00534c0f as *mut (), 0xeb);
        //patch::patch_bytes(0x00534bfa as *mut (), &[0xe9, 0x8a, 0x00, 0x00, 0x00]);
        //patch::patch_byte(0x00534c7a as *mut (), 0xeb);

        // 0x00534c84 -> 0x00526b20 (0xc9, 0xff, 0x05880640, 0x0422e8a0)
        //patch::patch_jmp(0x00526b5c as *mut (), 0x00526f08 as *const ());
        //patch::patch_byte(0x00526b79 as *mut (), 0xeb);

        // 0x00526b83 -> 0x004f4b00
        // 0x004f4b37 is where it's skipped

        // don't skip drawing pseudo env map materials
        patch::patch_nop(0x004f4b37 as *mut (), 6);

        // blend mode tests maybe fixes
        //patch::patch_byte((0x004f463f + 1) as *mut (), 0x03);

        // try changing vertex colors
        /*
        patch::patch_u32((0x00401319 + 2) as *mut (), 0x0058d3e8);
        patch::patch_u32((0x0040132b + 2) as *mut (), 0x0058d3e8);
        patch::patch_u32((0x0040133e + 2) as *mut (), 0x0058d3e8);

        patch::patch_u32((0x00534ed9 + 2) as *mut (), 0x0058d3e8);
        patch::patch_u32((0x00534eeb + 2) as *mut (), 0x0058d3e8);
        patch::patch_u32((0x00534efe + 2) as *mut (), 0x0058d3e8);
        */

        /*
        patch::patch_u32((0x004013b6 + 2) as *mut (), 0x0058d3ec);
        patch::patch_u32((0x004013d0 + 2) as *mut (), 0x0058d3ec);
        patch::patch_u32((0x004013e1 + 2) as *mut (), 0x0058d3ec);
        patch::patch_u32((0x004013f4 + 2) as *mut (), 0x0058d3ec);
        */

        //patch::patch_u32((0x0040152a + 2) as *mut (), (&raw const F_1_OVER_512) as *const () as u32);
        //patch::patch_u32((0x004013b6 + 2) as *mut (), (&raw const F_1_OVER_512) as *const () as u32);

        //patch::patch_byte((0x004f46c4 + 1) as *mut (), 6);

        // fix blend dst factor for suburbia fence dirt and SI outdoor ramps
        // src alpha -> 1 - src alpha
        patch::patch_byte((0x004f4605 + 1) as *mut (), 6);

        // fix blend op for airport screens
        // subtract -> add
        patch::patch_byte((0x004f451c + 1) as *mut (), 1);
    }
}

pub unsafe fn patch_aspect_ratio(aspect_ratio: f32) {
    unsafe {
        patch::patch_f32(0x0058eb14 as *mut (), aspect_ratio);
        patch::patch_f32(0x0058d96c as *mut (), aspect_ratio);
    }
}

unsafe extern "C" fn get_gfx_manager_wrapper_every_frame(create: bool) -> *const std::ffi::c_void {
    // function to sneak in some stuff to do every frame
    let (width, height) = window::get_window_size();
    let aspect_ratio = width as f32 / height as f32;
    unsafe {
        patch_aspect_ratio(aspect_ratio);

        let orig_func: extern "C" fn(bool) -> *const std::ffi::c_void =
            std::mem::transmute(0x004f92d0);

        orig_func(create)
    }
}

unsafe fn patch_every_frame() {
    unsafe {
        patch::patch_call(
            0x0042f7b2 as *mut (),
            get_gfx_manager_wrapper_every_frame as *const (),
        );
    }
}

unsafe extern "stdcall" fn create_device_wrapper(
    d3d8: *const std::ffi::c_void,
    adapter: u32,
    ty: u32,
    hwnd: *const std::ffi::c_void,
    behavior_flags: u32,
    present_params: *mut std::ffi::c_void,
    device_out: *mut std::ffi::c_void,
) -> u32 {
    unsafe {
        let gfx_context = match &mut *GFX_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to use uninitialized file context!"),
        };

        let orig_func: extern "stdcall" fn(
            *const std::ffi::c_void,
            u32,
            u32,
            *const std::ffi::c_void,
            u32,
            *mut std::ffi::c_void,
            *mut std::ffi::c_void,
        ) -> u32 = std::mem::transmute(gfx_context.orig_create_device);

        *(present_params.byte_add(12) as *mut u32) = 1;
        *(present_params.byte_add(20) as *mut u32) = 1;

        if *(present_params.byte_add(28) as *mut u32) != 0 {
            *(present_params.byte_add(44) as *mut u32) = 0;
            *(present_params.byte_add(48) as *mut u32) = 0;
        } else {
            *(present_params.byte_add(44) as *mut u32) = 0;
            *(present_params.byte_add(48) as *mut u32) = 1;
        }

        orig_func(
            d3d8,
            adapter,
            ty,
            hwnd,
            behavior_flags,
            present_params,
            device_out,
        )
    }
}

unsafe extern "stdcall" fn create_d3d8_wrapper(version: u32) -> *const std::ffi::c_void {
    unsafe {
        let orig_func: extern "stdcall" fn(u32) -> *const std::ffi::c_void =
            std::mem::transmute(0x00569d20);

        let result = orig_func(version);

        if !result.is_null() {
            let gfx_context = match &mut *GFX_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to use uninitialized gfx context!"),
            };

            let create_device = (*(result as *const *mut *const ())).byte_add(0x3c);

            gfx_context.orig_create_device = *create_device;
            patch::patch_u32(
                create_device as *mut (),
                create_device_wrapper as *const () as u32,
            );
        }

        result
    }
}

unsafe fn patch_presentation_mode() {
    unsafe {
        patch::patch_call(0x0054e433 as *mut (), create_d3d8_wrapper as *const ());
    }
}

static F_ZERO: f32 = 0.0;
static F_1_OVER_512: f32 = 1.0 / 512.0;

pub unsafe fn patch() {
    unsafe {
        patch_shadows();
        patch_draw_side();
        patch_every_frame();
        patch_presentation_mode();
    }
}
