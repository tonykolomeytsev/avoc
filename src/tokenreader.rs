use crate::dto::{ Token, TokenType };
use std::cell::Cell;

const OPERATORS: &'static &str = &"=+-*/\\().,<>";
const SINGLE_CHAR_OPERATORS: &'static &str = &"\\().,";
const KEYWORDS: &'static [&'static str] = &[
    "if", 
    "else",
    "while",
];

pub struct TokenReader {
    source: String,
    state: Cell<State>,
}

#[derive(Copy, Clone, Debug)]
struct State {
    expected: Expected,
    start_offset: usize,
    is_ready_to_push: bool,
    is_start_of_line: bool,
    prev_operator_char: Option<char>,
}

#[derive(Copy, Clone, Debug)]
enum Expected {
    Nothing,
    IntNumber,
    FloatNumber,
    Identifier,
    Operator,
    Indent,
    Newline,
}

impl TokenReader {

    pub fn new(source: String) -> TokenReader {
        TokenReader {
            source: source + "\n",
            state: Cell::from(State {
                start_offset: 0,
                expected: Expected::Nothing,
                is_ready_to_push: false,
                is_start_of_line: false,
                prev_operator_char: None,
            })
        }
    }

    pub fn parse(&self) -> Vec<Token> {
        let mut iter = self.source.chars();
        let mut offset = 0usize;
        let mut tokens: Vec<Token> = vec!();
        let mut new_char = None;
        loop {
            if !self.state.get().is_ready_to_push {
                new_char = iter.next();
                offset += 1;
            }
            push_token_if_ready(&self.state, &self.source, offset, &mut tokens);    
            match new_char {
                Some(val) => self.state.set(reduce_state(val, offset - 1, self.state.get())),
                None => break,
            };
        }
        self.state.set(State { is_ready_to_push: true, ..self.state.get() });
        push_token_if_ready(&self.state, &self.source, offset, &mut tokens);
        tokens
    }
}

fn push_token_if_ready(state_cell: &Cell<State>, source: &String, offset: usize, tokens: &mut Vec<Token>) {
    let state = state_cell.get();
    if state.is_ready_to_push {
        let start = state.start_offset;
        let end = offset - 1;
        let token_content = String::from(&source[start..end]);
        match state.expected {
            Expected::IntNumber => tokens.push(Token { token_type: TokenType::IntConstant, payload: token_content, pos_start: start }),
            Expected::FloatNumber => tokens.push(Token { token_type: TokenType::FloatConstant, payload: token_content, pos_start: start }),
            Expected::Identifier => tokens.push(get_keyword_or_identifier(token_content, start)),
            Expected::Operator => tokens.push(Token { token_type: TokenType::Operator, payload: token_content, pos_start: start }),
            Expected::Newline => tokens.push(Token { token_type: TokenType::NewLine, payload: token_content, pos_start: start }),
            Expected::Indent => tokens.push(Token { token_type: TokenType::Indent { depth: end - start }, payload: token_content, pos_start: start }),
            Expected::Nothing => {},
        };
        state_cell.set(State { 
            is_ready_to_push: false, 
            expected: Expected::Nothing, 
            is_start_of_line: false, 
            prev_operator_char: None, 
            ..state
        });
    }
}

fn get_keyword_or_identifier(token_content: String, start: usize) -> Token {
    match token_content {
        val if KEYWORDS.iter().any(|k| k.to_string() == val) => Token { token_type: TokenType::Operator, payload: val, pos_start: start },
        _ => Token { token_type: TokenType::Identifier, payload: token_content, pos_start: start }
    }
}

fn reduce_state(symbol: char, offset: usize, state: State) -> State {
    match state.expected {
        Expected::Nothing => reduce_state_nothing(symbol, offset, state),
        Expected::IntNumber => reduce_state_int_number(symbol, state),
        Expected::FloatNumber => reduce_state_float_number(symbol, state),
        Expected::Identifier => reduce_state_identifier(symbol, state),
        Expected::Operator => reduce_state_operator(symbol, state),
        Expected::Indent => reduce_state_whitespace(symbol, state),
        _ => panic!("Unexpected symbol {:?} at offset {}", symbol, offset)
    }
}

fn reduce_state_nothing(symbol: char, offset: usize, state: State) -> State {
    match symbol {
        val if val.is_digit(10) => State { expected: Expected::IntNumber, start_offset: offset, ..state },
        val if val.is_alphabetic() => State { expected: Expected::Identifier, start_offset: offset, ..state },
        val if val.is_whitespace() => match state.is_start_of_line {
            true => State { expected: Expected::Indent, start_offset: offset, ..state },
            false => State { start_offset: offset, ..state },
        },
        val if OPERATORS.chars().any(|s| s == val) =>
            State { expected: Expected::Operator, start_offset: offset, ..state },
        '\n' => State { expected: Expected::Newline, start_offset: offset, is_ready_to_push: true, ..state },
        _ => panic!("Unexpected symbol {:?} at offset {}", symbol, offset),
    }
}

fn reduce_state_int_number(symbol: char, state: State) -> State {
    match symbol {
        val if val.is_digit(10) => state,
        '.' => State { expected: Expected::FloatNumber, ..state },
        _ => State { is_ready_to_push: true, ..state },
    }
}

fn reduce_state_float_number(symbol: char, state: State) -> State {
    match symbol {
        val if val.is_digit(10) => state,
        _ => State { is_ready_to_push: true, ..state },
    }
}

fn reduce_state_identifier(symbol: char, state: State) -> State {
    match symbol {
        val if val.is_alphanumeric() => state,
        '_' => state,
        _ => State { is_ready_to_push: true, ..state },
    }
}

fn reduce_state_operator(symbol: char, state: State) -> State {
    match state.prev_operator_char {
        None => match symbol {
            val if SINGLE_CHAR_OPERATORS.chars().any(|s| s == val) => 
                State { is_ready_to_push: true, prev_operator_char: None, ..state },
            val if OPERATORS.chars().any(|s| s == val) =>
                State { prev_operator_char: Some(val), ..state },
            _ => 
                State { is_ready_to_push: true, prev_operator_char: None, ..state },
        },
        Some(_) => State { is_ready_to_push: true, prev_operator_char: None, ..state }
    }
}

fn reduce_state_whitespace(symbol: char, state: State) -> State {
    match symbol {
        val if val.is_whitespace() => state,
        _ => State { is_ready_to_push: true, ..state },
    }
}

