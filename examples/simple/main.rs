use peglog::{Emitter, Input, Parser};

mod parser;
use parser::*;

fn main() {
    let mut emitter = Emitter::default();
    println!("{}", Input::new("bbbbba").parse::<T>(&mut emitter));
    println!("{:?}", emitter);
    println!("{:?}", emitter.tree(T::ID, 0));
}
