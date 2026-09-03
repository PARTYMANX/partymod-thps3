use partymod_common::{logger::LogLevel, patch, syncunsafecell::SyncUnsafeCell};

use crate::{config, gameplay, logger};

pub struct NetContext {
    game_version: u32, // version number for checking client/server compatibility

    peerchat_url: Option<std::ffi::CString>,
    master_server_url: Option<std::ffi::CString>,

    their_bind: Option<unsafe extern "stdcall" fn(*const (), *mut SocketAddressIn, i32) -> i32>,
}

unsafe impl Sync for NetContext {}
unsafe impl Send for NetContext {}

pub static NET_CONTEXT: SyncUnsafeCell<Option<NetContext>> = SyncUnsafeCell::new(None);

pub fn init() {
    unsafe {
        let game_version = match gameplay::get_compatibility_mode() {
            true => 0x00010001u32,  // 1.01
            false => 0x00020000u32, // 2.00
        };

        let version_bytes = game_version.to_le_bytes();
        let version_major = ((version_bytes[3] as u32) * 10) + version_bytes[2] as u32;
        let version_minor = ((version_bytes[1] as u32) * 10) + version_bytes[0] as u32;

        logger::log(
            LogLevel::Info,
            &format!("Game version: {}.{:02}", version_major, version_minor),
        );

        let ctx = &mut *NET_CONTEXT.get();
        *ctx = Some(NetContext {
            game_version,

            peerchat_url: None,
            master_server_url: None,

            their_bind: None,
        });

        patch_online_service(ctx.as_mut().unwrap());
        patch_bind(ctx.as_mut().unwrap());
    }
}

fn patch_online_service(net_ctx: &mut NetContext) {
    let domain = config::get_string("Miscellaneous", "OnlineDomain", "openspy.net");

    logger::log(
        LogLevel::Info,
        &format!("Patching online server: {}", domain),
    );

    let peerchat = match std::ffi::CString::new(format!("peerchat.{}", domain)) {
        Ok(v) => v,
        Err(e) => {
            logger::log(
                LogLevel::Error,
                &format!("Failed to create peerchat server URL: {}", e),
            );
            return;
        }
    };

    let master_server = match std::ffi::CString::new(format!("master.{}", domain)) {
        Ok(v) => v,
        Err(e) => {
            logger::log(
                LogLevel::Error,
                &format!("Failed to create master server URL: {}", e),
            );
            return;
        }
    };

    unsafe {
        patch::patch_u32(
            (0x0050b278 + 1) as *mut (),
            peerchat.as_c_str().as_ptr() as u32,
        );
        patch::patch_u32(
            (0x00517845 + 1) as *mut (),
            master_server.as_c_str().as_ptr() as u32,
        );
        patch::patch_u32(
            (0x0051785d + 1) as *mut (),
            master_server.as_c_str().as_ptr() as u32,
        );
        patch::patch_u32(
            (0x0051959a + 1) as *mut (),
            master_server.as_c_str().as_ptr() as u32,
        );
    }

    net_ctx.peerchat_url = Some(peerchat);
    net_ctx.master_server_url = Some(master_server);
}

pub fn get_server_version() -> u32 {
    let net_ctx = unsafe {
        match &*NET_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized net context!"),
        }
    };

    net_ctx.game_version
}

#[repr(C)]
struct SocketAddressIn {
    sin_family: i16,
    sin_port: i16,
    sin_addr: u32,
    sin_zero: [u8; 8],
}

unsafe extern "stdcall" fn bind_wrapper(
    socket: *const (),
    address: *mut SocketAddressIn,
    namelen: i32,
) -> i32 {
    let net_ctx = unsafe {
        match &*NET_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized net context!"),
        }
    };

    unsafe {
        (*address).sin_addr = 0;
    }

    match net_ctx.their_bind {
        Some(their_bind) => unsafe { (their_bind)(socket, address, namelen) },
        None => panic!("Tried to call bind() wrapper without bind() being initialized!"),
    }
}

pub unsafe fn patch_bind(net_ctx: &mut NetContext) {
    unsafe {
        let bind_addr_base = (0x00519500 + 1) as *mut ();

        let their_bind_addr = bind_addr_base.byte_add(4 + *(bind_addr_base as *const usize));
        let their_bind: unsafe extern "stdcall" fn(*const (), *mut SocketAddressIn, i32) -> i32 =
            std::mem::transmute(their_bind_addr);

        net_ctx.their_bind = Some(their_bind);

        patch::patch_call(0x004d9f3e as *mut (), bind_wrapper as *const ());
        patch::patch_call(0x004d9f75 as *mut (), bind_wrapper as *const ());
        patch::patch_call(0x00518a32 as *mut (), bind_wrapper as *const ());
        patch::patch_call(0x00519500 as *mut (), bind_wrapper as *const ());
    }
}
