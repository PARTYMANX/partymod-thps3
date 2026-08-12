use std::{ptr, slice, sync::mpsc::Receiver};

use partymod_common::patch;
use windows::{Win32::{Foundation::{HWND, RECT}, Graphics::Imaging::{CLSID_WICImagingFactory, GUID_WICPixelFormat24bppBGR, GUID_WICPixelFormat24bppRGB, GUID_WICPixelFormat32bppBGRA, GUID_WICPixelFormat32bppPBGRA, IWICBitmap, IWICImagingFactory, WICBitmapCacheOnDemand, WICRect}, Media::MediaFoundation::{CLSID_MFMediaEngineClassFactory, IMFMediaEngine, IMFMediaEngineClassFactory, IMFMediaEngineNotify, IMFMediaEngineNotify_Impl, MF_MEDIA_ENGINE_CALLBACK, MF_MEDIA_ENGINE_DXGI_MANAGER, MF_MEDIA_ENGINE_EVENT_CANPLAY, MF_MEDIA_ENGINE_EVENT_ERROR, MF_MEDIA_ENGINE_EVENT_LOADSTART, MF_MEDIA_ENGINE_EVENT_NOTIFYSTABLESTATE, MF_MEDIA_ENGINE_PLAYBACK_HWND, MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT, MF_MEDIA_ENGINE_WAITFORSTABLE_STATE, MF_VERSION, MFARGB, MFCreateAttributes, MFShutdown, MFStartup, MFVideoNormalizedRect}, System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoInitialize, CoUninitialize}}, core::{BSTR, w}};
use windows_core::{ComObjectInner, IUnknown, implement};

use crate::{event, input, throttle, window};

struct MoviePlayer {
    mf_com: MfCom,
    wic_factory: IWICImagingFactory,
    media_engine_factory: IMFMediaEngineClassFactory,

    recv: Receiver<NotifyMessage>,
    media_engine: IMFMediaEngine,
    bitmap: IWICBitmap,

    width: u32,
    height: u32,

    texture: *const std::ffi::c_void,
}

impl MoviePlayer {
    fn new(path: &str) -> Option<Self> {
        let mf_com = MfCom::new()?;

        let wic_factory: IWICImagingFactory = unsafe {
            match CoCreateInstance(
                &CLSID_WICImagingFactory,
                None,
                CLSCTX_INPROC_SERVER,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return None;
                }
            }
        };

        let media_engine_factory: IMFMediaEngineClassFactory = unsafe { 
            match CoCreateInstance(
                &CLSID_MFMediaEngineClassFactory,
                None,
                CLSCTX_INPROC_SERVER,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return None;
                }
            }
        };

        let attributes = unsafe {
            let mut attr = None;
            if let Err(e) = MFCreateAttributes(&mut attr, 1) {
                return None;
            }

            attr.unwrap()
        };

        let (sender, recv) = std::sync::mpsc::channel();

        let notify: IUnknown = MediaEngineNotify {
            sender,
        }.into();

        unsafe {
            if let Err(e) = attributes.SetUnknown(
                &MF_MEDIA_ENGINE_CALLBACK, 
                &notify
            ) {
                return None;
            }
        }

        let media_engine = unsafe { 
            match media_engine_factory.CreateInstance(
                0,
                &attributes,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return None;
                }
            }
        };

        let wpath = BSTR::from(path);

        unsafe {
            if let Err(e) = media_engine.SetSource(&wpath) {
                return None;
            }
            
            if let Err(e) = media_engine.Load() {
                return None;
            }
        }

        loop {
            match recv.recv_timeout(std::time::Duration::from_secs_f64(1.0/60.0)) {
                Ok(v) => match v {
                    NotifyMessage::LoadReady => {
                        println!("LOAD START RECEIVED");
                        break;
                    },
                    NotifyMessage::PlaybackReady => todo!(),
                    NotifyMessage::Error => {
                        println!("Error!");

                        let _ = unsafe { media_engine.Shutdown() };

                        return None;
                    }
                },
                Err(_) => {
                    if Self::check_break_event() {
                        let _ = unsafe { media_engine.Shutdown() };

                        return None;
                    }
                },
            }
        }

        // wait for load or break event
        loop {
            match recv.recv_timeout(std::time::Duration::from_secs_f64(1.0/60.0)) {
                Ok(v) => match v {
                    NotifyMessage::LoadReady => {
                        println!("LOAD START RECEIVED???");
                    },
                    NotifyMessage::PlaybackReady => {
                        println!("LOADED!");
                        break;
                    },
                    NotifyMessage::Error => {
                        println!("Error!");

                        let _ = unsafe { media_engine.Shutdown() };

                        return None;
                    }
                },
                Err(_) => {
                    if Self::check_break_event() {
                        let _ = unsafe { media_engine.Shutdown() };

                        return None;
                    }
                },
            }
        }

        let mut width = 0;
        let mut height = 0;

        unsafe {
            if let Err(e) = media_engine.GetNativeVideoSize(Some(&mut width), Some(&mut height)) {
                let _ = media_engine.Shutdown();

                return None;
            }

            let _ = media_engine.SetVolume(0.02);
        }

        let bitmap = unsafe {
            match wic_factory.CreateBitmap(
                width,
                height,
                &GUID_WICPixelFormat32bppBGRA,
                WICBitmapCacheOnDemand
            ) {
                Ok(v) => v,
                Err(e) => {
                    let _ = media_engine.Shutdown();

                    return None;
                }
            }
        };

        let texture = unsafe {
            let d3d8_device = *(0x00970e48 as *const *const std::ffi::c_void);

            let d3d8_create_texture: *const extern "stdcall" fn(*const std::ffi::c_void, u32, u32, u32, u32, u32, u32, *mut *const std::ffi::c_void) -> u32
                = std::mem::transmute((*(d3d8_device as *const *const std::ffi::c_void)).byte_add(0x50));

            let mut texture = ptr::null();

            let result = (*d3d8_create_texture)(
                d3d8_device,
                width,
                height,
                1, // 1 level
                0, // no usage flags
                21, // 32 bit ARGB
                1, // managed storage
                &mut texture, // pointer out
            );

            if result != 0 {
                return None;
            }

            texture
        };

        Some(Self {
            mf_com,
            wic_factory,
            media_engine_factory,
            recv,
            media_engine,
            bitmap,
            width,
            height,
            texture,
        })
    }

    fn play(&mut self) {
        unsafe {
            if !self.media_engine.HasVideo().as_bool() && 
                !self.media_engine.HasAudio().as_bool() {
                return;
            }
        }

        let bitmap_rect = RECT {
            left: 0,
            top: 0,
            right: self.width as i32,
            bottom: self.height as i32,
        };

        unsafe {
            if let Err(e) = self.media_engine.Play() {
                return;
            }

            let mut is_breaking = false;
            while !is_breaking && !self.media_engine.IsEnded().as_bool() {
                // throttle to 60hz. this is a little weird but there's no easy
                // way to figure out what the target framerate should be.
                // oh well!
                throttle::throttle_frame();

                match self.media_engine.OnVideoStreamTick() {
                    Ok(_) => {
                        // documentation says this should only return Ok if there is a new frame.
                        // in reality, this always returns Ok. good stuff, microsoft
                        self.media_engine.TransferVideoFrame(
                            &self.bitmap,
                            None,
                            &bitmap_rect,
                            None,
                        ).unwrap();

                        self.copy_frame_to_texture();
                        self.display_frame();
                    },
                    Err(_) => {},
                }

                // check that the player hasn't run into an error
                match self.recv.try_recv() {
                    Ok(v) => match v {
                        NotifyMessage::Error => {
                            println!("Error!");
                            return;
                        }
                        _ => {
                            println!("got other message???");
                            return;
                        }
                    },
                    Err(_) => {},
                }

                if Self::check_break_event() {
                    is_breaking = true;
                }
            }
        }
    }

    fn copy_frame_to_texture(&self) {
        let wic_rect = WICRect {
            X: 0,
            Y: 0,
            Width: self.width as i32,
            Height: self.height as i32,
        };

        unsafe {
            let texture_lock: *const extern "stdcall" fn(*const std::ffi::c_void, u32, *mut D3D8LockedRect, *const [i32; 4], u32) -> u32
                = std::mem::transmute((*(self.texture as *const *const std::ffi::c_void)).byte_add(0x40));
            let texture_unlock: *const extern "stdcall" fn(*const std::ffi::c_void, u32) -> u32
                = std::mem::transmute((*(self.texture as *const *const std::ffi::c_void)).byte_add(0x44));

            let mut locked_rect = D3D8LockedRect {
                pitch: 0,
                bits: std::ptr::null_mut(),
            };

            if (*texture_lock)(
                self.texture,
                0,  // level 0
                &mut locked_rect,   // output pointer
                std::ptr::null(),   // null rect to get full texture
                0,  // no flags
            ) != 0 {
                println!("FAILED TO LOCK TEXTURE!");
                return;
            }

            let bits_slice = slice::from_raw_parts_mut(
                locked_rect.bits as *mut u8,
                (locked_rect.pitch as u32 * self.height) as usize,
            );

            self.bitmap.CopyPixels(
                &wic_rect,
                locked_rect.pitch as u32,
                bits_slice,
            ).unwrap();

            // TODO: fix format
            for row in 0..self.height {
                let row_offset = (locked_rect.pitch as u32 * row) as usize;

                let row_pixels = slice::from_raw_parts_mut(
                    locked_rect.bits.byte_add(row_offset) as *mut u32,
                    self.width as usize,
                );

                for pixel in row_pixels {
                    //*pixel = 0xff00ffff;
                }
            }

            if (*texture_unlock)(
                self.texture,
                0,  // level 0
            ) != 0 {
                println!("FAILED TO UNLOCK TEXTURE!");
                return;
            }
        }
    }

    fn display_frame(&self) {
        unsafe {
            let d3d8_device = *(0x00970e48 as *const *const std::ffi::c_void);

            let d3d8_clear: *const extern "stdcall" fn(*const std::ffi::c_void, u32, *const [i32; 4], u32, u32, f32, u32) -> u32
                = std::mem::transmute((*(d3d8_device as *const *const std::ffi::c_void)).byte_add(0x90));
            let d3d8_begin_scene: *const extern "stdcall" fn(*const std::ffi::c_void) -> u32
                = std::mem::transmute((*(d3d8_device as *const *const std::ffi::c_void)).byte_add(0x88));
            let d3d8_end_scene: *const extern "stdcall" fn(*const std::ffi::c_void) -> u32
                = std::mem::transmute((*(d3d8_device as *const *const std::ffi::c_void)).byte_add(0x8c));
            let d3d8_set_vertex_shader: *const extern "stdcall" fn(*const std::ffi::c_void, u32) -> u32
                = std::mem::transmute((*(d3d8_device as *const *const std::ffi::c_void)).byte_add(0x130));
            let d3d8_set_texture_stage_state: *const extern "stdcall" fn(*const std::ffi::c_void, u32, u32, u32) -> u32
                = std::mem::transmute((*(d3d8_device as *const *const std::ffi::c_void)).byte_add(0xfc));
            let d3d8_set_render_state: *const extern "stdcall" fn(*const std::ffi::c_void, u32, u32) -> u32
                = std::mem::transmute((*(d3d8_device as *const *const std::ffi::c_void)).byte_add(0xc8));
            let d3d8_set_texture: *const extern "stdcall" fn(*const std::ffi::c_void, u32, *const std::ffi::c_void) -> u32
                = std::mem::transmute((*(d3d8_device as *const *const std::ffi::c_void)).byte_add(0xf4));
            let d3d8_draw_primitive_up: *const extern "stdcall" fn(*const std::ffi::c_void, u32, u32, *const MovieVertex, u32) -> u32
                = std::mem::transmute((*(d3d8_device as *const *const std::ffi::c_void)).byte_add(0x120));
            let d3d8_present: *const extern "stdcall" fn(*const std::ffi::c_void, *const [i32; 4], *const [i32; 4], *const HWND, *const std::ffi::c_void) -> u32
                = std::mem::transmute((*(d3d8_device as *const *const std::ffi::c_void)).byte_add(0x3c));

            (*d3d8_clear)(
                d3d8_device,
                0, // no rectangles
                std::ptr::null(), // no rectangles
                1, // D3DCLEAR_TARGET
                0xff000000, // black clear color
                0.0, // z 0 (unused)
                0, // stencil 0 (unused)
            );
            
            if (*d3d8_begin_scene)(d3d8_device) != 0 {
                println!("Failed to begin scene!");
                return;
            }

            if (*d3d8_set_vertex_shader)(d3d8_device, 0x144) != 0 {
                println!("Failed to set shader!");
                return;
            }

            if (*d3d8_set_render_state)(
                d3d8_device,
                22,  // cull mode
                1,  // cull none
            ) != 0 {
                println!("Failed to set texture color op!");
                return;
            }

            if (*d3d8_set_texture_stage_state)(
                d3d8_device,
                0,  // stage 0
                1,  // color op
                4,  // modulate
            ) != 0 {
                println!("Failed to set texture color op!");
                return;
            }

            if (*d3d8_set_texture_stage_state)(
                d3d8_device,
                0,  // stage 0
                13,  // address u
                3,  // clamp
            ) != 0 {
                println!("Failed to set texture u address mode!");
                return;
            }

            if (*d3d8_set_texture_stage_state)(
                d3d8_device,
                0,  // stage 0
                14,  // address v
                3,  // clamp
            ) != 0 {
                println!("Failed to set texture v address mode!");
                return;
            }

            if (*d3d8_set_texture_stage_state)(
                d3d8_device,
                0,  // stage 0
                16,  // mag filter
                2,  // linear
            ) != 0 {
                println!("Failed to set texture magnify filter!");
                return;
            }

            if (*d3d8_set_texture)(d3d8_device, 0, self.texture) != 0 {
                println!("Failed to set texture!");
                return;
            }

            let (x, y) = window::get_window_size();

            // TODO: calculate correct video positioning/sizing

            let vertices = [
                MovieVertex {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                    color: 0xffffffff,
                    u: 0.0,
                    v: 0.0,
                },
                MovieVertex {
                    x: x as f32,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                    color: 0xffffffff,
                    u: 1.0,
                    v: 0.0,
                },
                MovieVertex {
                    x: 0.0,
                    y: y as f32,
                    z: 0.0,
                    w: 1.0,
                    color: 0xffffffff,
                    u: 0.0,
                    v: 1.0,
                },
                MovieVertex {
                    x: x as f32,
                    y: y as f32,
                    z: 0.0,
                    w: 1.0,
                    color: 0xffffffff,
                    u: 1.0,
                    v: 1.0,
                },
            ];

            if (*d3d8_draw_primitive_up)(
                d3d8_device,
                5, // triangle strip
                2, // two triangles
                vertices.as_ptr(),
                size_of::<MovieVertex>() as u32,
            ) != 0 {
                println!("Failed to draw!");
                return;
            }

            if (*d3d8_end_scene)(d3d8_device) != 0 {
                println!("Failed to end scene!");
                return;
            }

            if (*d3d8_present)(
                d3d8_device,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            ) != 0 {
                println!("Failed to present!");
                return;
            }
        };
    }

    fn check_break_event() -> bool {
        event::process_events();

        // TODO: handle exit (get Mlp::manager and check first member. if 1, exit)
        if is_exiting() {
            return true;
        }
        // TODO: handle input

        false
    }
}

#[repr(C)]
struct D3D8LockedRect {
    pitch: i32,
    bits: *mut std::ffi::c_void,
}

#[repr(C)]
struct MovieVertex {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    color: u32,
    u: f32,
    v: f32,
}

impl Drop for MoviePlayer {
    fn drop(&mut self) {
        unsafe {
            let texture_release: *const extern "stdcall" fn(*const std::ffi::c_void) -> u32
                = std::mem::transmute((*(self.texture as *const *const std::ffi::c_void)).byte_add(0x08));

            let _ = (*texture_release)(self.texture);
        };

        let _ = unsafe {
            self.media_engine.Shutdown()
        };
    }
}

// struct that exists solely to give us RAII on COM and Media Framework
struct MfCom(());

impl MfCom {
    fn new() -> Option<Self> {
        unsafe {
            let coinit_result = CoInitialize(None);
            
            if coinit_result.is_err() {
                return None;
            };

            if let Err(e) = MFStartup(MF_VERSION, 0) {
                return None;
            };
        }

        Some(MfCom(()))
    }
}

impl Drop for MfCom {
    fn drop(&mut self) {
        unsafe {
            let _ = MFShutdown();
            CoUninitialize();
        }
    }
}

enum NotifyMessage {
    LoadReady,
    PlaybackReady,
    Error,
}

#[implement(IMFMediaEngineNotify)]
struct MediaEngineNotify {
    // TODO: probably add some sort of value we can give back to the main thread so we know we can load etc.
    sender: std::sync::mpsc::Sender<NotifyMessage>,
}

impl IMFMediaEngineNotify_Impl for MediaEngineNotify_Impl {
    fn EventNotify(&self, event: u32, param1: usize, param2: u32) -> windows::core::Result<()> {
        if event == MF_MEDIA_ENGINE_EVENT_LOADSTART.0 as u32 {
            println!("LOAD START");
            self.sender.send(NotifyMessage::LoadReady).unwrap();
        } else if event == MF_MEDIA_ENGINE_EVENT_CANPLAY.0 as u32 {
            println!("CAN PLAY");
            self.sender.send(NotifyMessage::PlaybackReady).unwrap();
        } else if event == MF_MEDIA_ENGINE_EVENT_ERROR.0 as u32 {
            println!("ERROR");
            self.sender.send(NotifyMessage::Error).unwrap();
        } else {
            //println!("GOT EVENT: {}", event);
        }
        
        Ok(())
    }
}

fn play_movie(path: &str) {
    let prefix_cstr = unsafe {
        let get_path_prefix: extern "C" fn() -> *const std::ffi::c_char
            = std::mem::transmute(0x004032d0);

        std::ffi::CStr::from_ptr(get_path_prefix())
    };

    let prefix = prefix_cstr.to_string_lossy();
    let fullpath = format!("{}{}.mpg", prefix, path);

    println!("Playing {}", &fullpath);
    if let Some(mut player) = MoviePlayer::new(&fullpath) {
        // stop all streams
        unsafe {
            let close_streams_1: extern "C" fn()
                = std::mem::transmute(0x004c5c90);
            let close_streams_2: extern "C" fn(i32)
                = std::mem::transmute(0x004c5cb0);

            close_streams_1();
            close_streams_2(-1);
        }

        player.play();
    }
}

fn is_exiting() -> bool {
    unsafe {
        let get_mainloop_manager: extern "C" fn(bool) -> *const std::ffi::c_char
            = std::mem::transmute(0x004c02b0);
        let release_mainloop_manager: extern "thiscall" fn(*const std::ffi::c_char)
            = std::mem::transmute(0x004c0300);
        
        let mainloop_manager = get_mainloop_manager(false);

        let result = if !mainloop_manager.is_null() {
            (*mainloop_manager) != 0
        } else {
            false
        };

        release_mainloop_manager(mainloop_manager);

        result
    }
}

unsafe extern "C" fn intro_play_movie(path: *const std::ffi::c_char, unk: u32) -> u32 {
    let cstr = unsafe {
        std::ffi::CStr::from_ptr(path)
    };

    let native_path = cstr.to_string_lossy();

    play_movie(&native_path);
    
    if is_exiting() {
        // there is very slow deinit code after this, so just skip that and exit
        std::process::exit(0);
    } else {
        1
    }
}

unsafe extern "C" fn script_play_movie(path: *const std::ffi::c_char) {
    let cstr = unsafe {
        std::ffi::CStr::from_ptr(path)
    };

    let native_path = cstr.to_string_lossy();
    play_movie(&native_path);
}

pub unsafe fn patch() {
    unsafe {
        patch::patch_jmp(0x0040a1a0 as *mut (), intro_play_movie as *const ());
        patch::patch_jmp(0x004c7420 as *mut (), script_play_movie as *const ());
    }
}

