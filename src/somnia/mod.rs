pub mod vm;
pub mod compile;

use std::collections::HashMap;

use super::sexp;
use vm::{Reg, Op, VM};
use compile::compile;

pub fn run_test() {
    println!("-- somnia test --");
    let memory = &[0, 0, 0, 0, 0, 0, 0];
    let code = "(+ (+ 1 (+ -1 (+ 0 1))) (+ 2 (+ 3 (+ 4 5))))".to_string();
    println!("code: {:?}", code);
    let prog = compile(sexp::read(code).unwrap());
    println!("prog: {:?}", prog);
    let mut vm = VM::init(prog, memory);
    println!("register: {:?}", vm.reg);
    println!("memory: {:?}", vm.memory);
    vm.execute();
    println!("register: {:?}", vm.reg);
    println!("memory: {:?}", vm.memory);
}
