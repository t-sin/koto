pub mod vm;
pub mod compile;

use std::collections::HashMap;

use super::sexp;
use vm::{Reg, Op, VM};
use compile::compile;

pub fn run_test() {
    println!("-- somnia test --");
    let memory = &[0, 0, 0, 0, 0, 0, 0];
    // let code = "(+ 1 (+ 2 (+ 3 (+ 4 5))))".to_string();
    // println!("code: {:?}", code);
    // let prog = compile(sexp::read(code).unwrap());
    let prog = vec!(
        Box::new(Op::LOADC(32, Reg::R1)),
        Box::new(Op::PUSH(Reg::R1)),
        Box::new(Op::LOADC(1, Reg::R1)),
        Box::new(Op::PUSH(Reg::R1)),
        Box::new(Op::LOADC(4, Reg::R1)),
        Box::new(Op::PUSH(Reg::R1)),
        Box::new(Op::POP(Reg::R2)),
        Box::new(Op::POP(Reg::R3)),
        Box::new(Op::POP(Reg::R4)),
    );
    println!("prog: {:?}", prog);
    let mut vm = VM::init(prog, memory);
    println!("register: {:?}", vm.reg);
    println!("memory: {:?}", vm.memory);
    vm.execute();
    println!("register: {:?}", vm.reg);
    println!("memory: {:?}", vm.memory);
}
