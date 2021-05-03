
#[inline]
pub fn red(string: String) -> String {
    format!("{}{}{}", "\u{001b}[31m\u{001b}[1m", string, "\u{001b}[0m")
}

#[inline]
#[allow(dead_code)]
pub fn yellow(string: String) -> String {
    format!("{}{}{}", "\u{001b}[33m\u{001b}[1m", string, "\u{001b}[0m")
}

#[inline]
#[allow(dead_code)]
pub fn green(string: String) -> String {
    format!("{}{}{}", "\u{001b}[32m\u{001b}[1m", string, "\u{001b}[0m")
}

#[inline]
#[allow(dead_code)]
pub fn blue(string: String) -> String {
    format!("{}{}{}", "\u{001b}[34m\u{001b}[1m", string, "\u{001b}[0m")
}

pub fn print_error_info(file_name: &String, source: &String, offset: usize, message: String) {
    let mut line_num = 1;
    let mut sum = 0usize;
    for line in source.lines() {
        let len = line.len();
        if sum + len >= offset {
            let column = offset - sum;
            println!("\n{}: {}:{}:{}\n", red(String::from("error")), file_name, line_num, column);
            println!("{}", line);
            println!("{}", red(format!("{:width$}^ {}\n", "", message, width=column)));
            return
        }
        sum += len + 1;
        line_num += 1;
    }
    println!("\nCan't extract debug info. Message: {} at {}", message, offset)
}