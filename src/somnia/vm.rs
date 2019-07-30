
#[derive(Debug, Clone)]
pub enum Cell {
    Byte(u8),
    Arr(Vec<Box<u8>>),
}

#[derive(Debug)]
pub struct Register {
    // instruction pointer
    pub ip: usize,
    // general purpose registers
    pub r1: Cell,  pub r2: Cell,
    pub r3: Cell,  pub r4: Cell,
    // output values
    pub ol: Cell,  pub or: Cell,
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

impl Reg {
    fn get(self, vm: &mut VM) -> Cell {
        match self {
            Reg::R1 => vm.reg.r1.clone(),
            Reg::R2 => vm.reg.r2.clone(),
            Reg::R3 => vm.reg.r3.clone(),
            Reg::R4 => vm.reg.r4.clone(),
            Reg::IP => Cell::Byte(vm.reg.ip as u8),
            Reg::OL => vm.reg.ol.clone(),
            Reg::OR => vm.reg.or.clone(),
        }
    }

    fn set(self, vm: &mut VM, val: Cell) {
        match self {
            Reg::R1 => vm.reg.r1 = val,
            Reg::R2 => vm.reg.r2 = val,
            Reg::R3 => vm.reg.r3 = val,
            Reg::R4 => vm.reg.r4 = val,
            Reg::IP => vm.reg.ip = match val {
                Cell::Byte(b) => b as usize,
                Cell::Arr(_) => panic!(),
            },
            Reg::OL => vm.reg.ol = val,
            Reg::OR => vm.reg.or = val,
        };
    }
}

fn exec_1(vm: &mut VM) {
    let op = &vm.program[vm.reg.ip];
    println!("exec1: {:?}", op);
    match **op {
        Op::NOP => (),
        Op::LOAD(pos, reg) => {
            let val = Cell::Byte(vm.memory[pos]);
            reg.set(vm, val);
        },
        Op::STORE(reg, pos) => {
            let val = reg.get(vm);
            match val {
                Cell::Byte(u) => vm.memory[pos] = u,
                Cell::Arr(_) => panic!("array!"),
            };
        },
        Op::ADD(op1, op2, tr) => {
            let v1 = match op1.get(vm) {
                Cell::Byte(u) => u,
                Cell::Arr(_) => panic!("wrong type op1"),
            };
            let v2 = match op2.get(vm) {
                Cell::Byte(u) => u,
                Cell::Arr(_) => panic!("wrong type op1"),
            };
            match tr {
                Reg::R1 => vm.reg.r1 = Cell::Byte(v1 + v2),
                Reg::R2 => vm.reg.r2 = Cell::Byte(v1 + v2),
                Reg::R3 => vm.reg.r3 = Cell::Byte(v1 + v2),
                Reg::R4 => vm.reg.r4 = Cell::Byte(v1 + v2),
                r => panic!("{:?} cannot store result of add"),
            };
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
                r1: Cell::Byte(0), r2: Cell::Byte(0),
                r3: Cell::Byte(0), r4: Cell::Byte(0),
                ol: Cell::Byte(0), or: Cell::Byte(0),
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
