use std::sync::mpsc::Receiver;

use partymod_common::patch;
use windows::{Win32::{Foundation::RECT, Graphics::Imaging::{CLSID_WICImagingFactory, GUID_WICPixelFormat24bppBGR, GUID_WICPixelFormat24bppRGB, GUID_WICPixelFormat32bppBGRA, GUID_WICPixelFormat32bppPBGRA, IWICBitmap, IWICImagingFactory, WICBitmapCacheOnDemand, WICRect}, Media::MediaFoundation::{CLSID_MFMediaEngineClassFactory, IMFMediaEngine, IMFMediaEngineClassFactory, IMFMediaEngineNotify, IMFMediaEngineNotify_Impl, MF_MEDIA_ENGINE_CALLBACK, MF_MEDIA_ENGINE_DXGI_MANAGER, MF_MEDIA_ENGINE_EVENT_CANPLAY, MF_MEDIA_ENGINE_EVENT_ERROR, MF_MEDIA_ENGINE_EVENT_LOADSTART, MF_MEDIA_ENGINE_EVENT_NOTIFYSTABLESTATE, MF_MEDIA_ENGINE_PLAYBACK_HWND, MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT, MF_MEDIA_ENGINE_WAITFORSTABLE_STATE, MF_VERSION, MFARGB, MFCreateAttributes, MFShutdown, MFStartup, MFVideoNormalizedRect}, System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoInitialize, CoUninitialize}}, core::{BSTR, w}};
use windows_core::{ComObjectInner, IUnknown, implement};

struct MoviePlayer {
    mf_com: MfCom,
    wic_factory: IWICImagingFactory,
    media_engine_factory: IMFMediaEngineClassFactory,

    recv: Receiver<NotifyMessage>,
    media_engine: IMFMediaEngine,
    bitmap: IWICBitmap,

    width: u32,
    height: u32,
    copy_buf: Vec<u8>,
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

        let mut copy_buf = Vec::new();
        copy_buf.resize((4 * width * height) as usize, 0u8);

        Some(Self {
            mf_com,
            wic_factory,
            media_engine_factory,
            recv,
            media_engine,
            bitmap,
            width,
            height,
            copy_buf,
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

        let wic_rect = WICRect {
            X: 0,
            Y: 0,
            Width: self.width as i32,
            Height: self.height as i32,
        };

        unsafe {
            if let Err(e) = self.media_engine.Play() {
                return;
            }

            let mut is_breaking = false;
            while !is_breaking && !self.media_engine.IsEnded().as_bool() {
                // throttle here

                match self.media_engine.OnVideoStreamTick() {
                    Ok(v) => {
                        // documentation says this should only return Ok if there is a new frame.
                        // in reality, this always returns Ok. good stuff, microsoft
                        self.media_engine.TransferVideoFrame(
                            &self.bitmap,
                            None,
                            &bitmap_rect,
                            None,
                        ).unwrap();

                        // copy to texture
                        self.bitmap.CopyPixels(
                            &wic_rect,
                            self.width * 4,
                            &mut self.copy_buf,
                        ).unwrap();

                        // display
                    },
                    Err(_) => {},
                }

                if Self::check_break_event() {
                    is_breaking = true;
                }
            }
        }
    }

    fn check_break_event() -> bool {
        false
    }
}

impl Drop for MoviePlayer {
    fn drop(&mut self) {
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
            // TODO: handle error (likely file not found)
        } else {
            println!("GOT EVENT: {}", event);
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
        println!("ACTUALLY PLAYING");

        player.play();
    }
}

unsafe extern "C" fn intro_play_movie(path: *const std::ffi::c_char, unk: u32) -> u32 {
    let cstr = unsafe {
        std::ffi::CStr::from_ptr(path)
    };

    let native_path = cstr.to_string_lossy();

    play_movie(&native_path);
    return 1;
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

