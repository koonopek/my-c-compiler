use std::fmt;
use std::process::Command;
use std::{char, env::args, fs, iter::Peekable, str::Chars};

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

#[derive(PartialEq, Debug, Clone)]
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
    Comment,
}

#[derive(PartialEq, Debug, Clone)]
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
    // fn return_keyword() -> Token {
    //     Token {
    //         kind: TokenKind::Return,
    //         value: vec!['r', 'e', 't', 'u', 'r', 'n'],
    //     }
    // }
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
            Some('/') if chars.peek() == Some(&'/') => collect_comment('/', &mut chars),
            Some('/') if chars.peek() == Some(&'*') => collect_block_comment('/', &mut chars),
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

fn chars_to_usize(chars: &[char]) -> Result<u64, ParserErr> {
    let mut value = 0u64;
    for c in chars {
        let digit = if let Some(d) = c.to_digit(10) {
            d as u64
        } else {
            return Err(ParserErr(format!("invalid digit {:?}", chars)));
        };
        value = value * 10 + digit;
    }
    Ok(value)
}

// AST
#[derive(Debug, PartialEq)]
enum Exp {
    Const(u64),
}

impl Exp {
    fn to_code(self: &Self) -> Result<String, GenerateError> {
        match self {
            Exp::Const(exp) => return Ok(format!("mov w0, #{}", exp)),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Statement {
    Return(Exp),
}

impl Statement {
    fn from(cursor: &mut TokensCursor) -> Result<Statement, ParserErr> {
        cursor.expect(TokenKind::Return)?;
        let int = cursor.expect(TokenKind::Integer)?;
        let int_value = chars_to_usize(&int.value)?;
        cursor.expect(TokenKind::Semicolon)?;

        return Ok(Statement::Return(Exp::Const(int_value)));
    }

    fn to_code(self: &Self) -> Result<String, GenerateError> {
        match self {
            Statement::Return(exp) => {
                let exp_code = exp.to_code()?;

                return Ok(format!(
                    "{exp_code}
    ret"
                ));
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct Func {
    name: String,
    return_type: Type,
    // args
    // return type?
    statement: Statement,
}

#[derive(Debug, PartialEq)]
enum Type {
    Int,
}

impl Func {
    fn from(cursor: &mut TokensCursor) -> Result<Func, ParserErr> {
        let return_type = cursor.expect(TokenKind::Identifier)?;
        // at the moment we accept only int, later we can create some class wrapper like Type
        if return_type.value != ['i', 'n', 't'] {
            return Err(ParserErr(format!(
                "unsupported data type {:?}",
                return_type.value
            )));
        }

        let name = cursor.expect(TokenKind::Identifier)?;

        cursor.expect(TokenKind::LeftParen)?;

        // add support for different args than void
        let void = cursor.expect(TokenKind::Identifier)?;
        if void.value != ['v', 'o', 'i', 'd'] {
            return Err(ParserErr(format!(
                "unsupported data type {:?}",
                return_type.value
            )));
        }

        cursor.expect(TokenKind::RightParen)?;
        cursor.expect(TokenKind::LeftBrace)?;

        let stm = Statement::from(cursor)?;
        cursor.expect(TokenKind::RightBrace)?;

        Ok(Func {
            return_type: Type::Int,
            name: name.value.iter().collect(),
            statement: stm,
        })
    }

    fn to_code(self: &Self) -> Result<String, GenerateError> {
        let func_name = format!("_{}", self.name);
        let stmt = self.statement.to_code()?;
        return Ok(format!(
            "{func_name}: 
    {stmt}
"
        ));
    }
}

#[derive(Debug)]
struct GenerateError(String);

fn generate_code(prog: Prog) -> Result<String, GenerateError> {
    let mut func_name = String::new();
    func_name.push_str("_");
    func_name.push_str(&prog.function.name);

    let preamble = format!(".global {func_name}");
    let func_code = prog.function.to_code()?;

    return Ok(format!(
        "{preamble}
{func_code}"
    ));
}

#[derive(Debug, PartialEq)]
struct Prog {
    function: Func,
}

#[derive(Debug)]
struct ParserErr(String);

struct TokensCursor<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> TokensCursor<'a> {
    fn new(tokens: &'a [Token]) -> TokensCursor<'a> {
        TokensCursor {
            tokens,
            position: 0,
        }
    }

    // fn peek(&self) -> Option<&'a Token> {
    //     self.tokens.get(self.position)
    // }

    fn next(&mut self) -> Option<&'a Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&'a Token, ParserErr> {
        let t = self.next();

        match t {
            Some(t) => {
                if t.kind == kind {
                    return Ok(t);
                }
                return Err(ParserErr(format!("Expected {:?} find {:?}", kind, t.kind)));
            }
            None => return Err(ParserErr(format!("Expected {:?} find EOF", kind))),
        }
    }

    // fn expect_one_of(&mut self, kind: Vec<TokenKind>) -> Result<&'a Token, ParserErr> {
    //     let t = self.next();

    //     match t {
    //         Some(t) => {
    //             if kind.contains(&t.kind) {
    //                 return Ok(t);
    //             }
    //             return Err(ParserErr(format!("Expected {:?} find {:?}", kind, t.kind)));
    //         }
    //         None => return Err(ParserErr(format!("Expected {:?} find EOF", kind))),
    //     }
    // }
}

fn ast(mut cursor: TokensCursor) -> Result<Prog, ParserErr> {
    let prog = Prog {
        function: Func::from(&mut cursor)?,
    };

    // hmm
    cursor.expect(TokenKind::End)?;

    return Ok(prog);
}

fn main() {
    let cli_args: Vec<String> = args().skip(1).collect();
    if cli_args.is_empty() {
        eprintln!(
            "usage: my-c-compiler
  <input.c> [output]"
        );
        std::process::exit(1);
    }
    let input_path = &cli_args[0];
    let output_path = cli_args
        .get(1)
        .cloned()
        .unwrap_or_else(|| input_path.strip_suffix(".c").unwrap_or(input_path).to_string());
    let asm_path = format!("{}.s", output_path);

    let to_compile = fs::read_to_string(input_path)
        .expect(format!("failed to read file {}", input_path).as_ref());

    let tokens = lex(to_compile).unwrap_or_else(|err| {
        eprintln!("Lexer error: {}", err.msg);
        std::process::exit(1);
    });

    println!("LEXING ...");
    for token in &tokens {
        println!("{}", token);
    }

    println!("PARSING ...",);
    let cursor = TokensCursor::new(&tokens);
    let ast_tree = ast(cursor).unwrap_or_else(|err| {
        eprintln!("Parser error: {}", err.0);
        std::process::exit(1);
    });

    // println!("AST TREE");
    // println!("{:?}", ast_tree);
    println!("GENERATING ...",);

    let asm = generate_code(ast_tree).unwrap_or_else(|err| {
        eprintln!("Generating error: {}", err.0);
        std::process::exit(1);
    });

    println!("Compiled:");
    println!("{}", asm);
    fs::write(&asm_path, asm).expect("failed to write assembly file");

    let status = Command::new("gcc")
        .arg(&asm_path)
        .arg("-o")
        .arg(&output_path)
        .status()
        .expect("failed to run gcc");

    if !status.success() {
        std::process::exit(1);
    }
}
