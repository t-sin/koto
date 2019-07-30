
#[derive(Debug)]
pub enum Primitive {
    Byte(u8),
    Arr(Vec<Box<u8>>),
}

#[derive(Debug)]
pub struct Register {
    // instruction pointer
    pub ip: usize,
    // general purpose registers
    pub r1: Primitive,
    pub r2: Primitive,
    pub r3: Primitive,
    pub r4: Primitive,
    // output values
    pub ol: Primitive,
    pub or: Primitive,
}

#[derive(Debug, Clone, Copy)]
pub enum Reg {
    IP,
    R1, R2, R3, R4,
    OL, OR,
}

#[derive(Debug, Clone)]
pub enum Op {
    NOP,
    LOAD(usize, Reg),
    STORE(Reg, usize),
    ADD(Reg, Reg, Reg),
}

pub struct VM {
    pub reg: Register,
    pub program: Vec<Box<Op>>,
    pub memory: Vec<u8>,
}

fn exec_1(vm: &mut VM) {
    let op = &vm.program[vm.reg.ip];
    println!("exec1: {:?}", op);
    match **op {
        Op::NOP => (),
        Op::LOAD(pos, reg) => {
            match reg {
                Reg::R1 => vm.reg.r1 = Primitive::Byte(vm.memory[pos]),
                Reg::R2 => vm.reg.r2 = Primitive::Byte(vm.memory[pos]),
                Reg::R3 => vm.reg.r3 = Primitive::Byte(vm.memory[pos]),
                Reg::R4 => vm.reg.r4 = Primitive::Byte(vm.memory[pos]),
                Reg::IP => (),
                Reg::OL => (),
                Reg::OR => (),
            };
        },
        Op::STORE(_, _) => {
        },
        Op::ADD(_, _, _) => {
        },
    }
    vm.reg.ip += 1;
}

impl VM {
    pub fn init(program: &[Op], memory: Vec<u8>) -> VM {
        let mut boxed_prog = Vec::new();
        for op in program.iter() {
            boxed_prog.push(Box::new(op.clone()));
        }

        let vm = VM {
            reg: Register {
                ip: 0,
                r1: Primitive::Byte(0), r2: Primitive::Byte(0),
                r3: Primitive::Byte(0), r4: Primitive::Byte(0),
                ol: Primitive::Byte(0), or: Primitive::Byte(0),
            },
            program: boxed_prog,
            memory: memory,
        };
        vm
    }

    pub fn execute(&mut self) {
        println!("fn execute()");
        while (self.reg.ip < self.program.len()) {
            exec_1(self);
        }
    }
}
