use std::{env, fs::File, io::{BufRead, BufReader, Read, Write}, path::Path};

enum TextureOp {
    SetAlpha(u8),
    MultiplyAlpha(f64),
}

struct TextureModifier {
    texture: String,
    op: TextureOp,
}

fn search_name(buf: &[u8], target: &[u8]) -> Option<usize> {
    for start in 0..buf.len() - target.len() {
        if &buf[start..(start + target.len())] == target {
            return Some(start);
        }
    }

    None
}

fn process_modifier(modifier: &TextureModifier, buf: &mut [u8]) {
    println!("Applying modifier for {}", modifier.texture);

    // find texture
    let offset = match search_name(&buf, modifier.texture.as_bytes()) {
        Some(v) => v,
        None => {
            println!("Failed to find texture \"{}\" in dictionary!", modifier.texture);
            return
        },
    };

    // get size (u16 +0x104bytes, u16 +0x10abytes)
    let width_bytes = [buf[offset + 0x108], buf[offset + 0x109]];
    let width = u16::from_le_bytes(width_bytes) as usize;

    let height_bytes = [buf[offset + 0x10a], buf[offset + 0x10b]];
    let height = u16::from_le_bytes(height_bytes) as usize;

    println!("{}x{} pixels", width, height);

    let size = width * height * 4;
    let texture_offset = offset + 0x114;
    let texture_buf = &mut buf[texture_offset..(texture_offset + size)];

    // modify data (+0x114bytes)
    match modifier.op {
        TextureOp::SetAlpha(v) => {
            for pixel in 0..(width * height) {
                let alpha = &mut texture_buf[3 + (pixel * 4)];

                *alpha = v;
            }
        },
        TextureOp::MultiplyAlpha(v) => {
            for pixel in 0..(width * height) {
                let alpha = &mut texture_buf[3 + (pixel * 4)];

                *alpha = ((*alpha as f64) * v) as u8;
            }
        },
    }
}

fn parse_modifier(line_buf: &str) -> Option<TextureModifier> {
    let mut token_iter = line_buf.trim().split(' ');

    let texture_name = match token_iter.next() {
        Some(v) => v,
        None => {
            return None;
        }
    };

    let op = match token_iter.next() {
        Some(v) => v,
        None => {
            println!("Missing operator for texture \"{}\"!", texture_name);

            return None;
        }
    };

    println!("Name: {} Op: {}", texture_name, op);

    match op {
        "S" | "s" => {
            let value_buf = match token_iter.next() {
                Some(v) => v,
                None => {
                    println!("No set operator value for texture \"{}\"!", texture_name);
                    return None;
                }
            };

            println!("Parsing {} as byte", value_buf);

            let value = match value_buf.parse::<u8>() {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to parse set operator value for texture \"{}\": {}", texture_name, e);
                    return None;
                }
            };

            Some(TextureModifier {
                texture: texture_name.to_string(),
                op: TextureOp::SetAlpha(value),
            })
        }
        "M" | "m" => {
            let value_buf = match token_iter.next() {
                Some(v) => v,
                None => {
                    println!("No multiply operator value for texture \"{}\"!", texture_name);
                    return None;
                }
            };

            println!("Parsing {} as float", value_buf);

            let value = match value_buf.parse::<f64>() {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to parse multiply operator value for texture \"{}\": {}", texture_name, e);
                    return None;
                }
            };

            Some(TextureModifier {
                texture: texture_name.to_string(),
                op: TextureOp::MultiplyAlpha(value),
            })
        }
        _ => None
    }
}

fn modify_tdx(script_path_str: &str) {
    let script_path = Path::new(script_path_str);
    let script_file = File::open(script_path).unwrap();

    let mut script_reader = BufReader::new(script_file);

    let mut dict_name = String::new();
    script_reader.read_line(&mut dict_name).unwrap();
    dict_name = dict_name.trim().to_string();

    println!("Opening texture dictionary \"{}\"", dict_name);

    let dict_path = Path::new(&dict_name);
    let mut dict_file = File::open(dict_path).unwrap();
    let mut dict_buf = Vec::new();
    dict_file.read_to_end(&mut dict_buf).unwrap();

    let mut output_name = String::new();
    script_reader.read_line(&mut output_name).unwrap();
    output_name = output_name.trim().to_string();

    println!("Output: \"{}\"", output_name);

    let mut texture_modifiers = Vec::new();
    let mut line_buf = String::new();
    while let Ok(len) = script_reader.read_line(&mut line_buf) && len > 0 {
        match parse_modifier(&line_buf) {
            Some(v) => texture_modifiers.push(v),
            None => {},
        }

        line_buf.clear();
    }

    for modifier in texture_modifiers {
        process_modifier(&modifier, &mut dict_buf);
    }

    let output_path = Path::new(&output_name);
    let mut output_file = File::create(output_path).unwrap();
    output_file.write_all(&dict_buf).unwrap();
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0); // ignore first arg
    for arg in args {
        modify_tdx(&arg);
    }
}