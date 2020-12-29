use peglog::{Emitter, Input};

mod parser;
use parser::*;

fn main() {
    let mut emitter = Emitter::default();
    println!("{}", Input::new("bbbbba").parse::<T>(&mut emitter));
    println!("{:?}", emitter);
}
