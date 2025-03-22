use clap::{Parser, command};
use fxr_binary_reader::fxr::fxr_parser_with_sections::parse_fxr;
use std::path::PathBuf;
use std::process::ExitCode;

use std::fs::File;
use std::io::{self, Read};

fn is_binary(file_path: &PathBuf) -> io::Result<bool> {
    let mut file = File::open(file_path)?;
    let mut buffer = [0; 1024];
    let bytes_read = file.read(&mut buffer)?;

    // Check for ASCII NUL bytes
    for &byte in &buffer[..bytes_read] {
        if byte == 0 {
            return Ok(true);
        }
    }

    Ok(false)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    bin_path: PathBuf,

    #[arg(short, long, default_value_t = String::from("0x00000000"))]
    offset: String,
}

fn valid_args() -> Args {
    let args = Args::parse();
    // Check that the file exists
    if !args.bin_path.exists() {
        eprintln!("Error: File does not exist: {}", args.bin_path.display());
        std::process::exit(1);
    }
    // Check that the file is binary format
    if !is_binary(&args.bin_path).unwrap_or(false) {
        eprintln!(
            "Error: File is not a binary file: {}",
            args.bin_path.display()
        );
        std::process::exit(1);
    }
    // Check that the offset is a valid hex string
    if !args.offset.starts_with("0x") {
        eprintln!("Error: Offset is not a valid hex string: {}", args.offset);
        std::process::exit(1);
    }
    // Check that the offset would not be out of bounds
    let offset = u32::from_str_radix(&args.offset[2..], 16).unwrap_or(0);
    let file_size = std::fs::metadata(&args.bin_path).unwrap().len();
    if offset as u64 >= file_size {
        eprintln!("Error: Offset is out of bounds: {}", args.offset);
        std::process::exit(1);
    }
    args
}

fn main() -> ExitCode {
    let _args = valid_args();
    // If all checks pass, print success message
    println!("All checks passed. Proceeding with further processing...");
    let _header = parse_fxr(&_args.bin_path).unwrap();
    ExitCode::SUCCESS
}
