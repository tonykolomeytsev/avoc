
#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub payload: String,
    pub pos_start: usize,
}

#[derive(Debug)]
pub enum TokenType {
    Operator,
    Identifier,
    IntConstant,
    FloatConstant,
    NewLine,
    Indent { depth: usize },
}