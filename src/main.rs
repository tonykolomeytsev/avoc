
mod dto;
mod tokenreader;

use std::io::prelude::*;
use std::fs::File;
use tokenreader::TokenReader;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let content = read_source_file(&args[1]);
    let token_reader = TokenReader::new(content);
    let tokens = token_reader.parse();
    tokens.iter().for_each(|it| println!("{:?}", it))
}

fn read_source_file(file_name: &String) -> String {
    let mut file = File::open(file_name).expect("Can't open source file!");
    let mut source_content = String::new();
    file.read_to_string(&mut source_content).expect("Can't read source file!");
    source_content
}