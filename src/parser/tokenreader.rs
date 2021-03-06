use crate::dto::{ Token, SyntaxError };
use std::cell::Cell;

const OPERATORS: &'static &str = &"{}=+-*/^\\().,<>:@";
const KEYWORDS: &'static [&'static str] = &[
    // flow
    "if", 
    "else",
    "loop",
    "for",
    "in",
    "match",
    // variables
    "mut",
    // logical and bits
    "and",
    "or",
    "xor",
    "not",
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
    // commons
    expected: Expected,
    start_offset: usize,
    is_ready_to_push: bool,
    is_prev_escape_symbol: bool,
    // identifiers
    identifier_is_function: bool,
    // comments
    is_inside_block_comment: bool,
    // strings
    is_inside_string: bool,
    // numbers
    is_percent_float: bool,
}

#[derive(Copy, Clone, Debug)]
enum Expected {
    Nothing,
    IntNumber,
    FloatNumber,
    StringConstant,
    Identifier,
    Operator,
    Newline,
    BlockComment,
    LineComment,
}

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
                // commons
                start_offset: 0,
                expected: Expected::Nothing,
                is_ready_to_push: false,
                is_prev_escape_symbol: false,
                // identifiers
                identifier_is_function: false,
                //comments
                is_inside_block_comment: false,
                // strings
                is_inside_string: false,
                // numbers
                is_percent_float: false,
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
    /// let tokens = token_reader.parse(String::from("2+2")).unwrap();
    /// 
    /// assert_eq!(
    ///     vec![
    ///         Token::IntConstant { value: 2, pos: 0 },
    ///         Token::Operator { payload: String::from("+"), pos: 1 },
    ///         Token::IntConstant { value: 2, pos: 2 },
    ///     ],
    ///     tokens,
    /// );
    /// ```
    pub fn parse(&self, source: &String) -> Result<Vec<Token>, SyntaxError> {
        let mut iter = source.chars();
        let mut offset = 0usize;
        let mut tokens: Vec<Token> = vec!();
        let mut current_char = None;
        let mut prev_char = '\n';
        let mut it = 0;
        loop {
            if !self.state.get().is_ready_to_push {
                prev_char = match current_char {
                    Some(val) => val,
                    None => '\n',
                };
                current_char = iter.next();
                offset += 1;
            }
            match push_token_if_ready(&self.state, source, offset, &mut tokens) {
                Ok(_) => { /* no-op */ }
                Err(e) => return Err(e)
            };
            match current_char {
                Some(val) => {
                    match reduce_state(val, prev_char, offset - 1, self.state.get()) {
                        Ok(new_state) => self.state.set(new_state),
                        Err(e) => return Err(e), 
                    }
                },
                None => break,
            };
            it += 1;
            if it > 1000 {
                panic!()
            }
        }
        self.state.set(State { is_ready_to_push: true, ..self.state.get() });
        match push_token_if_ready(&self.state, source, offset, &mut tokens) {
            Ok(_) => { /* no-op */ }
            Err(e) => return Err(e)
        };
        Ok(tokens)
    }
}

fn push_token_if_ready(state_cell: &Cell<State>, source: &String, offset: usize, tokens: &mut Vec<Token>) -> Result<(), SyntaxError> {
    let state = state_cell.get();
    if state.is_ready_to_push {
        let start = state.start_offset;
        let end = offset - 1;
        let token_content = String::from(&source[start..end]);
        let result = match state.expected {
            Expected::IntNumber => match token_content.parse() {
                Ok(int_value) => Ok(tokens.push(Token::IntConstant { value: int_value, pos: start })),
                Err(_) => Err(SyntaxError { pos: start, message: format!("Can't parse int number {}", token_content) }),
            },
            Expected::FloatNumber => push_float_token_if_ready(&state, source, start, end, tokens, &token_content),
            Expected::StringConstant => {
                let token_content = String::from(&source[(start + 1)..(end - 1)]);
                Ok(tokens.push(Token::StringConstant { value: token_content, pos: start }))
            },
            Expected::Identifier => Ok(tokens.push(get_keyword_or_identifier(token_content, start, state.identifier_is_function))),
            Expected::Operator => Ok(tokens.push(Token::Operator { payload: token_content, pos: start })),
            Expected::Newline => Ok(tokens.push(Token::NewLine { pos: start })),
            Expected::BlockComment => Ok(()),
            Expected::LineComment => Ok(()),
            Expected::Nothing => Ok(()),
        };
        state_cell.set(State { 
            is_ready_to_push: false, 
            expected: Expected::Nothing,
            is_percent_float: false,
            ..state
        });
        result
    } else {
        Ok(())
    }
}

#[inline]
fn push_float_token_if_ready(
    state: &State, 
    source: &String, 
    start: usize, 
    end: usize,
    tokens: &mut Vec<Token>,
    token_content: &String,
) -> Result<(), SyntaxError> {
    match state.is_percent_float {
        true => {
            let token_content = String::from(&source[start..(end - 1)]);
            match token_content.parse() {
                Ok(float_value) => Ok(tokens.push(Token::FloatConstant { value: get_percent_float(float_value), pos: start })),
                Err(_) => Err(SyntaxError { pos: start, message: format!("Can't parse percentage number {}%", token_content) }),
            }
            
        }
        false => match token_content.parse() {
            Ok(float_value) => Ok(tokens.push(Token::FloatConstant { value: float_value, pos: start })),
            Err(_) => Err(SyntaxError { pos: start, message: format!("Can't parse float number {}", token_content) }),
        },
    }
}

#[inline]
fn get_percent_float(float_value: f32) -> f32 { float_value / 100.0 }

#[inline]
fn get_keyword_or_identifier(
    token_content: String, 
    start: usize,
    maybe_identifier_is_function: bool,
) -> Token {
    match token_content {
        val if KEYWORDS.iter().any(|k| k.to_string() == val) => Token::Operator { payload: val, pos: start },
        _ => match maybe_identifier_is_function {
            false => Token::Identifier { name: token_content, pos: start },
            true => Token::Function { name: token_content, pos: start },
        }
    }
}

#[inline]
fn reduce_state(symbol: char, prev_symbol: char, offset: usize, state: State) -> Result<State, SyntaxError> {
    match state.expected {
        Expected::Nothing => reduce_state_nothing(symbol, offset, state),
        Expected::IntNumber => reduce_state_int_number(symbol, offset, state),
        Expected::FloatNumber => reduce_state_float_number(symbol, offset, state),
        Expected::StringConstant => reduce_state_string_constant(symbol, state),
        Expected::Identifier => reduce_state_identifier(symbol, state),
        Expected::Operator => reduce_state_operator(symbol, prev_symbol, state),
        Expected::Newline => reduce_state_newline(symbol, state),
        Expected::BlockComment => reduce_state_block_comment(symbol, prev_symbol, state),
        Expected::LineComment => reduce_state_line_comment(symbol, state),
    }
}

#[inline]
fn reduce_state_nothing(symbol: char, offset: usize, state: State) -> Result<State, SyntaxError> {
    match symbol {
        val if val.is_digit(10) => Ok(State { expected: Expected::IntNumber, start_offset: offset, ..state }),
        val if val.is_alphabetic() => Ok(State { expected: Expected::Identifier, start_offset: offset, ..state }),
        '\n' => Ok(State { expected: Expected::Newline, start_offset: offset, ..state }),
        val if val.is_whitespace() => Ok(State { start_offset: offset, ..state }),
        val if OPERATORS.chars().any(|s| s == val) => 
            Ok(State { expected: Expected::Operator, start_offset: offset, ..state }),
        '"' => Ok(State { expected: Expected::StringConstant, is_inside_string: true, start_offset: offset, ..state }),
        '_' => Err(SyntaxError { pos: offset, message: String::from("Identifier names must not start with an underscore") }),
        _ => Err(SyntaxError { pos: offset, message: format!("Unexpected symbol '{}'", symbol) }),
    }
}

#[inline]
fn reduce_state_int_number(symbol: char, offset: usize, state: State) -> Result<State, SyntaxError> {
    match symbol {
        val if val.is_digit(10) => Ok(state),
        val if val.is_alphabetic() => Err(SyntaxError { pos: offset, message: format!("Invalid character in integer number record: {:?}", symbol) }),
        '%' => Ok(State { expected: Expected::FloatNumber, is_percent_float: true, ..state }),
        '.' => Ok(State { expected: Expected::FloatNumber, ..state }),
        _ => Ok(State { is_ready_to_push: true, ..state }),
    }
}

#[inline]
fn reduce_state_float_number(symbol: char, offset: usize, state: State) -> Result<State, SyntaxError> {
    match symbol {
        val if val.is_digit(10) => Ok(state),
        val if val.is_alphabetic() => Err(SyntaxError { pos: offset, message: format!("Invalid character in floating point number record: {:?}", symbol) }),
        '%' => match state.is_percent_float { 
            false => Ok(State { is_percent_float: true, ..state }),
            true => Err(SyntaxError { pos: offset, message: String::from("You cannot use the percent symbol twice on the same number") })
        },
        _ => Ok(State { is_ready_to_push: true, ..state }),
    }
}

#[inline]
fn reduce_state_identifier(symbol: char, state: State) -> Result<State, SyntaxError> {
    let new_state = match symbol {
        val if val.is_alphanumeric() => state,
        '_' => state,
        '(' | '{' => State { is_ready_to_push: true, identifier_is_function: true, ..state },
        _ => State { is_ready_to_push: true, ..state },
    };
    Ok(new_state)
}

#[inline]
fn reduce_state_operator(symbol: char, prev_symbol: char, state: State) -> Result<State, SyntaxError> {
    let new_state = match prev_symbol {
        '+' | '-' | '*' | '/' | '=' | '!' | '<' | '>' => match symbol {
            '=' => state,
            '>' => State { is_ready_to_push: prev_symbol != '-', ..state },
            '/' => match prev_symbol {
                '/' => State { expected: Expected::LineComment, ..state },
                _ => State { is_ready_to_push: true, ..state },
            },
            '*' => match prev_symbol {
                '/' => State { expected: Expected::BlockComment, is_inside_block_comment: true, ..state },
                _ => State { is_ready_to_push: true, ..state },
            },
            _ => State { is_ready_to_push: true, ..state },
        },
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

#[inline]
fn reduce_state_block_comment(symbol: char, prev_symbol: char, state: State) -> Result<State, SyntaxError> {
    let new_state = match prev_symbol {
        '*' => match symbol {
            '/' => State { is_inside_block_comment: false, ..state },
            _ => state,
        },
        _ => match state.is_inside_block_comment {
            true => state,
            false => State { is_ready_to_push: true, ..state },
        },
    };
    Ok(new_state)
}

#[inline]
fn reduce_state_line_comment(symbol: char, state: State) -> Result<State, SyntaxError> {
    match symbol {
        '\n' => Ok(State { is_ready_to_push: true, ..state }),
        _ => Ok(state),
    }
}

/// Testing the correct finding of string literals.
/// 
/// String literals in Avo can only be created with qoutes: "this is my string".
/// Multiline strings and strings with pattern formatting not supported.
#[test]
fn test_string_literals() {
    let source = String::from("\"hello world\"\n\"\\\"quoted hello world\\\"\"");
    let expected = vec!(
        Token::StringConstant { value: String::from("hello world"), pos: 0 },
        Token::NewLine { pos: 13 },
        Token::StringConstant { value: String::from("\\\"quoted hello world\\\""), pos: 14 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}

/// Testing the correct finding of integer literals.
#[test]
fn test_integer_literals() {
    let source = String::from("1+22*333/44^5-678");
    let expected = vec!(
        Token::IntConstant { value: 1, pos: 0 },
        Token::Operator { payload: String::from("+"), pos: 1 },
        Token::IntConstant { value: 22, pos: 2 },
        Token::Operator { payload: String::from("*"), pos: 4 },
        Token::IntConstant { value: 333, pos: 5 },
        Token::Operator { payload: String::from("/"), pos: 8 },
        Token::IntConstant { value: 44, pos: 9 },
        Token::Operator { payload: String::from("^"), pos: 11 },
        Token::IntConstant { value: 5, pos: 12 },
        Token::Operator { payload: String::from("-"), pos: 13 },
        Token::IntConstant { value: 678, pos: 14 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}

/// Testing the correct finding of integer literals.
/// 
/// Float numbers are specified using dot: 1.0, 34.56 etc.
#[test]
fn test_float_literals() {
    let source = String::from("1.0+22*3./4.44^0.5-67.8");
    let expected = vec!(
        Token::FloatConstant { value: 1.0, pos: 0 },
        Token::Operator { payload: String::from("+"), pos: 3 },
        Token::IntConstant { value: 22, pos: 4 },
        Token::Operator { payload: String::from("*"), pos: 6 },
        Token::FloatConstant { value: 3.0, pos: 7 },
        Token::Operator { payload: String::from("/"), pos: 9},
        Token::FloatConstant { value: 4.44, pos: 10 },
        Token::Operator { payload: String::from("^"), pos: 14 },
        Token::FloatConstant { value: 0.5, pos: 15 },
        Token::Operator { payload: String::from("-"), pos: 18 },
        Token::FloatConstant { value: 67.8, pos: 19 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}

/// Testing the correct finding of formatted float literals.
/// 
/// Formatted float literals now is percents `%`, but in the future, perhaps not only percents.
/// 
/// # Example
/// 
/// `146% == 1.46`
#[test]
fn test_formatted_float_literals() {
    let source = String::from("146%\n0%\n100%\n5.%\n4.2%");
    let expected = vec!(
        Token::FloatConstant { value: 1.46, pos: 0 },
        Token::NewLine { pos: 4 },
        Token::FloatConstant { value: 0.0, pos: 5 },
        Token::NewLine { pos: 7 },
        Token::FloatConstant { value: 1.0, pos: 8 },
        Token::NewLine { pos: 12 },
        Token::FloatConstant { value: 0.05, pos: 13 },
        Token::NewLine { pos: 16 },
        Token::FloatConstant { value: 0.042, pos: 17 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}

/// Testing the correct finding of identifiers (variables and functions names)
/// 
/// Identifiers names matches `[a-zA-Z][a-zA-Z0-9_]*` regexp.
#[test]
fn test_identifiers() {
    let source = String::from("a foo bar2 x_yz123 functionName variableName");
    let expected = vec!(
        Token::Identifier { name: String::from("a"), pos: 0 },
        Token::Identifier { name: String::from("foo"), pos: 2 },
        Token::Identifier { name: String::from("bar2"), pos: 6 },
        Token::Identifier { name: String::from("x_yz123"), pos: 11 },
        Token::Identifier { name: String::from("functionName"), pos: 19 },
        Token::Identifier { name: String::from("variableName"), pos: 32 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}

/// Testing the correct finding of arithmetical operators
/// 
/// # Operators
/// - `+` addition
/// - `-` subtraction
/// - `*` multiplication
/// - `/` division
/// - `^` power
/// - `+=` add and assign
/// - `-=` subtract and assign
/// - `*=` multiply and assign
/// - `/=` divide and assign
/// - `=` assign
/// - `(`, `)` brackets
#[test]
fn test_arithmetical_operators() {
    let source = String::from("a+b-c*d/e^f+=(-=)(*=/=)a=b");
    let expected = vec!(
        Token::Identifier { name: String::from("a"), pos: 0 },
        Token::Operator { payload: String::from("+"), pos: 1 },
        Token::Identifier { name: String::from("b"), pos: 2 },
        Token::Operator { payload: String::from("-"), pos: 3 },
        Token::Identifier { name: String::from("c"), pos: 4 },
        Token::Operator { payload: String::from("*"), pos: 5 },
        Token::Identifier { name: String::from("d"), pos: 6 },
        Token::Operator { payload: String::from("/"), pos: 7 },
        Token::Identifier { name: String::from("e"), pos: 8 },
        Token::Operator { payload: String::from("^"), pos: 9 },
        Token::Identifier { name: String::from("f"), pos: 10 },
        Token::Operator { payload: String::from("+="), pos: 11 },
        Token::Operator { payload: String::from("("), pos: 13 },
        Token::Operator { payload: String::from("-="), pos: 14 },
        Token::Operator { payload: String::from(")"), pos: 16 },
        Token::Operator { payload: String::from("("), pos: 17 },
        Token::Operator { payload: String::from("*="), pos: 18 },
        Token::Operator { payload: String::from("/="), pos: 20 },
        Token::Operator { payload: String::from(")"), pos: 22 },
        Token::Identifier { name: String::from("a"), pos: 23 },
        Token::Operator { payload: String::from("="), pos: 24 },
        Token::Identifier { name: String::from("b"), pos: 25 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}

/// Testing the correct finding of logical (bool) and bit (int) operators
/// 
/// # Operators
/// - `and`
/// - `or`
/// - `xor`
/// - `not`
#[test]
fn test_logical_operators() {
    let source = String::from("(A and B) or not (C and not D)");
    let expected = vec!(
        Token::Operator { payload: String::from("("), pos: 0 },
        Token::Identifier { name: String::from("A"), pos: 1 },
        Token::Operator { payload: String::from("and"), pos: 3 },
        Token::Identifier { name: String::from("B"), pos: 7 },
        Token::Operator { payload: String::from(")"), pos: 8 },
        Token::Operator { payload: String::from("or"), pos: 10 },
        Token::Operator { payload: String::from("not"), pos: 13 },
        Token::Operator { payload: String::from("("), pos: 17 },
        Token::Identifier { name: String::from("C"), pos: 18 },
        Token::Operator { payload: String::from("and"), pos: 20 },
        Token::Operator { payload: String::from("not"), pos: 24 },
        Token::Identifier { name: String::from("D"), pos: 28 },
        Token::Operator { payload: String::from(")"), pos: 29 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}

/// Testing the correct finding of flow operators
/// 
/// # Operators
/// - `if`
/// - `else`
/// - `for`
/// - `in`
/// - `loop`
/// - `{`, `}`
/// - `->`
#[test]
fn test_flow_operators() {
    // if happy path
    let source = String::from("if a > b { foo() } else { bar() }");
    let expected = vec!(
        Token::Operator { payload: String::from("if"), pos: 0 },
        Token::Identifier { name: String::from("a"), pos: 3 },
        Token::Operator { payload: String::from(">"), pos: 5 },
        Token::Identifier { name: String::from("b"), pos: 7 },
        Token::Operator { payload: String::from("{"), pos: 9 },
        Token::Function { name: String::from("foo"), pos: 11 },
        Token::Operator { payload: String::from("("), pos: 14 },
        Token::Operator { payload: String::from(")"), pos: 15 },
        Token::Operator { payload: String::from("}"), pos: 17 },
        Token::Operator { payload: String::from("else"), pos: 19 },
        Token::Operator { payload: String::from("{"), pos: 24 },
        Token::Function { name: String::from("bar"), pos: 26 },
        Token::Operator { payload: String::from("("), pos: 29 },
        Token::Operator { payload: String::from(")"), pos: 30 },
        Token::Operator { payload: String::from("}"), pos: 32 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual);

    // loop happy path
    let source = String::from("loop { foo() }");
    let expected = vec!(
        Token::Operator { payload: String::from("loop"), pos: 0 },
        Token::Operator { payload: String::from("{"), pos: 5 },
        Token::Function { name: String::from("foo"), pos: 7 },
        Token::Operator { payload: String::from("("), pos: 10 },
        Token::Operator { payload: String::from(")"), pos: 11 },
        Token::Operator { payload: String::from("}"), pos: 13 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual);

    // loop happy path
    let source = String::from("for i in range");
    let expected = vec!(
        Token::Operator { payload: String::from("for"), pos: 0 },
        Token::Identifier { name: String::from("i"), pos: 4 },
        Token::Operator { payload: String::from("in"), pos: 6 },
        Token::Identifier { name: String::from("range"), pos: 9 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual);

    // loop happy path
    let source = String::from("match expr { a -> None }");
    let expected = vec!(
        Token::Operator { payload: String::from("match"), pos: 0 },
        Token::Identifier { name: String::from("expr"), pos: 6 },
        Token::Operator { payload: String::from("{"), pos: 11 },
        Token::Identifier { name: String::from("a"), pos: 13 },
        Token::Operator { payload: String::from("->"), pos: 15 },
        Token::Identifier { name: String::from("None"), pos: 18 },
        Token::Operator { payload: String::from("}"), pos: 23 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}

/// Testing for correct finding of other operators
/// 
/// # Operators
/// - `{`, `}`
/// - `@`, `->`
/// - `:`
/// - `,`
#[test]
fn test_other_operators() {
    let source = String::from("{ @ x, y -> x:0.1:y }");
    let expected = vec!(
        Token::Operator { payload: String::from("{"), pos: 0 },
        Token::Operator { payload: String::from("@"), pos: 2 },
        Token::Identifier { name: String::from("x"), pos: 4 },
        Token::Operator { payload: String::from(","), pos: 5 },
        Token::Identifier { name: String::from("y"), pos: 7 },
        Token::Operator { payload: String::from("->"), pos: 9 },
        Token::Identifier { name: String::from("x"), pos: 12 },
        Token::Operator { payload: String::from(":"), pos: 13 },
        Token::FloatConstant { value: 0.1, pos: 14 },
        Token::Operator { payload: String::from(":"), pos: 17 },
        Token::Identifier { name: String::from("y"), pos: 18 },
        Token::Operator { payload: String::from("}"), pos: 20 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}

/// Testing for ignoring line comments.
/// 
/// Line comments starts with `//` operator.
#[test]
fn test_line_comments() {
    let source = String::from("// comment 1
    identifier // comment 2
    // comment 3");
    let expected = vec!(
        Token::NewLine { pos: 12 },
        Token::Identifier { name: String::from("identifier"), pos: 17 },
        Token::NewLine { pos: 40 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}

/// Testing for ignoring block comments.
/// 
/// Block comment starts with `/*` and ends with `*/`.
#[test]
fn test_block_comments() {
    let source = String::from("/* 
    comment 1 
    this is comment 1
    */
    identifier1 /*no-op*/identifier2 /* comment 2 *//* kek */");
    let expected = vec!(
        Token::NewLine { pos: 47 },
        Token::Identifier { name: String::from("identifier1"), pos: 52 },
        Token::Identifier { name: String::from("identifier2"), pos: 73 },
    );
    let actual = TokenReader::new().parse(&source).unwrap();
    assert_eq!(expected, actual)
}