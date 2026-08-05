use core::ffi;

use raw_window_handle::HasWindowHandle;
use sdl3::{render::create_renderer, sys::pixels::SDL_PixelFormat};
use windows::{Win32::{Foundation::RECT, Graphics::Imaging::{CLSID_WICImagingFactory, GUID_WICPixelFormat24bppBGR, GUID_WICPixelFormat24bppRGB, GUID_WICPixelFormat32bppBGRA, GUID_WICPixelFormat32bppPBGRA, IWICImagingFactory, WICBitmapCacheOnDemand, WICRect}, Media::MediaFoundation::{CLSID_MFMediaEngineClassFactory, IMFMediaEngine, IMFMediaEngineClassFactory, IMFMediaEngineNotify, IMFMediaEngineNotify_Impl, MF_MEDIA_ENGINE_CALLBACK, MF_MEDIA_ENGINE_DXGI_MANAGER, MF_MEDIA_ENGINE_EVENT_CANPLAY, MF_MEDIA_ENGINE_EVENT_LOADSTART, MF_MEDIA_ENGINE_EVENT_NOTIFYSTABLESTATE, MF_MEDIA_ENGINE_PLAYBACK_HWND, MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT, MF_MEDIA_ENGINE_WAITFORSTABLE_STATE, MF_VERSION, MFARGB, MFCreateAttributes, MFShutdown, MFStartup, MFVideoNormalizedRect}, System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoInitialize, CoUninitialize}}, core::{BSTR, w}};
use windows_core::{ComObjectInner, IUnknown, implement};

use crate::NotifyMessage::LoadReady;

fn main() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Video Fun!", 1920, 1080)
        .position_centered()
        .resizable()
        .high_pixel_density()
        .build()
        .unwrap();

    let mut renderer = create_renderer(window, None).unwrap();

    /*let hwnd = match window.window_handle().unwrap().as_raw() {
        raw_window_handle::RawWindowHandle::Win32(handle) => {
            handle.hwnd.get()
        }
        _ => unreachable!("Got non-windows window handle!"),
    };*/

    unsafe {
        CoInitialize(None).unwrap();
        MFStartup(MF_VERSION, 0).unwrap();
    }

    let wic_factory: IWICImagingFactory = unsafe {
        CoCreateInstance(
            &CLSID_WICImagingFactory,
            None,
            CLSCTX_INPROC_SERVER,
        ).unwrap()
    };

    let media_engine_factory: IMFMediaEngineClassFactory = unsafe { 
        CoCreateInstance(
            &CLSID_MFMediaEngineClassFactory,
            None,
            CLSCTX_INPROC_SERVER,
        ).unwrap()
    };

    let attributes = unsafe {
        let mut attr = None;
        MFCreateAttributes(&mut attr, 1).unwrap();

        attr.unwrap()
    };

    let (sender, recv) = std::sync::mpsc::channel();

    let notify: IUnknown = MediaEngineNotify{
        sender,
    }.into();

    unsafe {
        attributes.SetUnknown(&MF_MEDIA_ENGINE_CALLBACK, &notify).unwrap();
        //attributes.SetUINT64(&MF_MEDIA_ENGINE_PLAYBACK_HWND, hwnd as u64).unwrap();
    }
    

    let media_engine = unsafe { 
        media_engine_factory.CreateInstance(
            //MF_MEDIA_ENGINE_WAITFORSTABLE_STATE.0 as u32,
            0,
            &attributes,
        ).unwrap()
    };

    unsafe {
        let mut is_breaking = false;
        let mut event_pump = sdl_context.event_pump().unwrap();

        //media_engine.SetAutoPlay(true).unwrap();
        media_engine.SetSource(&BSTR::from_wide(w!("C:/Games/thps3exp/Data/MOVIES/Day.mpg").as_wide())).unwrap();
        media_engine.Load().unwrap();

        let src = media_engine.GetCurrentSource().unwrap();
        println!("SOURCE: {}", src.to_string());
        //media_engine.Play().unwrap();

        // wait for load or break event
        while !is_breaking {
            match recv.recv_timeout(std::time::Duration::from_secs_f64(1.0/60.0)) {
                Ok(v) => match v {
                    NotifyMessage::LoadReady => {
                        println!("LOAD START RECEIVED");
                        break;
                    },
                    NotifyMessage::PlaybackReady => todo!(),
                },
                Err(_) => {
                    if check_break_event(&mut event_pump) != 0 {
                        is_breaking = true;
                    }
                },
            }
        }

        // wait for load or break event
        while !is_breaking {
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
                    if check_break_event(&mut event_pump) != 0 {
                        is_breaking = true;
                    }
                },
            }
        }

        let mut width = 0;
        let mut height = 0;
        media_engine.GetNativeVideoSize(Some(&mut width), Some(&mut height)).unwrap();
        //media_engine.Get

        let bitmap = wic_factory.CreateBitmap(
            width,
            height,
            &GUID_WICPixelFormat32bppBGRA,
            WICBitmapCacheOnDemand
        ).unwrap();
        let bitmap_rect = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };

        let src_rect = MFVideoNormalizedRect {
            left: 0.0,
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
        };

        let border_color = MFARGB {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbAlpha: 255,
        };

        let wic_rect = WICRect {
            X: 0,
            Y: 0,
            Width: width as i32,
            Height: height as i32,
        };

        let texture_creator = renderer.texture_creator();
        let mut texture = texture_creator.create_texture(
            sdl3::pixels::PixelFormat::BGRA32,
            sdl3::render::TextureAccess::Streaming,
            width,
            height,
        ).unwrap();

        let mut copy_buf = Vec::new();
        copy_buf.resize((4 * width * height) as usize, 0u8);

        media_engine.Play().unwrap();

        // make sure video has stuff to play
        println!("HASAUDIO: {}, HASVIDEO: {}", media_engine.HasVideo().as_bool(), media_engine.HasAudio().as_bool());

        if media_engine.HasVideo().as_bool() || media_engine.HasAudio().as_bool() {
            while !is_breaking && !media_engine.IsEnded().as_bool() {
                // throttle here
                media_engine.GetCurrentTime();

                match media_engine.OnVideoStreamTick() {
                    Ok(v) => {
                        media_engine.TransferVideoFrame(
                            &bitmap,
                            Some(&src_rect),
                            &bitmap_rect,
                            Some(&border_color)
                        ).unwrap();

                        bitmap.CopyPixels(&wic_rect, width * 4, &mut copy_buf).unwrap();

                        texture.update(None, &copy_buf, (width * 4) as usize).unwrap();

                        renderer.set_draw_color(sdl3::pixels::Color {
                            r: 0,
                            g: 0,
                            b: 255,
                            a: 255,
                        });
                        renderer.clear();
                        renderer.copy(&texture, None, None).unwrap();
                        renderer.present();

                        //println!("OK {}", v);
                    },
                    Err(_) => {},
                }

                if check_break_event(&mut event_pump) != 0 {
                    is_breaking = true;
                }
            }
        }
    }

    unsafe {
        media_engine.Shutdown().unwrap();
        MFShutdown().unwrap();
        CoUninitialize();
    }
    
}

fn check_break_event(event_pump: &mut sdl3::EventPump) -> u32 {
    let mut result = 0;

    for event in event_pump.poll_iter() {
        match event {
            sdl3::event::Event::Quit { .. }
            | sdl3::event::Event::KeyDown {
                keycode: Some(sdl3::keyboard::Keycode::Escape),
                ..
            } => {
                result = 1;
            }
            _ => {}
        }
    }

    result
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
        } else {
            println!("GOT EVENT: {}", event);
        }
        
        Ok(())
    }
}