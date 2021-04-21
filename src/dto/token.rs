
#[derive(Debug)]
pub enum Token {
    Operator { payload: String, pos: usize },
    Identifier { name: String, pos: usize },
    IntConstant { value: i32, pos: usize },
    FloatConstant { value: f32, pos: usize },
    StringConstant { value: String, pos: usize },
    NewLine { pos: usize },
    Indent { depth: usize, pos: usize },
}