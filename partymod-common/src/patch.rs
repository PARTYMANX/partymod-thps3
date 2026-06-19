pub unsafe fn patch_call(addr: *mut (), func: *const ()) {
    unsafe {
        let mut old_protect = 0;
        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        );

        std::ptr::write(addr as *mut u8, 0xE8);
        std::ptr::write(
            addr.byte_offset(1) as *mut u32,
            func.byte_offset(-(addr as isize) - 5) as u32,
        );

        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            old_protect,
            std::ptr::null_mut(),
        );
    }
}

pub unsafe fn patch_jmp(addr: *mut (), func: *const ()) {
    unsafe {
        let mut old_protect = 0;
        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        );

        std::ptr::write(addr as *mut u8, 0xE9);
        std::ptr::write(
            addr.byte_offset(1) as *mut u32,
            func.byte_offset(-(addr as isize) - 5) as u32,
        );

        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            old_protect,
            std::ptr::null_mut(),
        );
    }
}

pub unsafe fn patch_byte(addr: *mut (), val: u8) {
    unsafe {
        let mut old_protect = 0;
        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        );

        std::ptr::write(addr as *mut u8, val);

        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            old_protect,
            std::ptr::null_mut(),
        );
    }
}

pub unsafe fn patch_bytes(addr: *mut (), bytes: &[u8]) {
    unsafe {
        let mut old_protect = 0;
        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        );

        for (i, val) in bytes.iter().enumerate() {
            std::ptr::write((addr as *mut u8).byte_offset(i as isize), *val);
        }

        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            old_protect,
            std::ptr::null_mut(),
        );
    }
}

pub unsafe fn patch_u32(addr: *mut (), val: u32) {
    unsafe {
        let mut old_protect = 0;
        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        );

        std::ptr::write(addr as *mut u32, val);

        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            old_protect,
            std::ptr::null_mut(),
        );
    }
}

pub unsafe fn patch_f32(addr: *mut (), val: f32) {
    unsafe {
        let mut old_protect = 0;
        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        );

        std::ptr::write(addr as *mut f32, val);

        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            old_protect,
            std::ptr::null_mut(),
        );
    }
}

pub unsafe fn patch_nop(addr: *mut (), count: usize) {
    unsafe {
        let mut old_protect = 0;
        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        );

        std::ptr::write_bytes(addr as *mut u8, 0x90, count);

        windows_sys::Win32::System::Memory::VirtualProtect(
            addr as *mut std::ffi::c_void,
            5,
            old_protect,
            std::ptr::null_mut(),
        );
    }
}
