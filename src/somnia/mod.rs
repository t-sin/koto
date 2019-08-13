pub mod vm;

use std::collections::HashMap;

use vm::{Reg, Op, VM};

pub fn run_test() {
    println!("-- somnia test --");
    let memory = &[2, 4, 8, 16];
    let prog = [
        Op::NOP, Op::LOAD(0, Reg::R1), Op::LOAD(1, Reg::R2), Op::STORE(Reg::R1, 3),
        Op::SHL(Reg::R1, Reg::R2, Reg::R3), Op::OUT(Reg::R3)];
    println!("prog: {:?}", prog);
    let mut vm = VM::init(&prog, memory);
    println!("register: {:?}", vm.reg);
    println!("memory: {:?}", vm.memory);
    vm.execute();
    println!("register: {:?}", vm.reg);
    println!("memory: {:?}", vm.memory);
}
