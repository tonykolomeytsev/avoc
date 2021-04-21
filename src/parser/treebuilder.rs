use crate::dto::Node;
use crate::dto::Token;

#[derive(Debug)]
pub struct TreeBuilder {
    state: State,
}

#[derive(Copy, Clone, Debug)]
struct State {
    expected: Expected,
}

#[derive(Copy, Clone, Debug)]
enum Expected {
    Nothing,
}

impl TreeBuilder {

    fn new() -> TreeBuilder {
        TreeBuilder {
            state: State {
                expected: Expected::Nothing,
            }
        }
    }

    fn parse(&self, tokens: &Vec<Token>) -> Node {
        Node::SingleToken(Token::NewLine { pos: 0 })
    }
}