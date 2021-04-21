
mod dto;
mod tokenreader;

use std::io::prelude::*;
use std::fs::File;
use tokenreader::{ TokenReader };

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let content = read_source_file(&args[1]);
    let token_reader = TokenReader::new(content);
    match token_reader.parse() {
        Ok(tokens) => tokens.iter().for_each(|it| println!("{:?}", it)),
        Err(e) => {
            println!("Syntax error {}\n", get_debug_info(&args[1], e.pos, e.message));
        }
    }
}

fn read_source_file(file_name: &String) -> String {
    let mut file = File::open(file_name).expect("Can't open source file!");
    let mut source_content = String::new();
    file.read_to_string(&mut source_content).expect("Can't read source file!");
    source_content
}

fn get_debug_info(file_name: &String, offset: usize, message: String) -> String {
    let file_content = read_source_file(file_name);
    let mut line_num = 1;
    let mut sum = 0usize;
    for line in file_content.lines() {
        let len = line.len();
        if sum + len >= offset {
            let column = offset - sum;
            return format!("at line {} column {}:\n{}\n{:width$}^ {}", line_num, column, line, "", message, width=column)
        }
        sum += len + 1;
        line_num += 1;
    }
    format!("\nCan't extract debug info. Message: {} at {}", message, offset)
}