pub mod vm;

use std::collections::HashMap;

use vm::{Primitive, Reg, Op, VM};

pub fn run_test() {
    println!("-- somnia test --");
    let memory = vec![2, 4, 8, 16];
    let prog = [Op::NOP, Op::LOAD(3, Reg::R1)];
    println!("prog: {:?}", prog);
    let mut vm = VM::init(&prog, memory);
    println!("register: {:?}", vm.reg);
    println!("memory: {:?}", vm.memory);
    vm.execute();
    println!("register: {:?}", vm.reg);
    println!("memory: {:?}", vm.memory);
}
