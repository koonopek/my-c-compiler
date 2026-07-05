use crate::lex::{Token, TokenKind};
use std::fmt;

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

#[derive(Debug, PartialEq)]
pub enum UnOp {
    Neg,
    BitwiseComplement,
    LogicalNeg,
}

impl UnOp {
    fn from(t: &Token) -> Result<UnOp, ParserErr> {
        return match t.kind {
            TokenKind::Neg => Ok(UnOp::Neg),
            TokenKind::BitwiseComplement => Ok(UnOp::BitwiseComplement),
            TokenKind::LogicalNeg => Ok(UnOp::LogicalNeg),
            _ => Err(ParserErr(format!("Invalid UnOp Token '{:?}'", t))),
        };
    }
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnOp::Neg => write!(f, "Neg"),
            UnOp::BitwiseComplement => write!(f, "BitwiseComplement"),
            UnOp::LogicalNeg => write!(f, "LogicalNeg"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl BinOp {}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Mod => write!(f, "Mod"),
            BinOp::Add => write!(f, "Add"),
            BinOp::Sub => write!(f, "Sub"),
            BinOp::Mul => write!(f, "Mul"),
            BinOp::Div => write!(f, "Div"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Const(u64),
    // unary operator
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
}

impl Expr {
    fn from(cursor: &mut TokensCursor) -> Result<Expr, ParserErr> {
        return Expr::parse_expr(cursor);
    }

    fn parse_expr(cursor: &mut TokensCursor) -> Result<Expr, ParserErr> {
        let mut term = Expr::parse_term(cursor)?;

        let result = loop {
            match cursor.peek() {
                Some(t) if t.kind == TokenKind::Add => {
                    cursor.expect(TokenKind::Add)?;
                    term = Expr::Binary(
                        BinOp::Add,
                        Box::new(term),
                        Box::new(Expr::parse_term(cursor)?),
                    );
                }
                Some(t) if t.kind == TokenKind::Neg => {
                    cursor.expect(TokenKind::Neg)?;
                    term = Expr::Binary(
                        BinOp::Sub,
                        Box::new(term),
                        Box::new(Expr::parse_term(cursor)?),
                    );
                }
                _ => break term,
            };
        };

        return Ok(result);
    }

    // <term> ::= <factor> { ("*" | "/") <factor> }
    fn parse_term(cursor: &mut TokensCursor) -> Result<Expr, ParserErr> {
        let mut factor = Expr::parse_factor(cursor)?;

        let result = loop {
            match cursor.peek() {
                Some(t) if t.kind == TokenKind::Mul => {
                    cursor.expect(TokenKind::Mul)?;
                    factor = Expr::Binary(
                        BinOp::Mul,
                        Box::new(factor),
                        Box::new(Expr::parse_factor(cursor)?),
                    );
                }
                Some(t) if t.kind == TokenKind::Div => {
                    cursor.expect(TokenKind::Div)?;
                    factor = Expr::Binary(
                        BinOp::Div,
                        Box::new(factor),
                        Box::new(Expr::parse_factor(cursor)?),
                    );
                }
                Some(t) if t.kind == TokenKind::Mod => {
                    cursor.expect(TokenKind::Mod)?;
                    factor = Expr::Binary(
                        BinOp::Mod,
                        Box::new(factor),
                        Box::new(Expr::parse_factor(cursor)?),
                    );
                }
                _ => break factor,
            };
        };

        return Ok(result);
    }

    // <factor> ::= "(" <exp> ")" | <unary_op> <factor> | <int>
    fn parse_factor(cursor: &mut TokensCursor) -> Result<Expr, ParserErr> {
        if cursor.peek_is(TokenKind::LeftParen) {
            cursor.expect(TokenKind::LeftParen)?;
            let expr = Expr::from(cursor)?;
            cursor.expect(TokenKind::RightParen)?;
            return Ok(expr);
        }

        let expr = Expr::parse_unary(cursor);
        return expr;
    }

    fn parse_unary(cursor: &mut TokensCursor) -> Result<Expr, ParserErr> {
        let t = cursor.next().ok_or_else(expected_token_err)?;

        if t.kind == TokenKind::Integer {
            let int_value = chars_to_usize(&t.value)?;

            return Ok(Expr::Const(int_value));
        } else if let Ok(un_op) = UnOp::from(t) {
            return Ok(Expr::Unary(un_op, Box::from(Expr::parse_factor(cursor)?)));
        } else {
            Err(ParserErr(format!("unexpected token {:?}", t)))
        }
    }
}

fn write_indent(f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
    for _ in 0..indent {
        write!(f, " ")?;
    }

    Ok(())
}

impl Expr {
    fn fmt_pretty(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        write_indent(f, indent)?;

        match self {
            Expr::Const(value) => writeln!(f, "Const {}", value),
            Expr::Unary(op, expr) => {
                writeln!(f, "Unary {}", op)?;
                expr.fmt_pretty(f, indent + 2)
            }
            Expr::Binary(op, left, right) => {
                writeln!(f, "Binary {}", op)?;
                left.fmt_pretty(f, indent + 2)?;
                right.fmt_pretty(f, indent + 2)
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_pretty(f, 0)
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expr),
}

fn expected_token_err() -> ParserErr {
    ParserErr(String::from("expected token"))
}

impl Statement {
    fn from(cursor: &mut TokensCursor) -> Result<Statement, ParserErr> {
        cursor.expect(TokenKind::Return)?;

        let expr = Expr::from(cursor)?;
        cursor.expect(TokenKind::Semicolon)?;

        return Ok(Statement::Return(expr));
    }
}

impl Statement {
    fn fmt_pretty(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        write_indent(f, indent)?;

        match self {
            Statement::Return(expr) => {
                writeln!(f, "Return")?;
                expr.fmt_pretty(f, indent + 2)
            }
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_pretty(f, 0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Func {
    pub name: String,
    pub return_type: Type,
    // args
    // return type?
    pub statement: Statement,
}

#[derive(Debug, PartialEq)]
pub enum Type {
    Int,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
        }
    }
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
}

impl Func {
    fn fmt_pretty(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        write_indent(f, indent)?;
        writeln!(f, "Func {}", self.name)?;

        write_indent(f, indent + 2)?;
        writeln!(f, "ReturnType {}", self.return_type)?;

        write_indent(f, indent + 2)?;
        writeln!(f, "Body")?;
        self.statement.fmt_pretty(f, indent + 4)
    }
}

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_pretty(f, 0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Prog {
    pub function: Func,
}

impl fmt::Display for Prog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Prog")?;
        self.function.fmt_pretty(f, 2)
    }
}

#[derive(Debug)]
pub struct ParserErr(pub String);

pub struct TokensCursor<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl fmt::Display for TokensCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;

        for token in &self.tokens[self.position..] {
            if !first {
                write!(f, " ")?;
            }

            write!(f, "{:?}", token.kind)?;
            first = false;
        }

        Ok(())
    }
}

impl<'a> TokensCursor<'a> {
    pub fn new(tokens: &'a [Token]) -> TokensCursor<'a> {
        TokensCursor {
            tokens,
            position: 0,
        }
    }

    // fn consume(&mut self) -> Option<()> {
    //     self.position += 1;
    // }

    // fn take_while<F>(&mut self, mut predicate: F) -> Vec<&'a Token>
    // where
    //     F: FnMut(&Token) -> bool,
    // {
    //     let mut tokens = Vec::new();

    //     while let Some(token) = self.tokens.get(self.position) {
    //         if !predicate(token) {
    //             break;
    //         }

    //         tokens.push(token);
    //         self.position += 1;
    //     }

    //     tokens
    // }

    fn peek_is(&self, kind: TokenKind) -> bool {
        let token = self.tokens.get(self.position);
        if let Some(t) = token {
            return t.kind == kind;
        }
        return false;
    }

    fn peek(&self) -> Option<&Token> {
        return self.tokens.get(self.position);
    }

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
}

pub fn ast(mut cursor: TokensCursor) -> Result<Prog, ParserErr> {
    let prog = Prog {
        function: Func::from(&mut cursor)?,
    };

    // hmm
    cursor.expect(TokenKind::End)?;

    return Ok(prog);
}
