use std::num::Wrapping;

#[derive(Debug)]
pub struct Register {
    // zero register
    pub zr: u32,
    // instruction pointer
    pub ip: u32,
    // stack pointer
    pub sp: u32,
    // general purpose registers
    pub r1: u32,  pub r2: u32,
    pub r3: u32,  pub r4: u32,
    // output values
    pub ol: u32,  pub or: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum Reg {
    ZR, IP, SP,
    R1, R2, R3, R4,
    OL, OR,
}

#[derive(Debug, Clone)]
pub enum Op {
    NOP,
    // loading/storing
    LOADC(u32, Reg),
    LOAD(u32, Reg),
    STORE(Reg, u32),
    OUT(Reg),
    // basic arithmatic
    ADD(Reg, Reg, Reg),
    SUB(Reg, Reg, Reg),
    MUL(Reg, Reg, Reg),
    DIV(Reg, Reg, Reg),
    // bit shift
    SHL(Reg, Reg, Reg),
    SHR(Reg, Reg, Reg),
}

pub struct VM {
    pub reg: Register,
    pub program: Vec<Box<Op>>,
    pub memory: Vec<u32>,
}

impl Reg {
    fn to_num(reg: Reg) -> u8 {
        match reg {
            Reg::ZR => 0,
            Reg::R1 => 1,
            Reg::R2 => 2,
            Reg::R3 => 3,
            Reg::R4 => 4,
            Reg::IP => 32,
            Reg::SP => 33,
            Reg::OL => 34,
            Reg::OR => 35,
        }
    }
    fn to_reg(n: u8) -> Reg {
        match n {
            1 => Reg::R1,
            2 => Reg::R2,
            3 => Reg::R3,
            4 => Reg::R4,
            32 => Reg::IP,
            33 => Reg::SP,
            34 => Reg::OL,
            35 => Reg::OR,
            _ => Reg::ZR,
        }
    }


    fn get(self, vm: &mut VM) -> u32 {
        match self {
            Reg::ZR => vm.reg.zr,
            Reg::R1 => vm.reg.r1,
            Reg::R2 => vm.reg.r2,
            Reg::R3 => vm.reg.r3,
            Reg::R4 => vm.reg.r4,
            Reg::IP => vm.reg.ip,
            Reg::SP => vm.reg.sp,
            Reg::OL => vm.reg.ol,
            Reg::OR => vm.reg.or,
        }
    }

    fn set(self, vm: &mut VM, val: u32) {
        match self {
            Reg::ZR => (),
            Reg::R1 => vm.reg.r1 = val,
            Reg::R2 => vm.reg.r2 = val,
            Reg::R3 => vm.reg.r3 = val,
            Reg::R4 => vm.reg.r4 = val,
            Reg::IP => vm.reg.ip = val,
            Reg::SP => vm.reg.sp = val,
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
        Op::LOAD(pos, reg) => reg.set(vm, vm.memory[pos as usize]),
        Op::LOADC(val, reg) => reg.set(vm, val),
        Op::STORE(reg, pos) => vm.memory[pos as usize] = reg.get(vm),
        Op::OUT(reg) => {
            let v = reg.get(vm);
            Reg::OL.set(vm, v);
            Reg::OR.set(vm, v);
        }
        Op::ADD(op1, op2, tr) => {
            let v1 = op1.get(vm);
            let v2 = op2.get(vm);
            let val = (Wrapping(v1) + Wrapping(v2)).0;
            match tr {
                Reg::R1 => vm.reg.r1 = val,
                Reg::R2 => vm.reg.r2 = val,
                Reg::R3 => vm.reg.r3 = val,
                Reg::R4 => vm.reg.r4 = val,
                r => panic!("{:?} cannot store result of add"),
            };
        },
        Op::SUB(op1, op2, tr) => {
            let v1 = op1.get(vm);
            let v2 = op2.get(vm);
            let val = (Wrapping(v1) - Wrapping(v2)).0;
            match tr {
                Reg::R1 => vm.reg.r1 = val,
                Reg::R2 => vm.reg.r2 = val,
                Reg::R3 => vm.reg.r3 = val,
                Reg::R4 => vm.reg.r4 = val,
                r => panic!("{:?} cannot store result of sub"),
            };
        },
        Op::MUL(op1, op2, tr) => {
            let v1 = op1.get(vm);
            let v2 = op2.get(vm);
            let val = (Wrapping(v1) * Wrapping(v2)).0;
            match tr {
                Reg::R1 => vm.reg.r1 = val,
                Reg::R2 => vm.reg.r2 = val,
                Reg::R3 => vm.reg.r3 = val,
                Reg::R4 => vm.reg.r4 = val,
                r => panic!("{:?} cannot store result of mul"),
            };
        },
        Op::DIV(op1, op2, tr) => {
            let v1 = op1.get(vm);
            let v2 = op2.get(vm);
            let val = (Wrapping(v1) / Wrapping(v2)).0;
            match tr {
                Reg::R1 => vm.reg.r1 = val,
                Reg::R2 => vm.reg.r2 = val,
                Reg::R3 => vm.reg.r3 = val,
                Reg::R4 => vm.reg.r4 = val,
                r => panic!("{:?} cannot store result of div"),
            };
        },
        Op::SHL(op1, op2, tr) => {
            let v1 = op1.get(vm);
            let v2 = op2.get(vm);
            let val = (Wrapping(v1) << v2 as usize).0;
            match tr {
                Reg::R1 => vm.reg.r1 = val,
                Reg::R2 => vm.reg.r2 = val,
                Reg::R3 => vm.reg.r3 = val,
                Reg::R4 => vm.reg.r4 = val,
                r => panic!("{:?} cannot store result of shl"),
            };
        },
        Op::SHR(op1, op2, tr) => {
            let v1 = op1.get(vm);
            let v2 = op2.get(vm);
            let val = (Wrapping(v1) >> v2 as usize).0;
            match tr {
                Reg::R1 => vm.reg.r1 = val,
                Reg::R2 => vm.reg.r2 = val,
                Reg::R3 => vm.reg.r3 = val,
                Reg::R4 => vm.reg.r4 = val,
                r => panic!("{:?} cannot store result of shr"),
            };
        },
    }
    vm.reg.ip += 1;
}

impl VM {
    pub fn init(program: Vec<Box<Op>>, memory: &[u32]) -> VM {
        let vm = VM {
            reg: Register {
                zr: 0, ip: 0, sp: 0,
                r1: 0, r2: 0, r3: 0, r4: 0,
                ol: 0, or: 0
            },
            program: program,
            memory: Vec::from(memory),
        };
        vm
    }

    pub fn execute(&mut self) {
        while (self.reg.ip < self.program.len() as u32) {
            exec_1(self);
        }
    }
}
