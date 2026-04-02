unsafe extern "C" {
    fn get_stdin() -> *mut libc::FILE;
    fn get_stdout() -> *mut libc::FILE;
    fn get_stderr() -> *mut libc::FILE;
}

pub fn init_console() {
    unsafe {
        windows_sys::Win32::System::Console::AllocConsole();

        libc::freopen(c"CONIN$".as_ptr(), c"r".as_ptr(), get_stdin());
        libc::freopen(c"CONOUT$".as_ptr(), c"w".as_ptr(), get_stderr());
        libc::freopen(c"CONOUT$".as_ptr(), c"w".as_ptr(), get_stdout());
    }
}
