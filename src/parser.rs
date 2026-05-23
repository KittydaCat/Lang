use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, PartialEq)]
enum Token {
    Name(String),

    Number(f64),

    LParens,
    RParens,
    Comma,

    Plus,
    Minus,
    // String(String),
}

#[derive(Debug)]
pub enum Statement {
    Value(Value),
}

#[derive(Debug)]
pub enum Value {
    Function { name: String, args: Vec<Value> },
    Number(f64),
    // StringLit(String),
}

pub fn dew_it(file: &str) -> Vec<Statement> {
    parse(&mut dbg!(tokenize(file)).into_iter().peekable())
}

fn tokenize(file: &str) -> Vec<Token> {
    let mut cars = file.chars();

    let mut string = String::new();

    let mut tokens = Vec::new();

    loop {
        let next = cars.next();

        if let Some(c @ ('0'..'9' | 'a'..'z' | 'A'..'Z' | '_')) = next {
            string.push(c);
        } else if !string.is_empty() {
            if matches!(string.chars().next().unwrap(), '0'..'9') {
                tokens.push(Token::Number(string.parse().unwrap()));

                string = String::new();
            } else {
                tokens.push(Token::Name(string));

                string = String::new();
            }
        }

        match next {
            Some('0'..'9' | 'a'..'z' | 'A'..'Z' | '_') => {}

            Some('(') => tokens.push(Token::LParens),
            Some(')') => tokens.push(Token::RParens),

            Some(',') => tokens.push(Token::Comma),

            Some('\t' | '\n' | ' ') => {}

            // unknown
            Some(x) => {
                todo!("{x}")
            }

            None => {
                break;
            }
        }
    }

    tokens
}

fn parse(tokens: &mut Peekable<IntoIter<Token>>) -> Vec<Statement> {
    let mut statements = Vec::new();

    while let Some(_) = tokens.peek() {
        statements.push(Statement::Value(parse_value(tokens)));
    }

    statements
}

fn parse_value(tokens: &mut Peekable<IntoIter<Token>>) -> Value {
    match tokens.next().unwrap() {
        Token::Name(name) => {
            if let Some(Token::LParens) = tokens.peek() {
                Value::Function {
                    name,
                    args: parse_function_call(tokens),
                }
            } else {
                todo!("var probs")
            }
        }
        Token::Number(x) => Value::Number(x),
        Token::Comma | Token::LParens | Token::RParens => unimplemented!(),

        _ => {
            todo!()
        }
    }
}

fn parse_function_call(tokens: &mut Peekable<IntoIter<Token>>) -> Vec<Value> {
    let mut args = Vec::new();

    assert_eq!(tokens.next(), Some(Token::LParens));

    while *tokens.peek().unwrap() != Token::RParens {
        args.push(parse_value(tokens));

        match tokens.peek().unwrap() {
            Token::Comma => {
                assert_eq!(tokens.next(), Some(Token::Comma));
            }

            Token::RParens => {}

            _ => unimplemented!(),
        }
    }

    assert_eq!(tokens.next(), Some(Token::RParens));

    args
}
