use std::arch::asm;

use partymod_common::{patch, syncunsafecell::SyncUnsafeCell};

use crate::config;

struct GameplayContext {
    compatibility_mode: bool,
}

static GAMEPLAY_CONTEXT: SyncUnsafeCell<Option<GameplayContext>> = SyncUnsafeCell::new(None);

unsafe extern "C" fn ledge_warp_acos() {
    unsafe {
        // clamps st(0) to [-1, 1]
        // expects the floating point argument to be passed in via the x87 stack
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
        let orig_acos: extern "C" fn() = std::mem::transmute(0x00577cdc);

        orig_acos()
    }
}

unsafe fn patch_ledge_warp() {
    unsafe {
        patch::patch_call(0x0049f1dc as *mut (), ledge_warp_acos as *const ());
    }
}

pub fn init() {
    let compatibility_mode = config::get_bool("Miscellaneous", "CompatibilityMode", false);

    if !compatibility_mode {
        unsafe {
            patch_trick_limit();
            patch_tag_limit();
        }
    }

    unsafe {
        let ctx = &mut *GAMEPLAY_CONTEXT.get();
        *ctx = Some(GameplayContext { compatibility_mode });
    }
}

pub fn get_compatibility_mode() -> bool {
    unsafe {
        match &mut *GAMEPLAY_CONTEXT.get() {
            Some(v) => v.compatibility_mode,
            None => panic!("Tried to use uninitialized gameplay context!"),
        }
    }
}

unsafe fn patch_trick_limit() {
    unsafe {
        patch::patch_byte(0x004355ad as *mut (), 0xeb);
    }
}

const MAX_PENDING_TRICKS: usize = 512;

#[repr(C)]
struct FixedPendingTricks {
    checksums: [u32; MAX_PENDING_TRICKS],
    trick_count: u32,
}

extern "thiscall" fn pending_tricks_constructor(this: *mut Box<FixedPendingTricks>) {
    unsafe {
        *this = Box::new(FixedPendingTricks {
            checksums: [0; MAX_PENDING_TRICKS],
            trick_count: 0,
        });
    }
}

extern "thiscall" fn pending_tricks_flush_tricks(this: *mut Box<FixedPendingTricks>) -> bool {
    unsafe {
        (*this).trick_count = 0;
    }

    true
}

extern "thiscall" fn pending_tricks_trick_off_object_wrapper(
    this: *mut Box<FixedPendingTricks>,
    obj: u32,
) -> u32 {
    unsafe {
        let orig_func: unsafe extern "thiscall" fn(*const FixedPendingTricks, u32) -> u32 =
            std::mem::transmute(0x004bfc60);

        if (*this).trick_count > MAX_PENDING_TRICKS as u32 {
            (*this).trick_count = MAX_PENDING_TRICKS as u32;
        }

        let result = orig_func((*this).as_ref(), obj);

        result
    }
}

extern "thiscall" fn pending_tricks_write_to_buffer_wrapper(
    this: *mut Box<FixedPendingTricks>,
    buf: *const (),
    size: u32,
) -> u32 {
    unsafe {
        let orig_func: unsafe extern "thiscall" fn(
            *const FixedPendingTricks,
            *const (),
            u32,
        ) -> u32 = std::mem::transmute(0x004bfd40);

        if (*this).trick_count > MAX_PENDING_TRICKS as u32 {
            (*this).trick_count = MAX_PENDING_TRICKS as u32;
        }

        orig_func((*this).as_ref(), buf, size)
    }
}

unsafe fn patch_tag_limit() {
    unsafe {
        let max_size = (size_of::<u32>() * MAX_PENDING_TRICKS) as u32;

        // replace CPendingTricks::CPendingTricks()
        patch::patch_jmp(
            0x004bfc50 as *mut (),
            pending_tricks_constructor as *const (),
        );

        // replace CPendingTricks::FlushTricks()
        patch::patch_jmp(
            0x004bfe10 as *mut (),
            pending_tricks_flush_tricks as *const (),
        );

        // wrap CPendingTricks::TrickOffObject()
        patch::patch_jmp(
            0x004b5260 as *mut (),
            pending_tricks_trick_off_object_wrapper as *const (),
        );
        // remove AND to prevent wrapping
        // note: this is unsafe but also we're never going to see more than 512 objects
        patch::patch_nop(0x004bfcef as *mut (), 3);
        // skip bounds check
        patch::patch_byte(0x004bfc86 as *mut (), 0xeb);
        // fix offsets to trick_count
        patch::patch_u32(
            (0x004bfc7d + 2) as *mut (),
            (size_of::<u32>() * MAX_PENDING_TRICKS) as u32,
        );
        patch::patch_u32(
            (0x004bfce5 + 2) as *mut (),
            (size_of::<u32>() * MAX_PENDING_TRICKS) as u32,
        );
        patch::patch_u32(
            (0x004bfcf5 + 2) as *mut (),
            (size_of::<u32>() * MAX_PENDING_TRICKS) as u32,
        );
        patch::patch_u32(
            (0x004bfcfc + 2) as *mut (),
            (size_of::<u32>() * MAX_PENDING_TRICKS) as u32,
        );

        // wrap CPendingTricks::WriteToBuffer()
        patch::patch_call(
            0x004b51f4 as *mut (),
            pending_tricks_write_to_buffer_wrapper as *const (),
        );
        // skip bounds check
        patch::patch_byte(0x004bfd67 as *mut (), 0xeb);
        // fix offset to trick_count
        patch::patch_u32(
            (0x004bfd5e + 2) as *mut (),
            (size_of::<u32>() * MAX_PENDING_TRICKS) as u32,
        );

        // extend request size in Score::LogTrickObjectRequest
        // extend stack allocation
        patch::patch_u32((0x004359b5 + 2) as *mut (), max_size + 0x20);
        // stack addition at end
        patch::patch_u32((0x00435ad0 + 2) as *mut (), max_size + 0x2c);
        // fix write buffer size
        patch::patch_u32((0x00435a50 + 1) as *mut (), max_size);
        // fix message size
        patch::patch_u32((0x00435a84 + 1) as *mut (), max_size);
        // fix stack offsets
        patch::patch_u32((0x004359de + 3) as *mut (), max_size + (0xb8 - 0x80));
        patch::patch_u32((0x00435a04 + 3) as *mut (), max_size + (0xbc - 0x80));
        patch::patch_u32((0x00435a3c + 3) as *mut (), max_size + (0xb4 - 0x80));
        patch::patch_u32((0x00435a9a + 3) as *mut (), max_size + (0xb4 - 0x80));
        patch::patch_u32((0x00435aaf + 3) as *mut (), max_size + (0xb4 - 0x80));
        patch::patch_u32((0x00435abf + 3) as *mut (), max_size + (0xac - 0x80));

        // Score::LogTrickObject
        // extend message size
        // disabled because it may cause other issues as in THPS4
        // patch::patch_u32((0x00435cfe + 1) as *mut (), max_size);
        // patch::patch_u32((0x00435dab + 1) as *mut (), max_size);
        // extend stack allocation
        patch::patch_u32((0x00435af5 + 2) as *mut (), (max_size * 2) + 0x58);
        // add back the stack
        patch::patch_u32((0x00435e9b + 2) as *mut (), (max_size * 2) + 0x64);
        // fix stack offsets
        // parameters
        patch::patch_u32(
            (0x00435b4e + 3) as *mut (),
            (max_size * 2) + (0x178 - 0x100),
        );
        patch::patch_u32(
            (0x00435b7b + 3) as *mut (),
            (max_size * 2) + (0x17c - 0x100),
        );
        patch::patch_u32(
            (0x00435b89 + 3) as *mut (),
            (max_size * 2) + (0x184 - 0x100),
        );
        patch::patch_u32(
            (0x00435b98 + 3) as *mut (),
            (max_size * 2) + (0x188 - 0x100),
        );
        patch::patch_u32(
            (0x00435bb7 + 3) as *mut (),
            (max_size * 2) + (0x180 - 0x100),
        );
        patch::patch_u32(
            (0x00435c8d + 3) as *mut (),
            (max_size * 2) + (0x17c - 0x100),
        );
        patch::patch_u32(
            (0x00435c94 + 3) as *mut (),
            (max_size * 2) + (0x180 - 0x100),
        );
        patch::patch_u32(
            (0x00435cb7 + 3) as *mut (),
            (max_size * 2) + (0x184 - 0x100),
        );
        patch::patch_u32(
            (0x00435d2a + 3) as *mut (),
            (max_size * 2) + (0x180 - 0x100),
        );
        patch::patch_u32(
            (0x00435d4d + 3) as *mut (),
            (max_size * 2) + (0x184 - 0x100),
        );
        patch::patch_u32(
            (0x00435e2a + 3) as *mut (),
            (max_size * 2) + (0x180 - 0x100),
        );
        patch::patch_u32(
            (0x00435e35 + 3) as *mut (),
            (max_size * 2) + (0x184 - 0x100),
        );
        // locals - low
        patch::patch_u32(
            (0x00435b1e + 3) as *mut (),
            (max_size * 2) + (0x178 - 0x100),
        );
        patch::patch_u32(
            (0x00435b47 + 3) as *mut (),
            (max_size * 2) + (0x188 - 0x100),
        );
        patch::patch_u32(
            (0x00435b57 + 3) as *mut (),
            (max_size * 2) + (0x170 - 0x100),
        );
        patch::patch_u32(
            (0x00435bd0 + 3) as *mut (),
            (max_size * 2) + (0x170 - 0x100),
        );
        patch::patch_u32(
            (0x00435be5 + 3) as *mut (),
            (max_size * 2) + (0x170 - 0x100),
        );
        patch::patch_u32(
            (0x00435e04 + 3) as *mut (),
            (max_size * 2) + (0x170 - 0x100),
        );
        patch::patch_u32(
            (0x00435e19 + 3) as *mut (),
            (max_size * 2) + (0x170 - 0x100),
        );
        patch::patch_u32(
            (0x00435e64 + 3) as *mut (),
            (max_size * 2) + (0x170 - 0x100),
        );
        patch::patch_u32(
            (0x00435e79 + 3) as *mut (),
            (max_size * 2) + (0x170 - 0x100),
        );
        patch::patch_u32(
            (0x00435e89 + 3) as *mut (),
            (max_size * 2) + (0x168 - 0x100),
        );
        // locals - between message buffers
        patch::patch_u32((0x00435d31 + 3) as *mut (), max_size + (0xe0 - 0x80));
        patch::patch_u32((0x00435d38 + 3) as *mut (), max_size + (0xe4 - 0x80));
        patch::patch_u32((0x00435d46 + 3) as *mut (), max_size + (0xe4 - 0x80));
        patch::patch_u32((0x00435d54 + 3) as *mut (), max_size + (0xdc - 0x80));
        patch::patch_u32((0x00435d5b + 3) as *mut (), max_size + (0xe8 - 0x80));
        patch::patch_u32((0x00435da4 + 3) as *mut (), max_size + (0xe8 - 0x80));
    }
}

pub unsafe fn patch() {
    unsafe {
        patch_ledge_warp();
    }
}
