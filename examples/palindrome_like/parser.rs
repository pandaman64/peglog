use peglog::{Emitter, Input, Parser};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyntaxKind {
    Token,
    S,
    Sa,
    Sb,
    Empty,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Language;
impl rowan::Language for Language {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        match raw.0 {
            0 => SyntaxKind::Token,
            1u16 => SyntaxKind::S,
            2u16 => SyntaxKind::Sa,
            3u16 => SyntaxKind::Sb,
            4u16 => SyntaxKind::Empty,
            _ => unreachable!(),
        }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(match kind {
            SyntaxKind::Token => 0,
            SyntaxKind::S => 1u16,
            SyntaxKind::Sa => 2u16,
            SyntaxKind::Sb => 3u16,
            SyntaxKind::Empty => 4u16,
        })
    }
}
impl peglog::Language for Language {
    const TOKEN: Self::Kind = SyntaxKind::Token;
}
pub struct S;
impl Parser for S {
    type Language = Language;
    const KIND: SyntaxKind = SyntaxKind::S;
    fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool {
        false
            || input.clause(
                emitter,
                <Language as rowan::Language>::kind_to_raw(SyntaxKind::S),
                1u16,
                |input, emitter| true && input.parse::<Sa>(emitter),
            )
            || input.clause(
                emitter,
                <Language as rowan::Language>::kind_to_raw(SyntaxKind::S),
                2u16,
                |input, emitter| true && input.parse::<Sb>(emitter),
            )
            || input.clause(
                emitter,
                <Language as rowan::Language>::kind_to_raw(SyntaxKind::S),
                3u16,
                |input, emitter| true && input.parse::<Empty>(emitter),
            )
    }
}
pub struct Sa;
impl Parser for Sa {
    type Language = Language;
    const KIND: SyntaxKind = SyntaxKind::Sa;
    fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool {
        false
            || input.clause(
                emitter,
                <Language as rowan::Language>::kind_to_raw(SyntaxKind::Sa),
                1u16,
                |input, emitter| {
                    true && input.consume("a", emitter)
                        && input.parse::<S>(emitter)
                        && input.consume("a", emitter)
                },
            )
    }
}
pub struct Sb;
impl Parser for Sb {
    type Language = Language;
    const KIND: SyntaxKind = SyntaxKind::Sb;
    fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool {
        false
            || input.clause(
                emitter,
                <Language as rowan::Language>::kind_to_raw(SyntaxKind::Sb),
                1u16,
                |input, emitter| {
                    true && input.consume("b", emitter)
                        && input.parse::<S>(emitter)
                        && input.consume("b", emitter)
                },
            )
    }
}
pub struct Empty;
impl Parser for Empty {
    type Language = Language;
    const KIND: SyntaxKind = SyntaxKind::Empty;
    fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool {
        false
            || input.clause(
                emitter,
                <Language as rowan::Language>::kind_to_raw(SyntaxKind::Empty),
                1u16,
                |input, emitter| true && input.consume("", emitter),
            )
    }
}
