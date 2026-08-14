use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use partymod_common::{
    bps::bps_patch_checked, logger::LogLevel, patch, syncunsafecell::SyncUnsafeCell,
};

use crate::logger;

pub struct FileContext {
    patches: HashMap<String, Vec<u8>>,
}

pub static FILE_CONTEXT: SyncUnsafeCell<Option<FileContext>> = SyncUnsafeCell::new(None);

pub fn init() {
    unsafe {
        let ctx = &mut *FILE_CONTEXT.get();
        *ctx = Some(FileContext {
            patches: HashMap::new(),
        });
    }
}

pub fn register_patch(path: &str, patch: &[u8]) {
    let file_context = match unsafe { &mut *FILE_CONTEXT.get() } {
        Some(v) => v,
        None => panic!("Tried to use uninitialized file context!"),
    };

    file_context
        .patches
        .insert(path.to_string().to_lowercase(), patch.to_vec());
}

struct PatchedFile {
    cursor: usize,
    buffer: Vec<u8>,
}

enum PatchableFile {
    Unpatched(std::fs::File),
    Patched(PatchedFile),
}

impl PatchableFile {
    fn open(path: &Path, opts: OpenOptions) -> Option<Box<Self>> {
        println!("OPENING FILE {}!", path.to_string_lossy());

        let mut f = match opts.open(path) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let file_context = match unsafe { &*FILE_CONTEXT.get() } {
            Some(v) => v,
            None => panic!("Tried to use uninitialized file context!"),
        };

        let path_str = match path.to_str() {
            Some(v) => v,
            None => {
                logger::log(LogLevel::Error, &format!("Failed to get path???"));
                return Some(Box::new(PatchableFile::Unpatched(f)));
            }
        };

        match file_context.patches.get(&path_str.to_lowercase()) {
            Some(patch_buf) => {
                logger::log(LogLevel::Info, &format!("Patching {}...", path_str));

                let mut buf = Vec::new();
                if let Err(e) = f.read_to_end(&mut buf) {
                    logger::log(LogLevel::Error, &format!("Failed to read file: {}", e));
                    let _ = f.seek(SeekFrom::Start(0));
                    return Some(Box::new(PatchableFile::Unpatched(f)));
                }

                match bps_patch_checked(&buf, patch_buf) {
                    Ok(v) => {
                        logger::log(LogLevel::Info, &format!("Patch successful!"));
                        Some(Box::new(PatchableFile::Patched(PatchedFile {
                            cursor: 0,
                            buffer: v,
                        })))
                    }
                    Err(e) => {
                        logger::log(LogLevel::Error, &format!("Failed to patch file: {}", e));
                        let _ = f.seek(SeekFrom::Start(0));
                        Some(Box::new(PatchableFile::Unpatched(f)))
                    }
                }
            }
            None => Some(Box::new(PatchableFile::Unpatched(f))),
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        match self {
            PatchableFile::Unpatched(f) => match f.read(buf) {
                Ok(v) => v,
                Err(_) => 0,
            },
            PatchableFile::Patched(f) => {
                let start = f.cursor;
                let end = usize::min(f.cursor + buf.len(), f.buffer.len());

                let slice = &f.buffer[start..end];

                buf[0..slice.len()].copy_from_slice(slice);

                f.cursor += slice.len();
                slice.len()
            }
        }
    }

    fn write(&mut self, buf: &mut [u8]) -> usize {
        match self {
            PatchableFile::Unpatched(f) => match f.write(buf) {
                Ok(v) => v,
                Err(_) => 0,
            },
            PatchableFile::Patched(_) => {
                panic!("Tried to write to patched file!");
            }
        }
    }

    fn gets(&mut self, buf: &mut [u8]) {
        match self {
            PatchableFile::Unpatched(f) => {
                let mut counter = 0;
                let mut b = [0u8];

                loop {
                    match f.read(&mut b) {
                        Ok(v) => {
                            if v > 0 && buf[counter] != 0 {
                                buf[counter] = b[0];
                                counter += v;

                                if counter >= buf.len() {
                                    return;
                                }
                            } else {
                                return;
                            }
                        }
                        Err(e) => {
                            logger::log(LogLevel::Error, &format!("Failed to read file: {}", e));
                        }
                    }
                }
            }
            PatchableFile::Patched(f) => {
                let start = f.cursor;
                let end = usize::min(f.cursor + buf.len(), f.buffer.len());

                let slice = &f.buffer[start..end];

                for (i, c) in slice.iter().enumerate() {
                    let b = c;
                    f.cursor += 1;

                    buf[i] = *b;

                    if *b == 0 {
                        return;
                    }
                }
            }
        }
    }

    fn puts(&mut self, str: &std::ffi::CStr) {
        match self {
            PatchableFile::Unpatched(f) => match f.write(str.to_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    logger::log(LogLevel::Error, &format!("Failed to write to file: {}", e));
                }
            },
            PatchableFile::Patched(_) => {
                panic!("Tried to write to patched file!");
            }
        }
    }

    fn eof(&mut self) -> bool {
        match self {
            PatchableFile::Unpatched(f) => {
                let pos = match f.seek(SeekFrom::Current(0)) {
                    Ok(v) => v,
                    Err(e) => {
                        logger::log(
                            LogLevel::Error,
                            &format!("Failed to get file position: {}", e),
                        );
                        return true;
                    }
                };

                let mut b = [0u8];
                let n = match f.read(&mut b) {
                    Ok(v) => v,
                    Err(e) => {
                        logger::log(LogLevel::Error, &format!("Failed to read file: {}", e));
                        return true;
                    }
                };

                if let Err(e) = f.seek(SeekFrom::Start(pos)) {
                    logger::log(LogLevel::Error, &format!("Failed to seek file: {}", e));
                }
                n == 0
            }
            PatchableFile::Patched(f) => f.cursor >= f.buffer.len(),
        }
    }

    fn seek(&mut self, pos: SeekFrom) -> bool {
        match self {
            PatchableFile::Unpatched(f) => match f.seek(pos) {
                Ok(_) => false,
                Err(e) => {
                    logger::log(LogLevel::Error, &format!("Failed to seek file: {}", e));
                    return true;
                }
            },
            PatchableFile::Patched(f) => match pos {
                SeekFrom::Start(v) => {
                    f.cursor = v as usize;
                    false
                }
                SeekFrom::End(v) => {
                    f.cursor = (f.buffer.len() as i64 - v) as usize;
                    false
                }
                SeekFrom::Current(v) => {
                    f.cursor = (f.cursor as i64 + v) as usize;
                    false
                }
            },
        }
    }

    fn flush(&mut self) {
        if let PatchableFile::Unpatched(f) = self {
            if let Err(e) = f.flush() {
                logger::log(LogLevel::Error, &format!("Failed to flush file: {}", e));
            }
        }
    }

    fn tell(&mut self) -> u64 {
        match self {
            PatchableFile::Unpatched(f) => match f.seek(SeekFrom::Current(0)) {
                Ok(v) => v,
                Err(e) => {
                    logger::log(
                        LogLevel::Error,
                        &format!("Failed to get file position: {}", e),
                    );
                    return 0;
                }
            },
            PatchableFile::Patched(f) => f.cursor as u64,
        }
    }
}

unsafe extern "C" fn file_exists(path: *const std::ffi::c_char) -> bool {
    println!("EXISTS");

    let path_str = unsafe {
        let str = std::ffi::CStr::from_ptr(path);
        String::from(str.to_string_lossy())
    };

    let p = std::path::Path::new(&path_str);

    match std::fs::exists(p) {
        Ok(v) => v,
        Err(_) => false,
    }
}

unsafe extern "C" fn open_file(
    path: *const std::ffi::c_char,
    opts: *const std::ffi::c_char,
) -> *mut PatchableFile {
    //println!("OPEN");

    let path_str = unsafe {
        let str = std::ffi::CStr::from_ptr(path);
        String::from(str.to_string_lossy())
    };

    let opts_str = unsafe {
        std::ffi::CStr::from_ptr(opts)
        //String::from(str.to_string_lossy())
    };

    let mut o = std::fs::OpenOptions::new();

    for c in opts_str.to_bytes() {
        match c {
            b'r' | b'R' => {
                o.read(true);
            }
            b'w' | b'W' => {
                o.create(true).write(true);
            }
            b'a' | b'A' => {
                o.append(true).write(true);
            }
            b'+' => {
                o.write(true).read(true);
            }
            _ => {}
        }
    }

    let p = std::path::Path::new(&path_str);

    match PatchableFile::open(p, o) {
        Some(v) => Box::into_raw(v),
        None => std::ptr::null_mut(),
    }
}

unsafe extern "C" fn close_file(f: *mut PatchableFile) {
    //println!("CLOSE");

    if f.is_null() {
        return;
    }

    unsafe {
        // becomes owned, thus drops after this block.
        let _ = Box::from_raw(f);
    }
}

unsafe extern "C" fn read_file(buf: *mut u8, size: u32, count: u32, f: *mut PatchableFile) -> u32 {
    //println!("READ {} {}", size, count);

    if f.is_null() {
        return 0;
    }

    unsafe {
        let mut file = Box::from_raw(f);

        let slice_len = (size * count) as usize;
        let slice = std::slice::from_raw_parts_mut(buf, slice_len);

        let result = file.read(slice) as u32;

        // prevent dropping the box
        let _ = Box::into_raw(file);

        result
    }
}

unsafe extern "C" fn write_file(buf: *mut u8, size: u32, count: u32, f: *mut PatchableFile) -> u32 {
    //println!("WRITE");

    if f.is_null() {
        return 0;
    }

    unsafe {
        let mut file = Box::from_raw(f);

        let slice_len = (size * count) as usize;
        let slice = std::slice::from_raw_parts_mut(buf, slice_len);

        let result = file.write(slice) as u32;

        // prevent dropping the box
        let _ = Box::into_raw(file);

        result
    }
}

unsafe extern "C" fn gets_file(buf: *mut std::ffi::c_char, count: u32, f: *mut PatchableFile) {
    //println!("GETS");

    if f.is_null() {
        return;
    }

    unsafe {
        let mut file = Box::from_raw(f);

        let slice_len = count as usize;
        let slice = std::slice::from_raw_parts_mut(buf as *mut u8, slice_len);

        file.gets(slice);

        // prevent dropping the box
        let _ = Box::into_raw(file);
    }
}

unsafe extern "C" fn puts_file(buf: *mut std::ffi::c_char, f: *mut PatchableFile) {
    //println!("PUTS");

    if f.is_null() {
        return;
    }

    unsafe {
        let mut file = Box::from_raw(f);

        let str = std::ffi::CStr::from_ptr(buf);

        file.puts(str);

        // prevent dropping the box
        let _ = Box::into_raw(file);
    }
}

unsafe extern "C" fn eof_file(f: *mut PatchableFile) -> bool {
    //println!("EOF");

    if f.is_null() {
        return true;
    }

    unsafe {
        let mut file = Box::from_raw(f);

        let result = file.eof();

        // prevent dropping the box
        let _ = Box::into_raw(file);

        result
    }
}

unsafe extern "C" fn seek_file(f: *mut PatchableFile, offset: i32, origin: i32) -> bool {
    //println!("SEEK {} {}", offset, origin);

    if f.is_null() {
        return true;
    }

    unsafe {
        let mut file = Box::from_raw(f);

        let pos = match origin {
            0 => SeekFrom::Start(offset as u64),
            1 => SeekFrom::Current(offset as i64),
            2 => SeekFrom::End(offset as i64),
            _ => SeekFrom::Current(offset as i64),
        };

        let result = file.seek(pos);

        // prevent dropping the box
        let _ = Box::into_raw(file);

        result
    }
}

unsafe extern "C" fn flush_file(f: *mut PatchableFile) {
    //println!("FLUSH");

    if f.is_null() {
        return;
    }

    unsafe {
        let mut file = Box::from_raw(f);

        file.flush();

        // prevent dropping the box
        let _ = Box::into_raw(file);
    }
}

unsafe extern "C" fn tell_file(f: *mut PatchableFile) -> u32 {
    //println!("TELL");

    if f.is_null() {
        return 0;
    }

    unsafe {
        let mut file = Box::from_raw(f);

        let result = file.tell() as u32;

        // prevent dropping the box
        let _ = Box::into_raw(file);

        result
    }
}

unsafe extern "C" fn write_file_funcs() {
    unsafe {
        let get_file_funcs: extern "C" fn() -> *mut [*const (); 11] =
            std::mem::transmute(0x0054e150);

        let file_funcs = &mut *get_file_funcs();

        file_funcs[0] = 0x004030e0 as *const ();
        file_funcs[1] = 0x00403120 as *const ();
        file_funcs[2] = close_file as *const ();
        file_funcs[3] = read_file as *const ();
        file_funcs[4] = write_file as *const ();
        file_funcs[5] = gets_file as *const ();
        file_funcs[6] = puts_file as *const ();
        file_funcs[7] = eof_file as *const ();
        file_funcs[8] = seek_file as *const ();
        file_funcs[9] = flush_file as *const ();
        file_funcs[10] = tell_file as *const ();
    }
}

pub unsafe fn patch() {
    unsafe {
        // replace RW file I/O. i don't think it's used at all lol
        patch::patch_jmp(0x0054e220 as *mut (), file_exists as *const ());
        patch::patch_jmp(0x00577afe as *mut (), open_file as *const ());
        patch::patch_jmp(0x00577b0a as *mut (), close_file as *const ());
        patch::patch_jmp(0x00577b16 as *mut (), read_file as *const ());
        patch::patch_jmp(0x00577b1c as *mut (), write_file as *const ());
        patch::patch_jmp(0x00577b22 as *mut (), gets_file as *const ());
        patch::patch_jmp(0x00577b28 as *mut (), puts_file as *const ());
        patch::patch_jmp(0x00577b3a as *mut (), eof_file as *const ());
        patch::patch_jmp(0x00577b10 as *mut (), seek_file as *const ());
        patch::patch_jmp(0x00577b2e as *mut (), flush_file as *const ());
        patch::patch_jmp(0x00577b34 as *mut (), tell_file as *const ());

        // now replace the actually used stuff
        patch::patch_nop(0x004031ac as *mut (), 6);
        patch::patch_call(0x004031ac as *mut (), open_file as *const ());
        patch::patch_jmp(0x00403220 as *mut (), write_file_funcs as *const ());
    }
}
