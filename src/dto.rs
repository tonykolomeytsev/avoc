
#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub payload: String,
    pub pos: usize,
}

#[derive(Debug)]
pub enum TokenType {
    Operator,
    Identifier,
    IntConstant,
    FloatConstant,
    StringConstant,
    NewLine,
    Indent { depth: usize },
}