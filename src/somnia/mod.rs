pub mod vm;

use std::collections::HashMap;

use vm::{Primitive, Reg, Op, VM};

pub fn run_test() {
    let prog = [Op::NOP];
    println!("prog: {:?}", prog);
    let mut vm = VM::init(&prog);
    println!("register: {:?}", vm.reg);
    vm.execute();
    println!("register: {:?}", vm.reg);
}
