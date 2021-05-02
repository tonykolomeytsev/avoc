
mod dto;
mod parser;

use std::io::prelude::*;
use std::fs::File;
use parser::TokenReader;
use parser::TreeBuilder;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source = read_source_file(&args[1]);
    match TokenReader::new().parse(&source) {
        Ok(tokens) => tokens.iter().for_each(|it| println!("{:?}", it)),
        Err(e) => print_debug_info(&args[1], &source, e.pos, e.message),
    };
}

fn read_source_file(file_name: &String) -> String {
    let mut file = File::open(file_name).expect("Can't open source file!");
    let mut source_content = String::new();
    file.read_to_string(&mut source_content).expect("Can't read source file!");
    source_content
}

fn print_debug_info(file_name: &String, source: &String, offset: usize, message: String) {
    let mut line_num = 1;
    let mut sum = 0usize;
    for line in source.lines() {
        let len = line.len();
        if sum + len >= offset {
            let column = offset - sum;
            println!("Error in {}:{}:{}", file_name, line_num, column);
            println!("{}", line);
            print!("\u{001b}[31m\u{001b}[1m");
            println!("{:width$}^ {}", "", message, width=column);
            print!("\u{001b}[0m");
            return
        }
        sum += len + 1;
        line_num += 1;
    }
    println!("\nCan't extract debug info. Message: {} at {}", message, offset)
}