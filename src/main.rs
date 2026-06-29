use std::fmt;
use std::{char, env::args, error::Error, fs, iter::Peekable, str::Chars};

// const KEYWORDS: [&str; 3] = [
//     Open brace {
// Close brace }
// Open parenthesis \(
// Close parenthesis \)
// Semicolon ;
// Int keyword int
// Return keyword return
// Identifier [a-zA-Z]\w*
// Integer literal [0-9]+
// ];
//

const DELIMITER: [char; 5] = [
    '{', '}', '(', ')',
    ';', // Int keyword int
        // Return keyword return
        // Identifier [a-zA-Z]\w*
        // Integer literal [0-9]+
];

#[derive(PartialEq, Debug)]
enum TokenKind {
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Semicolon,
    Whitespace,
    Return,
    Integer,
    Identifier,
    // maybe stupid
    End,
}

#[derive(PartialEq, Debug)]
struct Token {
    kind: TokenKind,
    value: Vec<char>,
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
    fn return_keyword() -> Token {
        Token {
            kind: TokenKind::Return,
            value: vec!['r', 'e', 't', 'u', 'r', 'n'],
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
            value: vec![],
        }
    }
}

#[derive(Debug)]
struct LexerErr {
    msg: String,
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

fn collect_alphabetic(prev_char: char, chars: &mut Peekable<Chars>) -> Result<Token, LexerErr> {
    let mut word: Vec<char> = vec![prev_char];

    return loop {
        let c_opt = chars.peek();

        match c_opt {
            Some(c) if !DELIMITER.contains(c) && !c.is_whitespace() => {
                // println!("will consume {:?}", chars.peek());
                // we are sure we can unwrapped cause we have peek'ed and checked
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
                // println!("will consume {:?}", chars.peek());
                // we are sure we can unwrapped cause we have peek'ed and checked
                word.push(*c);
                chars.next();
            }
            Some(_) => break Ok(Token::integer(word)),
            None => break Ok(Token::integer(word)),
        }
    };
}

fn lex(to_lex: String) -> Result<Vec<Token>, LexerErr> {
    let mut chars = to_lex.chars().peekable();

    let mut tokens: Vec<Token> = Vec::new();

    loop {
        let token = match chars.next() {
            Some('{') => Ok(Token::left_brace()),
            Some('}') => Ok(Token::right_brace()),
            Some('(') => Ok(Token::left_parent()),
            Some(')') => Ok(Token::right_parent()),
            Some(';') => Ok(Token::semicolon()),
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
                // lets skip whitespaces for now
                if t.kind == TokenKind::Whitespace {
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

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        eprintln!("usage: my-c-compiler <input.c>");
        std::process::exit(1);
    }
    let input_path = &args[1];

    let to_compile = fs::read_to_string(input_path)
        .expect(format!("failed to read file {}", input_path).as_ref());

    let tokens = lex(to_compile).unwrap();

    for token in &tokens {
        println!("{}", token);
    }

    // println!("Compiled:");
    // println!("{}", compiled);
    // fs::write("return_2.s", compiled).expect("failed to write to file");
}
