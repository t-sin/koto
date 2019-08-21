pub mod vm;
pub mod compile;

use std::collections::HashMap;

use super::sexp;
use vm::{Reg, Op, VM};
use compile::compile;

pub fn run_test() {
    println!("-- somnia test --");
    let memory = &[2, 4, 8, 16];
    let code = "(+ 1 42)".to_string();
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
