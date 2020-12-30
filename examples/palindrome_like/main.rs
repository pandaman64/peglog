use peglog::{Emitter, Input};

mod parser;
use parser::*;

// https://github.com/rust-analyzer/rowan/blob/5cf7c244f55b57777a1cc3190f90b04794985900/examples/math.rs#L113
fn print(indent: usize, element: rowan::SyntaxElement<Language>) {
    let kind: SyntaxKind = element.kind().into();
    print!("{:indent$}", "", indent = indent);
    match element {
        rowan::SyntaxElement::Node(node) => {
            let range = node.text_range();
            println!("- {:?}@{:?}..{:?}", kind, range.start(), range.end());
            for child in node.children_with_tokens() {
                print(indent + 2, child);
            }
        }

        rowan::SyntaxElement::Token(token) => println!("- {:?} {:?}", token.text(), kind),
    }
}

fn main() {
    let mut emitter = Emitter::default();
    // println!("{}", Input::new("abbaabba").parse::<S>(&mut emitter));
    println!("{}", Input::new("aabbaa").parse::<S>(&mut emitter));
    println!("{:?}", emitter);
    println!("{:?}", emitter.green_tree::<S>(0));
    print(0, emitter.syntax_tree::<S>(0).into());
}
