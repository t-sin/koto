#[derive(Debug)]
pub struct Register {
    // instruction pointer
    pub ip: u32,
    // general purpose registers
    pub r1: u32,  pub r2: u32,
    pub r3: u32,  pub r4: u32,
    // output values
    pub ol: u32,  pub or: u32,
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
    LOAD(u32, Reg),
    STORE(Reg, u32),
    ADD(Reg, Reg, Reg),
}

pub struct VM {
    pub reg: Register,
    pub program: Vec<Box<Op>>,
    pub memory: Vec<u32>,
}

impl Reg {
    fn get(self, vm: &mut VM) -> u32 {
        match self {
            Reg::R1 => vm.reg.r1,
            Reg::R2 => vm.reg.r2,
            Reg::R3 => vm.reg.r3,
            Reg::R4 => vm.reg.r4,
            Reg::IP => vm.reg.ip,
            Reg::OL => vm.reg.ol,
            Reg::OR => vm.reg.or,
        }
    }

    fn set(self, vm: &mut VM, val: u32) {
        match self {
            Reg::R1 => vm.reg.r1 = val,
            Reg::R2 => vm.reg.r2 = val,
            Reg::R3 => vm.reg.r3 = val,
            Reg::R4 => vm.reg.r4 = val,
            Reg::IP => vm.reg.ip = val,
            Reg::OL => vm.reg.ol = val,
            Reg::OR => vm.reg.or = val,
        };
    }
}

fn exec_1(vm: &mut VM) {
    let op = &vm.program[vm.reg.ip as usize];
    println!("exec1: {:?}", op);
    match **op {
        Op::NOP => (),
        Op::LOAD(pos, reg) => {
            reg.set(vm, vm.memory[pos as usize]);
        },
        Op::STORE(reg, pos) => {
            vm.memory[pos as usize] = reg.get(vm);
        },
        Op::ADD(op1, op2, tr) => {
            let v1 = op1.get(vm);
            let v2 = op2.get(vm);
            let val = v1 + v2;
            match tr {
                Reg::R1 => vm.reg.r1 = val,
                Reg::R2 => vm.reg.r2 = val,
                Reg::R3 => vm.reg.r3 = val,
                Reg::R4 => vm.reg.r4 = val,
                r => panic!("{:?} cannot store result of add"),
            };
        },
    }
    vm.reg.ip += 1;
}

impl VM {
    pub fn init(program: &[Op], memory: Vec<u32>) -> VM {
        let mut boxed_prog = Vec::new();
        for op in program.iter() {
            boxed_prog.push(Box::new(op.clone()));
        }

        let vm = VM {
            reg: Register {
                ip: 0,
                r1: 0, r2: 0, r3: 0, r4: 0,
                ol: 0, or: 0
            },
            program: boxed_prog,
            memory: memory,
        };
        vm
    }

    pub fn execute(&mut self) {
        while (self.reg.ip < self.program.len() as u32) {
            exec_1(self);
        }
    }
}
