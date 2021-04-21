use crate::dto::token::Token;
use std::cell::Cell;

const OPERATORS: &'static &str = &"=+-*/\\().,<>";
const SINGLE_CHAR_OPERATORS: &'static &str = &"\\().,";
const KEYWORDS: &'static [&'static str] = &[
    "if", 
    "else",
    "while",
];

/// Simple `String` to `Vec<Token>` converter.
/// 
/// The `TokenReader` reads all chars from string and creates an list of tokens after calling the [`parse`] method.
/// 
/// [`parse`]: TokenReader::parse
pub struct TokenReader {
    state: Cell<State>,
}

#[derive(Copy, Clone, Debug)]
struct State {
    expected: Expected,
    start_offset: usize,
    is_ready_to_push: bool,
    is_start_of_line: bool,
    prev_operator_char: Option<char>,
    is_inside_string: bool,
    is_prev_escape_symbol: bool,
}

#[derive(Copy, Clone, Debug)]
enum Expected {
    Nothing,
    IntNumber,
    FloatNumber,
    StringConstant,
    Identifier,
    Operator,
    Indent,
    Newline,
}

#[derive(Debug)]
pub struct SyntaxError { pub pos: usize, pub message: String }

impl TokenReader {

    /// Creates an new `TokenReader` from source code string.
    /// 
    /// To get tokens from source use the `parse` method.
    /// 
    /// # Examples
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// let source_code = get_source_code();
    /// let token_reader = TokenReader::new();
    /// ```
    pub fn new() -> TokenReader {
        TokenReader {
            state: Cell::from(State {
                start_offset: 0,
                expected: Expected::Nothing,
                is_ready_to_push: false,
                is_start_of_line: false,
                prev_operator_char: None,
                is_inside_string: false,
                is_prev_escape_symbol: false,
            })
        }
    }

    /// Creates an `Vec<Token>` from source string.
    /// 
    /// This is an expensive operation, please cache the results of its work.
    /// 
    /// Also you may check [`Token`] and [`TokenType`].
    /// 
    /// [`Token`]: crate::dto::Token
    /// [`TokenType`]: crate::dto::Token
    /// 
    /// # Examples
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// let token_reader = TokenReader::new();
    /// let tokens = token_reader.parse(String::from("2 + 2")).unwrap();
    /// 
    /// assert_eq!(
    ///     vec![
    ///         Token { token_type: TokenType::Number, payload: "2", pos: 0 },
    ///         Token { token_type: TokenType::Operator, payload: "+", pos: 1 },
    ///         Token { token_type: TokenType::Number, payload: "2", pos: 2 },
    ///     ],
    ///     tokens,
    /// );
    /// ```
    pub fn parse(&self, source: &String) -> Result<Vec<Token>, SyntaxError> {
        let mut iter = source.chars();
        let mut offset = 0usize;
        let mut tokens: Vec<Token> = vec!();
        let mut new_char = None;
        loop {
            if !self.state.get().is_ready_to_push {
                new_char = iter.next();
                offset += 1;
            }
            push_token_if_ready(&self.state, source, offset, &mut tokens);
            match new_char {
                Some(val) => {
                    match reduce_state(val, offset - 1, self.state.get()) {
                        Ok(new_state) => self.state.set(new_state),
                        Err(e) => return Err(e), 
                    }
                },
                None => break,
            };
        }
        self.state.set(State { is_ready_to_push: true, ..self.state.get() });
        push_token_if_ready(&self.state, source, offset, &mut tokens);
        Ok(tokens)
    }
}

fn push_token_if_ready(state_cell: &Cell<State>, source: &String, offset: usize, tokens: &mut Vec<Token>) {
    let state = state_cell.get();
    if state.is_ready_to_push {
        let start = state.start_offset;
        let end = offset - 1;
        let token_content = String::from(&source[start..end]);
        match state.expected {
            Expected::IntNumber => tokens.push(Token::IntConstant { value: token_content.parse().unwrap(), pos: offset }),
            Expected::FloatNumber => tokens.push(Token::FloatConstant { value: token_content.parse().unwrap(), pos: offset }),
            Expected::StringConstant => {
                let token_content = String::from(&source[(start + 1)..(end - 1)]);
                tokens.push(Token::StringConstant { value: token_content, pos: start })
            },
            Expected::Identifier => tokens.push(get_keyword_or_identifier(token_content, start)),
            Expected::Operator => tokens.push(Token::Operator { payload: token_content, pos: start }),
            Expected::Newline => tokens.push(Token::NewLine { pos: offset }),
            Expected::Indent => tokens.push(Token::Indent { depth: end - start, pos: offset }),
            Expected::Nothing => { /* no-op */ },
        };
        state_cell.set(State { 
            is_ready_to_push: false, 
            expected: Expected::Nothing,
            is_start_of_line: match state.expected { Expected::Newline => true, _ => false }, 
            prev_operator_char: None, 
            ..state
        });
    }
}

#[inline]
fn get_keyword_or_identifier(token_content: String, start: usize) -> Token {
    match token_content {
        val if KEYWORDS.iter().any(|k| k.to_string() == val) => Token::Operator { payload: val, pos: start },
        _ => Token::Identifier { name: token_content, pos: start }
    }
}

#[inline]
fn reduce_state(symbol: char, offset: usize, state: State) -> Result<State, SyntaxError> {
    match state.expected {
        Expected::Nothing => reduce_state_nothing(symbol, offset, state),
        Expected::IntNumber => reduce_state_int_number(symbol, offset, state),
        Expected::FloatNumber => reduce_state_float_number(symbol, offset, state),
        Expected::StringConstant => reduce_state_string_constant(symbol, state),
        Expected::Identifier => reduce_state_identifier(symbol, state),
        Expected::Operator => reduce_state_operator(symbol, state),
        Expected::Indent => reduce_state_whitespace(symbol, state),
        Expected::Newline => reduce_state_newline(symbol, state),
    }
}

#[inline]
fn reduce_state_nothing(symbol: char, offset: usize, state: State) -> Result<State, SyntaxError> {
    match symbol {
        val if val.is_digit(10) => Ok(State { expected: Expected::IntNumber, start_offset: offset, ..state }),
        val if val.is_alphabetic() => Ok(State { expected: Expected::Identifier, start_offset: offset, ..state }),
        '\n' => Ok(State { expected: Expected::Newline, start_offset: offset, ..state }),
        val if val.is_whitespace() => match state.is_start_of_line {
            true => Ok(State { expected: Expected::Indent, start_offset: offset, ..state }),
            false => Ok(State { start_offset: offset, ..state }),
        },
        val if OPERATORS.chars().any(|s| s == val) =>
            Ok(State { expected: Expected::Operator, start_offset: offset, ..state }),
        '"' => Ok(State { expected: Expected::StringConstant, is_inside_string: true, start_offset: offset, ..state }),
        _ => Err(SyntaxError { pos: offset, message: format!("Unexpected symbol {:?}", symbol) }),
    }
}

#[inline]
fn reduce_state_int_number(symbol: char, offset: usize, state: State) -> Result<State, SyntaxError> {
    match symbol {
        val if val.is_digit(10) => Ok(state),
        val if val.is_alphabetic() => Err(SyntaxError { pos: offset, message: format!("Invalid character in integer number record: {:?}", symbol) }),
        '.' => Ok(State { expected: Expected::FloatNumber, ..state }),
        _ => Ok(State { is_ready_to_push: true, ..state }),
    }
}

#[inline]
fn reduce_state_float_number(symbol: char, offset: usize, state: State) -> Result<State, SyntaxError> {
    match symbol {
        val if val.is_digit(10) => Ok(state),
        val if val.is_alphabetic() => Err(SyntaxError { pos: offset, message: format!("Invalid character in floating point number record: {:?}", symbol) }),
        _ => Ok(State { is_ready_to_push: true, ..state }),
    }
}

#[inline]
fn reduce_state_identifier(symbol: char, state: State) -> Result<State, SyntaxError> {
    let new_state = match symbol {
        val if val.is_alphanumeric() => state,
        '_' => state,
        _ => State { is_ready_to_push: true, ..state },
    };
    Ok(new_state)
}

#[inline]
fn reduce_state_operator(symbol: char, state: State) -> Result<State, SyntaxError> {
    let new_state = match state.prev_operator_char {
        None => match symbol {
            val if SINGLE_CHAR_OPERATORS.chars().any(|s| s == val) => 
                State { is_ready_to_push: true, prev_operator_char: None, ..state },
            val if OPERATORS.chars().any(|s| s == val) =>
                State { prev_operator_char: Some(val), ..state },
            _ => 
                State { is_ready_to_push: true, prev_operator_char: None, ..state },
        },
        Some(_) => State { is_ready_to_push: true, prev_operator_char: None, ..state }
    };
    Ok(new_state)
}

#[inline]
fn reduce_state_whitespace(symbol: char, state: State) -> Result<State, SyntaxError> {
    let new_state = match symbol {
        val if val.is_whitespace() => state,
        _ => State { is_ready_to_push: true, ..state },
    };
    Ok(new_state)
}

#[inline]
fn reduce_state_newline(symbol: char, state: State) -> Result<State, SyntaxError> {
    let new_state = match symbol {
        '\n' => state,
        _ => State { is_ready_to_push: true, ..state },
    };
    Ok(new_state)
}

#[inline]
fn reduce_state_string_constant(symbol: char, state: State) -> Result<State, SyntaxError> {
    let new_state = match state.is_prev_escape_symbol {
        false => match symbol {
            '"' => State { is_inside_string: false, ..state },
            '\\' => State { is_prev_escape_symbol: true, ..state },
            _ => match state.is_inside_string {
                true => state,
                false => State { is_ready_to_push: true, ..state },
            },
        },
        true => State { is_prev_escape_symbol: false, ..state },
    };
    Ok(new_state)
}
