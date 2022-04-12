use bumpalo::Bump;

pub type Instrs<'ast> = Vec<Instr<'ast>, &'ast Bump>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr<'ast> {
    Add,
    Sub,
    Right,
    Left,
    Out,
    In,
    Loop(Instrs<'ast>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError;

pub fn parse<I>(alloc: &Bump, mut src: I) -> Result<Instrs<'_>, ParseError>
where
    I: Iterator<Item = u8>,
{
    let mut instrs = Vec::new_in(alloc);

    loop {
        match src.next() {
            Some(b'+') => instrs.push(Instr::Add),
            Some(b'-') => instrs.push(Instr::Sub),
            Some(b'>') => instrs.push(Instr::Right),
            Some(b'<') => instrs.push(Instr::Left),
            Some(b'.') => instrs.push(Instr::Out),
            Some(b',') => instrs.push(Instr::In),
            Some(b'[') => {
                let loop_instrs = parse_loop(alloc, &mut src, 0)?;
                instrs.push(Instr::Loop(loop_instrs));
            }
            Some(b']') => return Err(ParseError),
            Some(_) => {} // comment
            None => break,
        }
    }

    Ok(instrs)
}

fn parse_loop<'ast, I>(
    alloc: &'ast Bump,
    src: &mut I,
    depth: u16,
) -> Result<Instrs<'ast>, ParseError>
where
    I: Iterator<Item = u8>,
{
    const MAX_DEPTH: u16 = 1000;

    if depth > MAX_DEPTH {
        return Err(ParseError);
    }

    let mut instrs = Vec::new_in(alloc);

    loop {
        match src.next() {
            Some(b'+') => instrs.push(Instr::Add),
            Some(b'-') => instrs.push(Instr::Sub),
            Some(b'>') => instrs.push(Instr::Right),
            Some(b'<') => instrs.push(Instr::Left),
            Some(b'.') => instrs.push(Instr::Out),
            Some(b',') => instrs.push(Instr::In),
            Some(b'[') => {
                let loop_instrs = parse_loop(alloc, src, depth + 1)?;
                instrs.push(Instr::Loop(loop_instrs));
            }
            Some(b']') => break,
            Some(_) => {} // comment
            None => return Err(ParseError),
        }
    }

    Ok(instrs)
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;

    #[test]
    fn simple() {
        let alloc = Bump::new();

        let bf = ">+<++[-].";
        let instrs = super::parse(&alloc, bf.bytes());
        insta::assert_debug_snapshot!(instrs);
    }

    #[test]
    fn nested_loop() {
        let alloc = Bump::new();

        let bf = "+[-[-[-]]+>>>]";
        let instrs = super::parse(&alloc, bf.bytes());
        insta::assert_debug_snapshot!(instrs);
    }
}
