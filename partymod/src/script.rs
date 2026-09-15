pub unsafe fn get_script_checksum(cscript: *const ()) -> u32 {
    unsafe { (cscript as *const u32).byte_offset(0x390).read() }
}
