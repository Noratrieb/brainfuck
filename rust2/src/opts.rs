use crate::parse::Instr;
use bumpalo::Bump;

pub type Ir<'ir> = Vec<IrInstr<'ir>, &'ir Bump>;

pub enum IrInstr<'ir> {
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
    ast_to_ir(alloc, instrs)
}

fn ast_to_ir<'ir>(alloc: &'ir Bump, ast: &[Instr<'_>]) -> Ir<'ir> {
    let mut vec = Vec::new_in(alloc);
    vec.extend(ast.iter().map(|instr| match instr {
        Instr::Add => IrInstr::Add(1),
        Instr::Sub => IrInstr::Sub(1),
        Instr::Right => IrInstr::Right(1),
        Instr::Left => IrInstr::Left(1),
        Instr::Out => IrInstr::Out,
        Instr::In => IrInstr::In,
        Instr::Loop(body) => IrInstr::Loop(ast_to_ir(alloc, body)),
    }));
    vec
}
