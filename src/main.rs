use clap::Parser;
use patternscan::scan;
use std::io::Cursor;
use std::io::Read;
use std::fs::File;
use std::fs;
use std::io::Seek;
use std::io::SeekFrom;
use std::thread;
use pretty_hex::*;

/// Tool for finding patterns in memory
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
     ///PID of the process to scan
     #[clap(short, long)]
     file_name: String,

	////HEX pattern to search for
    #[clap(default_value="FF", short, long)]
    hex_pattern: String,

	///Parallel degree
	#[clap(default_value_t=4,short='P', long)]
	parallel: u64,

	///Size of a buffer to print
	#[clap(default_value_t=256,short, long)]
	buffer: usize,

    ///STRING pattern in hex to search for
    #[clap(default_value="0xFF", short, long)]
    string_pattern: String,

}

fn hexlify(p: String) -> String {
    let buffer = p.as_bytes();
    let mut hex_pattern: String = String::new();
    for b in buffer {
        hex_pattern = format!("{} {:02x}", hex_pattern, b);
    }
    hex_pattern
}


fn scan_memory(fname: String, scan_from: u64, scan_to: u64, pattern: String, buffer_to_print: usize) {
    print!("Scanning memory from {} to {} in a separate thread\n", scan_from, scan_to);

    let mut f = File::open(fname).unwrap();  

    let mut position = scan_from;
    while position < scan_to {
        f.seek(SeekFrom::Start(position)).unwrap(); 
        let mut buffer = [0; 1_048_576]; //1M buffer
        f.read(&mut buffer).unwrap();
        let positions = scan(Cursor::new(buffer), &pattern).unwrap();
        if positions.len() > 0 {
	    println!("\nFound {} positions in a chunk", positions.len());
            for p in positions {
                println!("Offset: {} \n", p+position as usize);
	            println!("{:?}\n\t", buffer[(p as usize)..(p as usize+buffer_to_print)].hex_dump());
            }
	}
	position += 1_048_576;
        print!("\rScanned: {} %", ((position-scan_from) as f64 / (scan_to-scan_from) as f64 * 100 as f64) as u8);
    }
}


fn main() {
    let args = Args::parse();
    let mut pattern = args.hex_pattern;

    let scan_from: u64 = 0;
    let file_size = fs::metadata(&args.file_name).unwrap().len();
    let scan_to: u64 = file_size;

    if args.string_pattern != "0xFF" {
        pattern = hexlify(args.string_pattern);
    }

    let chunk = file_size / args.parallel;
    let mut scan_from_chunk = scan_from;
    let mut threads: Vec<thread::JoinHandle<_>> = Vec::new();

    while scan_from_chunk < scan_to {
        let mut scan_to_chunk = scan_from_chunk+chunk;
        if scan_to_chunk > scan_to {
            scan_to_chunk = scan_to;
        }
        let t = thread::Builder::new().stack_size(32 * 1024 * 1024);
        let p = pattern.clone();
        let fname = args.file_name.clone();
        threads.push(t.spawn(move || {scan_memory(fname, scan_from_chunk, scan_to_chunk, p, args.buffer);}).unwrap());
        scan_from_chunk+=chunk;
    }

    for t in threads {
	    t.join().unwrap();
    }
    
}
