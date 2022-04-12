use crate::parse::Instr;
use bumpalo::Bump;

pub type Ir<'ir> = Vec<Stmt<'ir>, &'ir Bump>;

#[derive(Debug)]
pub enum Stmt<'ir> {
    Add(u8),
    Sub(u8),
    Right(usize),
    Left(usize),
    Loop(Ir<'ir>),
    Out,
    In,
    SetNull,
}

pub fn optimize<'ir>(alloc: &'ir Bump, instrs: &[Instr<'_>]) -> Ir<'ir> {
    let mut ir = ast_to_ir(alloc, instrs);
    pass_find_set_null(&mut ir);
    ir
}

fn ast_to_ir<'ir>(alloc: &'ir Bump, ast: &[Instr<'_>]) -> Ir<'ir> {
    let mut ir = Vec::new_in(alloc);

    let mut instr_iter = ast.iter();

    let Some(first) = instr_iter.next() else { return ir; };

    let mut last = first;
    let mut count = 1;

    for next in instr_iter {
        match last {
            Instr::Add | Instr::Sub | Instr::Right | Instr::Left if last == next => {
                count += 1;
                continue;
            }
            _ => {
                let new_last = match last {
                    Instr::Add => Stmt::Add(count),
                    Instr::Sub => Stmt::Sub(count),
                    Instr::Right => Stmt::Right(count.into()),
                    Instr::Left => Stmt::Left(count.into()),
                    Instr::Out => Stmt::Out,
                    Instr::In => Stmt::In,
                    Instr::Loop(body) => Stmt::Loop(ast_to_ir(alloc, body)),
                };
                ir.push(new_last);
                last = next;
                count = 1;
            }
        }
    }

    let new_last = match last {
        Instr::Add => Stmt::Add(count),
        Instr::Sub => Stmt::Sub(count),
        Instr::Right => Stmt::Right(count.into()),
        Instr::Left => Stmt::Left(count.into()),
        Instr::Out => Stmt::Out,
        Instr::In => Stmt::In,
        Instr::Loop(body) => Stmt::Loop(ast_to_ir(alloc, body)),
    };
    ir.push(new_last);

    ir
}

fn pass_find_set_null(ir: &mut Ir<'_>) {
    for stmt in ir {
        if let Stmt::Loop(body) = stmt {
            if let [Stmt::Sub(_)] = body.as_slice() {
                println!("REPLACE");
                *stmt = Stmt::SetNull;
            }
        }
    }
}
