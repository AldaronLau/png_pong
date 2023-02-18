use std::{fs::File, io::BufReader};

use png_pong::{decode::Error, Decoder};

fn main() {
    let reader = BufReader::new(File::open("res/icon.png").unwrap());
    for chunk in Decoder::new(reader).expect("Not a PNG file").into_chunks() {
        match chunk {
            Ok(c) => println!("Chunk {:?}", c),
            Err(e) => match e {
                Error::UnknownChunkType(bytes) => println!(
                    "Unknown Chunk: {:?}",
                    String::from_utf8_lossy(&bytes)
                ),
                e => panic!("Other Error: {:?}", e),
            },
        }
    }
}
