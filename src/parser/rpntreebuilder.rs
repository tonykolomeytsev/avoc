
use crate::dto::{ Token, Node, NodeType, SyntaxError };
use std::collections::VecDeque;

pub struct RpnTreeBuilder {
    stack: VecDeque<Node>,
}

impl RpnTreeBuilder {

    pub fn new() -> RpnTreeBuilder {
        RpnTreeBuilder {
            stack: VecDeque::new(),
        }
    }

    pub fn push_token(&mut self, token: &Token, output: &mut Vec<Node>) -> Result<(), SyntaxError> {
        match token {
            Token::Operator { payload, pos } => handle_operator(payload, pos, output),
        };
        Ok(())
    }

    pub fn notify_met_separator(&mut self, output: &mut Vec<Node>) -> Result<(), SyntaxError> {
        todo!()
    }
}

#[inline]
fn handle_operator(payload: &String, pos: &usize, output: &mut Vec<Node>) -> Result<(), SyntaxError> {
    todo!()
}