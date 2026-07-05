use crate::parse::{BinOp, Expr, Prog, Statement, UnOp};

#[derive(Debug)]
pub struct GenerateError(pub String);

impl Expr {
    fn to_code(&self) -> Result<String, GenerateError> {
        match self {
            Expr::Const(exp) => return Ok(format!("\tmov w0, #{}\n", exp)),
            Expr::Unary(un_op, expr) => {
                // LOOK WE ARE REVERSING ORDER HERE
                return Ok(format!("{}{}", expr.to_code()?, handle_un_op(un_op)?,));
            }
            Expr::Binary(bin_op, left_exp, right_exp) => {
                // calculate left_exp and store in register
                let mut code = String::new();

                code.push_str(left_exp.to_code()?.as_str());
                code.push_str("\tstr w0, [sp, #-16]!\n");

                code.push_str(right_exp.to_code()?.as_str());
                // left in w1
                code.push_str("\tldr w1, [sp], #16\n");

                match bin_op {
                    BinOp::Add => code.push_str("\tadd w0, w1, w0\n"),
                    BinOp::Sub => code.push_str("\tsub w0, w1, w0\n"),
                    BinOp::Mul => code.push_str("\tmul w0, w1, w0\n"),
                    BinOp::Div => code.push_str("\tsdiv w0, w1, w0\n"),
                    BinOp::Mod => {
                        code.push_str("\tsdiv w2, w1, w0\n");
                        code.push_str("\tmsub w0, w2, w0, w1\n");
                    }
                }

                Ok(code)
            }
        }
    }
}

fn handle_un_op(un_op: &UnOp) -> Result<String, GenerateError> {
    match un_op {
        UnOp::Neg => Ok(format!("\tneg w0, w0\n")),
        UnOp::BitwiseComplement => Ok(format!("\tmvn w0, w0\n")),
        UnOp::LogicalNeg => Ok(format!("\tcmp w0, #0 \n\tcset w0, eq\n")),
    }
}

impl Statement {
    fn to_code(&self) -> Result<String, GenerateError> {
        match self {
            Statement::Return(exp) => {
                let exp_code = exp.to_code()?;

                return Ok(format!("{exp_code}\n\tret\n"));
            }
        }
    }
}

pub fn generate_code(prog: Prog) -> Result<String, GenerateError> {
    let mut func_name = String::new();
    func_name.push_str("_");
    func_name.push_str(&prog.function.name);

    let preamble = format!(".global {func_name}");
    let func_name = format!("_{}", prog.function.name);
    let stmt = prog.function.statement.to_code()?;
    let func_code = format!("{func_name}:\n{stmt}");

    return Ok(format!(
        "{preamble}
{func_code}"
    ));
}
