use std::env;
use std::fs::File;
use std::io::Read;
use parser;

fn main() {
    let filename = env::args().nth(1).unwrap();
    let mut file = File::open(&filename).map_err(|e| format!("file not found: '{}'", filename)).unwrap();

    let mut source = String::new();

    file.read_to_string(&mut source).map_err(|_e| "error reading file").unwrap();

    let requests = parser::parse(&source);
    println!("{:?}", requests);
}
