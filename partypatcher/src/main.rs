use crate::patcher::{ChecksumSet, PatchConfig, PatcherConfig, PatcherResult, do_patch};

mod patcher;
mod crc;
mod bps;

fn main() -> PatcherResult {
    let patch_config = PatcherConfig {
        input_filename: "Skate3.exe",
        output_filename: "THPS3.exe",
        failure_hint: "Make sure THPS3 US Patch 1.01 is installed",
        patches: vec![PatchConfig { 
            input_size: 1908736, 
            patch_data: include_bytes!("us101.bps"), 
            checksums: vec![
                ChecksumSet { input: 0xdda4822f, output: 0xbb5e5c48 },
                ChecksumSet { input: 0x045925e8, output: 0xbb3c2f62 },
                ChecksumSet { input: 0xa1414bba, output: 0xff4861b5 },
            ], 
        }],
    };

    do_patch(&patch_config)
}