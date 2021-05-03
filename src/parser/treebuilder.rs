use crate::dto::{ Node, Token };
use std::cell::Cell;

#[derive(Debug)]
pub struct TreeBuilder {
    state: Cell<State>,
}

#[derive(Copy, Clone, Debug)]
struct State {
    reading: Reading,
}

#[derive(Copy, Clone, Debug)]
enum Reading {
    Nothing,
}

#[derive(Debug)]
pub struct SyntaxError { pub pos: usize, pub message: String }

impl TreeBuilder {

    pub fn new() -> TreeBuilder {
        TreeBuilder {
            state: Cell::from(State {
                reading: Reading::Nothing,
            })
        }
    }

    pub fn build_tree(&self, _: &Vec<Token>) -> Result<Node, SyntaxError> {
        todo!()
    }
}
