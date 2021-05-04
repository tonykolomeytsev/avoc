
use crate::dto::{ Token, Node, SyntaxError };
use std::collections::VecDeque;

pub struct RpnTreeBuilder {
    stack: VecDeque<Node>,
    output: Vec<Node>,
}

impl RpnTreeBuilder {

    pub fn new() -> RpnTreeBuilder {
        RpnTreeBuilder {
            stack: VecDeque::new(),
            output: vec!(),
        }
    }

    /// This is the modified "Shunting-yard" algorithm
    /// https://en.wikipedia.org/wiki/Shunting-yard_algorithm
    pub fn push_token(&mut self, token: Token) -> Result<(), SyntaxError> {
        match &token {
            // if token is constant or variable, put it into output
            Token::FloatConstant { value: _, pos: _ } => Ok(self.output.push(Node::from(token))),
            Token::IntConstant { value: _, pos: _ } => Ok(self.output.push(Node::from(token))),
            Token::StringConstant { value: _, pos: _ } => Ok(self.output.push(Node::from(token))),
            Token::Identifier { name: _, pos: _ } => Ok(self.output.push(Node::from(token))),
            // if token is prefix function, push it into stack
            Token::Function { name: _, pos: _ } => Ok(self.stack.push_back(Node::from(token))),
            // if token is operator
            Token::Operator { payload, pos } => match payload.as_str() {
                // if token is operator and it is left bracket '(', put it into stack
                "(" => Ok(self.stack.push_back(Node::from(token))),
                // if token is operator and it is right bracket ')', handle necessary operations
                ")" => handle_right_bracket(&pos, &mut self.stack, &mut self.output),
                // if token is any other operator, handle it
                _ => match handle_operator(&payload, &pos, &mut self.stack, &mut self.output) {
                    Ok(_) => Ok(self.stack.push_back(Node::from(token))), // ...and then put it to stack
                    Err(e) => Err(e),
                },
            },
            Token::NewLine { pos } => 
                Err(SyntaxError { message: String::from("No need to pass the NewLine token to the push_token function, call notify_met_separator instead"), pos: *pos }),
        }
    }

    /// When expression ends, push all operators from stack to output
    pub fn notify_met_separator(&mut self, pos: usize) -> Result<(), SyntaxError> {
        loop {
            match &self.stack.back() {
                Some(node) => match &node.data {
                    Some(token) => match token {
                        Token::Operator { payload, pos } => match payload.as_str() {
                            "(" | ")" => return Err(SyntaxError { message: String::from("The expression contains an extra or inconsistent parenthesis"), pos: *pos }),
                            _ => (),
                        },
                        _ => (),
                    },
                    None => return Err(SyntaxError { message: format!("Unexpected node without token: {:?}", node), pos: pos }),
                },
                None => break,
            };
            self.output.push(self.stack.pop_back().unwrap())
        }
        Ok(())
    }
}

#[inline]
fn handle_right_bracket(
    pos: &usize, 
    stack: &mut VecDeque<Node>,
    output: &mut Vec<Node>, 
) -> Result<(), SyntaxError> {
    // while stack peek is not '(', pop it and put into output
    loop {
        let peek_is_left_bracket = match &stack.back() {
            Some(node) => match &node.data {
                Some(token) => match token {
                    Token::Operator { payload, pos: _ } => match payload.as_str() {
                        "(" => true,
                        _ => false,
                    },
                    _ => false,
                },
                None => return Err(SyntaxError { message: format!("Unexpected node without token: {:?}", node), pos: *pos }),
            },
            // if stack ended earlier than we met '(', then the expression does not match parentheses
            None => return Err(SyntaxError { message: String::from("This closing parenthesis has no matching opening parenthesis"), pos: *pos }),
        };
        let peek = stack.pop_back().unwrap();
        if peek_is_left_bracket {
            break
        } else {
            output.push(peek)
        }
    }
    Ok(())
}

#[inline]
fn handle_operator(
    op: &String, 
    pos: &usize, 
    stack: &mut VecDeque<Node>,
    output: &mut Vec<Node>,
) -> Result<(), SyntaxError> {
    loop {
        let is_need_push_to_output = match &stack.back() {
            Some(node) => match &node.data {
                // while stack peek is...
                Some(token) => match token {
                    // ...prefix function
                    Token::Function { name: _, pos: _ } => true,
                    // ... or peek operator priority higher or equals than handled ooperator
                    Token::Operator { payload, pos: _} => get_priority(payload) >= get_priority(op),
                    _ => false,
                },
                None => return Err(SyntaxError { message: format!("Unexpected node without token: {:?}", node), pos: *pos }),
            },
            None => false,
        };
        if is_need_push_to_output {
            // ...push popped node to output
            output.push(stack.pop_back().unwrap())
        } else {
            break
        }
    };
    Ok(())
}

/// Same as the Java operator precedence table: 
/// http://www.cs.bilkent.edu.tr/~guvenir/courses/CS101/op_precedence.html
#[inline]
fn get_priority(operator: &String) -> u8 {
    match operator.as_str() {
        "." => 15,
        "u-" | "not" => 13,
        "^" => 12,
        "*" | "/" => 11,
        "+" | "-" => 10,
        "<" | "<=" | ">" | ">=" | "is" => 9,
        "==" | "!=" => 8,
        "and" => 7,
        "or" | "xor" => 6,
        "if" | "match" => 2,
        "=" | "+=" | "-=" | "*=" | "/=" => 1,
        _ => 0,
    }
}

#[test]
fn test_proirity() {
    assert_eq!(get_priority(&String::from("and")), 7);
}

#[test]
fn test_simple_math_expressions() {
    let source = vec!(
        Token::Identifier { name: "a".to_string(), pos: 0 },
        Token::Operator { payload: "+".to_string(), pos: 0 },
        Token::Identifier { name: "b".to_string(), pos: 0 },
        Token::Operator { payload: "-".to_string(), pos: 0 },
        Token::Identifier { name: "c".to_string(), pos: 0 },
        Token::Operator { payload: "*".to_string(), pos: 0 },
        Token::Identifier { name: "d".to_string(), pos: 0 },
    );
    let expected: Vec<Node> = vec!(
        Token::Identifier { name: "a".to_string(), pos: 0 },
        Token::Identifier { name: "b".to_string(), pos: 0 },
        Token::Operator { payload: "+".to_string(), pos: 0 },
        Token::Identifier { name: "c".to_string(), pos: 0 },
        Token::Identifier { name: "d".to_string(), pos: 0 },
        Token::Operator { payload: "*".to_string(), pos: 0 },
        Token::Operator { payload: "-".to_string(), pos: 0 },
    ).iter().map(|token| Node::from(token.clone())).collect();

    let mut builder = RpnTreeBuilder::new();
    for token in source {
        builder.push_token(token).unwrap();
    };
    builder.notify_met_separator(0).unwrap();
    assert_eq!(expected, builder.output);
}