use crate::dto::Token;

#[derive(Debug)]
pub enum Node {
    SingleToken(Token),
    Statement(Vec<Node>),
    If { expr: Vec<Node>, children: Vec<Node> },
}