
mod dto;
mod parser;
mod io;

use std::io::prelude::*;
use std::fs::File;
use parser::TokenReader;
use parser::TreeBuilder;
use io::print_error_info;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source = read_source_file(&args[1]);
    match TokenReader::new().parse(&source) {
        Ok(tokens) => match TreeBuilder::new().build_tree(&tokens) {
            Ok(tree_node) => println!("{:?}", tree_node),
            Err(e) => print_error_info(&args[1], &source, e.pos, e.message),
        },
        Err(e) => print_error_info(&args[1], &source, e.pos, e.message),
    };
}

fn read_source_file(file_name: &String) -> String {
    let mut file = File::open(file_name).expect("Can't open source file!");
    let mut source_content = String::new();
    file.read_to_string(&mut source_content).expect("Can't read source file!");
    source_content
}