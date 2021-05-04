
use crate::dto::Token;

#[derive(Debug, PartialEq)]
pub struct Node {
    pub data: Option<Token>,
    pub node_type: NodeType,
    pub condition: Vec<Node>,
    pub children: Vec<Node>,
}

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Token,
    // Expression,
    // If,
    // Else,
    // Match,
    // Repeat,
    // For,
    // Loop,
    // Lambda,
}

impl Node {

    pub fn from(token: Token) -> Node {
        Node {
            data: Some(token),
            node_type: NodeType::Token,
            condition: vec!(),
            children: vec!(),
        }
    }
    
    fn add_condition_child(&mut self, condition_child: Node) {
        self.condition.push(condition_child)
    }

    fn add_child(&mut self, child: Node) {
        self.condition.push(child)
    }
}

