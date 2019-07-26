#[derive(Debug)]
pub enum Primitive {
    Float(f64),
    Int(i32),
    UInt(u32),
    IByte(i8),
    UByte(u8),
    Str(String),
}

#[derive(Debug)]
pub struct Reg {
    // instruction pointer
    pub ip: u32,
    // output values
    pub ol: Primitive,
    pub or: Primitive,
}

#[derive(Debug, Clone)]
pub enum Op {
    NOP,
}

pub struct VM {
    pub reg: Reg,
    pub program: Vec<Box<Op>>,
    pub memory: Vec<Box<u32>>,
}

impl VM {
    pub fn init(program: &[Op]) -> VM {
        let mut boxed_prog = Vec::new();
        for op in program.iter() {
            boxed_prog.push(Box::new(op.clone()));
        }

        let vm = VM {
            reg: Reg {
                ip: 0,
                ol: Primitive::Float(0.0),
                or: Primitive::Float(0.0),
            },
            program: boxed_prog,
            memory: vec![Box::new(2)],
        };
        vm
    }

    pub fn execute(&mut self) {
        println!("fn execute()");
    }
}

