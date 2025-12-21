use std::{fs::File, io::{Read, Write}, path::Path, process::{ExitCode, Termination}};

#[cfg(windows)] 
use libc::c_int;

use crate::{bps::bps_patch_unchecked, crc::crc32};
#[cfg(windows)] 
extern "C" {
    fn _getch()->c_int;
}

#[repr(u8)]
pub enum PatcherResult {
    Success = 0,
    Failure = 1,
}

impl Termination for PatcherResult {
    fn report(self) -> std::process::ExitCode {
        ExitCode::from(self as u8)
    }
}

pub struct ChecksumSet {
    pub input: u32,
    pub output: u32,
}

pub struct PatchConfig {
    pub input_size: usize,
    pub patch_data: &'static [u8],
    pub checksums: Vec<ChecksumSet>,
}

pub struct PatcherConfig {
    pub input_filename: &'static str,
    pub output_filename: &'static str,
    pub failure_hint: &'static str,
    pub patches: Vec<PatchConfig>,
}

#[cfg(windows)]
fn pause_for_key() {
    println!("Press any key to continue...");
    unsafe { _getch() };
}

#[cfg(not(windows))]
fn pause_for_key() {}

pub fn do_patch_internal(config: &PatcherConfig) -> PatcherResult {
    // get input file and dump it
    let mut input_file = match File::open(Path::new(config.input_filename)) {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to open {}: {}", config.input_filename, e);
            return PatcherResult::Failure;
        }
    };

    let mut input_file_buffer = Vec::new();
    match input_file.read_to_end(&mut input_file_buffer) {
        Ok(_) => {},
        Err(e) => {
            println!("Failed to read {}: {}", config.input_filename, e);
        }
    }

    // use the size of input file to find appropriate patch
    let mut target_patch_result = None;

    for patch in &config.patches {
        if patch.input_size == input_file_buffer.len() {
            target_patch_result = Some(patch);
            break;
        }
    }

    let target_patch = match target_patch_result {
        Some(v) => v,
        None => {
            println!("Unexpected input size: {} bytes", input_file_buffer.len());
            return PatcherResult::Failure;
        },
    };

    // get checksum of data
    let input_crc = crc32(&input_file_buffer);
    let mut expected_output_crc = None;

    for checksum in &target_patch.checksums {
        if input_crc == checksum.input {
            expected_output_crc = Some(checksum.output);
            break;
        }
    }

    if expected_output_crc.is_none() {
        println!("Unexpected input crc: {:#010x}", input_crc);
    }

    // start patch
    let output_buffer = match bps_patch_unchecked(&input_file_buffer, target_patch.patch_data) {
        Ok(v) => v,
        Err(e) => {
            println!("Patch failed: {:?}!", e);
            return PatcherResult::Failure;
        },
    };

    // test output crc
    let actual_output_crc = crc32(&output_buffer);
    match expected_output_crc {
        Some(v) => if actual_output_crc != v {
            println!("Unexpected output crc {:#010x} for input crc {:#010x}!", actual_output_crc, input_crc);
        },
        None => {
            println!("Output crc: {:#010x}", actual_output_crc);
        },
    }

    // open and write output file
    println!("Writing {}...", config.output_filename);

    let mut output_file = match File::create(Path::new(config.output_filename)) {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to open {} for writing: {}", config.output_filename, e);
            return PatcherResult::Failure;
        }
    };

    match output_file.write_all(&output_buffer) {
        Ok(_) => {},
        Err(e) => {
            println!("Failed to write {}: {}", config.output_filename, e);
        }
    };

    PatcherResult::Success
}

pub fn do_patch(config: &PatcherConfig) -> PatcherResult {
    let result = do_patch_internal(config);

    match result {
        PatcherResult::Success => {
            println!("Patch successful!");
        },
        PatcherResult::Failure => {
            println!("Patch failed!");
            if !config.failure_hint.is_empty() {
                println!("{}", config.failure_hint);
            }
        },
    }
    pause_for_key();

    result
}