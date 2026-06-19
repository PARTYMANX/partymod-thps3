use std::arch::asm;

use partymod_common::patch;

unsafe extern "C" fn ledge_warp_acos() {
    unsafe {
        // clamps st(0) to [-1, 1]
        asm!(
            "ftst",
            "jl 2f",
            "fld1",
            "fcom",
            "fstp st(0)",
            "jle 3f",
            "fstp st(0)",
            "fld1",
            "jmp 3f",
            "2:",
            "fchs",
            "fld1",
            "fcom",
            "fstp st(0)",
            "fchs",
            "jle 3f",
            "fstp st(0)",
            "fld1",
            "fchs",
            "3:",
        );

        // call acos
        let orig_acos: extern "C" fn()
            = std::mem::transmute(0x00577cdc);

        orig_acos()
    }
}

unsafe fn patch_ledge_warp() {
    unsafe {
        patch::patch_call(0x0049f1dc as *mut (), ledge_warp_acos as *const ());
    }
}

// TODO: trick limit, tag limit

pub unsafe fn patch() {
    unsafe {
        patch_ledge_warp();
    }
}