use peglog::{Emitter, Input, Parser};
pub struct T;
impl Parser for T {
    const ID: u16 = 0u16;
    fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool {
        false
            || ({
                let backtrack = *input;
                let result = true && input.consume("a", emitter);
                if !result {
                    *input = backtrack;
                }
                result
            })
            || ({
                let backtrack = *input;
                let result = true && input.consume("b", emitter) && input.parse::<T>(emitter);
                if !result {
                    *input = backtrack;
                }
                result
            })
    }
}
