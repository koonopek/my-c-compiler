use std::fmt;
use std::{iter::Peekable, str::Chars};

const DELIMITER: [char; 10] = ['{', '}', '(', ')', ';', '+', '*', '/', '-', '%'];

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind {
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Semicolon,
    Neg,
    BitwiseComplement,
    LogicalNeg,
    Mul,
    Add,
    Div,
    Whitespace,
    Return,
    Integer,
    Identifier,
    Mod,
    // maybe stupid
    End,
    Comment,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: Vec<char>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value: String = self.value.iter().collect();
        write!(f, "{:?} {}", self.kind, value)
    }
}

impl Token {
    fn left_brace() -> Token {
        Token {
            kind: TokenKind::LeftBrace,
            value: vec!['{'],
        }
    }
    fn right_brace() -> Token {
        Token {
            kind: TokenKind::RightBrace,
            value: vec!['}'],
        }
    }
    fn left_parent() -> Token {
        Token {
            kind: TokenKind::LeftParen,
            value: vec!['('],
        }
    }
    fn right_parent() -> Token {
        Token {
            kind: TokenKind::RightParen,
            value: vec![')'],
        }
    }
    fn semicolon() -> Token {
        Token {
            kind: TokenKind::Semicolon,
            value: vec![';'],
        }
    }
    fn neg() -> Token {
        Token {
            kind: TokenKind::Neg,
            value: vec!['-'],
        }
    }
    fn bitwise_complement() -> Token {
        Token {
            kind: TokenKind::BitwiseComplement,
            value: vec!['~'],
        }
    }
    fn logical_neg() -> Token {
        Token {
            kind: TokenKind::LogicalNeg,
            value: vec!['!'],
        }
    }
    fn mul() -> Token {
        Token {
            kind: TokenKind::Mul,
            value: vec!['*'],
        }
    }
    fn add() -> Token {
        Token {
            kind: TokenKind::Add,
            value: vec!['+'],
        }
    }
    fn div() -> Token {
        Token {
            kind: TokenKind::Div,
            value: vec!['/'],
        }
    }
    fn modulo() -> Token {
        Token {
            kind: TokenKind::Mod,
            value: vec!['%'],
        }
    }
    fn identifier(value: Vec<char>) -> Token {
        Token {
            kind: TokenKind::Identifier,
            value,
        }
    }
    fn integer(value: Vec<char>) -> Token {
        Token {
            kind: TokenKind::Integer,
            value,
        }
    }
    fn end() -> Token {
        Token {
            kind: TokenKind::End,
            value: vec![],
        }
    }

    fn whitespace(value: Vec<char>) -> Token {
        Token {
            kind: TokenKind::Whitespace,
            value,
        }
    }
    fn comment(value: Vec<char>) -> Token {
        Token {
            kind: TokenKind::Comment,
            value,
        }
    }
}

#[derive(Debug)]
pub struct LexerErr {
    pub msg: String,
}

impl LexerErr {
    fn from_msg(msg: String) -> LexerErr {
        return LexerErr { msg };
    }
}

fn is_special_identifier(word: &Vec<char>) -> Option<TokenKind> {
    match word.as_slice() {
        ['r', 'e', 't', 'u', 'r', 'n'] => Some(TokenKind::Return),
        _ => None,
    }
}

fn collect_comment(prev_char: char, chars: &mut Peekable<Chars>) -> Result<Token, LexerErr> {
    let mut word: Vec<char> = vec![prev_char];

    return loop {
        let c_opt = chars.peek();

        match c_opt {
            Some(c) if *c != '\n' => {
                word.push(*c);
                chars.next();
            }
            Some(_) | None => {
                break Ok(Token::comment(word));
            }
        }
    };
}

fn collect_block_comment(prev_char: char, chars: &mut Peekable<Chars>) -> Result<Token, LexerErr> {
    let mut word: Vec<char> = vec![prev_char];
    let mut prev = prev_char;
    loop {
        match chars.next() {
            Some(c) => {
                word.push(c);
                if prev == '*' && c == '/' {
                    break Ok(Token::comment(word));
                }
                prev = c;
            }
            None => {
                break Err(LexerErr::from_msg(String::from(
                    "unfinished block comment missing */",
                )));
            }
        }
    }
}

fn collect_alphabetic(prev_char: char, chars: &mut Peekable<Chars>) -> Result<Token, LexerErr> {
    let mut word: Vec<char> = vec![prev_char];

    return loop {
        let c_opt = chars.peek();

        match c_opt {
            Some(c) if !DELIMITER.contains(c) && !c.is_whitespace() => {
                word.push(*c);
                chars.next();
            }
            Some(_) | None => {
                let token = match is_special_identifier(&word) {
                    Some(kind) => Token { kind, value: word },
                    None => Token::identifier(word),
                };

                break Ok(token);
            }
        }
    };
}

fn collect_numeric(prev_char: char, chars: &mut Peekable<Chars>) -> Result<Token, LexerErr> {
    let mut word: Vec<char> = vec![prev_char];

    return loop {
        let c_opt = chars.peek();

        match c_opt {
            Some(c) if !DELIMITER.contains(c) && !c.is_whitespace() => {
                if !c.is_numeric() {
                    break Err(LexerErr::from_msg(format!("{} is not valid numeric", c)));
                }
                word.push(*c);
                chars.next();
            }
            Some(_) => break Ok(Token::integer(word)),
            None => break Ok(Token::integer(word)),
        }
    };
}

pub fn lex(to_lex: String) -> Result<Vec<Token>, LexerErr> {
    let mut chars = to_lex.chars().peekable();

    let mut tokens: Vec<Token> = Vec::new();

    loop {
        let token = match chars.next() {
            Some('{') => Ok(Token::left_brace()),
            Some('}') => Ok(Token::right_brace()),
            Some('(') => Ok(Token::left_parent()),
            Some(')') => Ok(Token::right_parent()),
            Some('-') => Ok(Token::neg()),
            Some('~') => Ok(Token::bitwise_complement()),
            Some('!') => Ok(Token::logical_neg()),
            Some('*') => Ok(Token::mul()),
            Some('+') => Ok(Token::add()),
            Some(';') => Ok(Token::semicolon()),
            Some('/') if chars.peek() == Some(&'/') => collect_comment('/', &mut chars),
            Some('/') if chars.peek() == Some(&'*') => collect_block_comment('/', &mut chars),
            Some('/') => Ok(Token::div()),
            Some('%') => Ok(Token::modulo()),
            Some(c) if c.is_whitespace() => Ok(Token::whitespace(vec![c])),
            Some(c) if c.is_alphabetic() => collect_alphabetic(c.clone(), &mut chars),
            Some(c) if c.is_numeric() => collect_numeric(c.clone(), &mut chars),
            None => Ok(Token::end()),
            Some(c) => Err(LexerErr::from_msg(format!("unknown char '{}' ", c))),
        };

        match token {
            Err(e) => return Err(e),
            Ok(t) => {
                let is_end = t.kind == TokenKind::End;
                // lets skip whitespaces
                if t.kind == TokenKind::Whitespace {
                    continue;
                }
                // lets skip comments
                if t.kind == TokenKind::Comment {
                    continue;
                }
                tokens.push(t);
                if is_end {
                    break;
                }
            }
        }
    }

    Ok(tokens)
}
