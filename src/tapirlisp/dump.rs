use std::fmt;
use std::marker::Send;
use std::sync::{Arc, Mutex};

use super::super::units::unit::{Unit, Osc, Eg, AUnit, Node, UnitGraph};
use super::super::units::core::{Pan, Clip, Offset, Gain, Add, Multiply};
use super::super::units::oscillator::{Rand, Sine, Tri, Saw, Pulse, Phase, WaveTable};
use super::super::units::sequencer::{AdsrEg, Seq};

//// こまっていること
// 
// - Unitトレイトを実装した各構造体をTapirLispのS式にシリアライズしたい
//   （super::super::units::unit等に書きたくない）
// - シリアライズ処理はここに書きたい (tlispのS式として出力するのでtlisp内に置きたい)
// - しかし、Node列挙体のバリアントにはUnit等のトレイトを指定しているので、
//   具体的な型に結び付ける方法がわからない
//
// どうしたらよいだろうこれ……

impl fmt::Display for Pan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(pan {} {})", self.v.lock().unwrap(), self.src.lock().unwrap())
    }
}

impl fmt::Display for Unit + Send {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hoge!")
    }
}

impl fmt::Display for Osc + Send {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hoge!")
    }
}

impl fmt::Display for Eg + Send {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hoge!")
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Val(v) => v.fmt(f),
            Node::Sig(u) => (*u.lock().unwrap()).fmt(f),
            Node::Osc(u) => (*u.lock().unwrap()).fmt(f),
            Node::Eg(u) => (*u.lock().unwrap()).fmt(f),
        }
    }
}

impl fmt::Display for UnitGraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.node.fmt(f)
    }
}

pub fn dump(ug: AUnit) {
    println!("unit graph = {}", *ug.lock().unwrap());
}
