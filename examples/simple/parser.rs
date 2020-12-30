use peglog::{Emitter, Input, Parser};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyntaxKind {
    Token,
    T,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Language;
impl rowan::Language for Language {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        match raw.0 {
            0 => SyntaxKind::Token,
            1u16 => SyntaxKind::T,
            _ => unreachable!(),
        }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(match kind {
            SyntaxKind::Token => 0,
            SyntaxKind::T => 1u16,
        })
    }
}
impl peglog::Language for Language {
    const TOKEN: Self::Kind = SyntaxKind::Token;
}
pub struct T;
impl Parser for T {
    type Language = Language;
    const KIND: SyntaxKind = SyntaxKind::T;
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
