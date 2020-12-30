use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Input<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Input<'a> {
    pub fn new(input: &'a str) -> Self {
        Input { input, position: 0 }
    }

    pub fn consume(&mut self, pattern: &str, emitter: &mut Emitter) -> bool {
        if self.input[self.position..].starts_with(pattern) {
            self.position += pattern.len();
            emitter.log.push(Event::Consume {
                text: rowan::SmolStr::new(pattern),
            });
            true
        } else {
            false
        }
    }

    pub fn clause<F>(
        &mut self,
        emitter: &mut Emitter,
        kind: rowan::SyntaxKind,
        clause_id: u16,
        parser: F,
    ) -> bool
    where
        F: FnOnce(&mut Input<'_>, &mut Emitter) -> bool,
    {
        let backtrack = *self;

        let position = self.position;
        emitter.start(kind, clause_id, position);
        let result = parser(self, emitter);
        if result {
            emitter.commit(kind, clause_id, position);
        } else {
            emitter.abort(kind, clause_id, position);
            *self = backtrack;
        }
        result
    }

    pub fn parse<P: Parser>(&mut self, emitter: &mut Emitter) -> bool {
        use rowan::Language;

        let kind = P::Language::kind_to_raw(P::KIND);
        let clause_id = 0; // special clause id
        let position = self.position;

        emitter.start(kind, clause_id, position);
        let result = P::parse(self, emitter);
        if result {
            emitter.commit(kind, clause_id, position);
        } else {
            emitter.abort(kind, clause_id, position);
        }
        result
    }
}

#[derive(Debug)]
pub enum Event {
    Start {
        kind: rowan::SyntaxKind,
        clause_id: u16,
        position: usize,
    },
    Consume {
        text: rowan::SmolStr,
    },
    Commit,
    Abort,
}

#[derive(Debug, Clone, Copy)]
pub enum ParseResult {
    Running {
        start_log_index: usize,
    },
    Success {
        start_log_index: usize,
        end_log_index: usize,
    },
    Failure {
        start_log_index: usize,
        end_log_index: usize,
    },
}

impl ParseResult {
    fn start_log_index(&self) -> usize {
        match self {
            ParseResult::Running { start_log_index }
            | ParseResult::Success {
                start_log_index, ..
            }
            | ParseResult::Failure {
                start_log_index, ..
            } => *start_log_index,
        }
    }
}

#[derive(Debug, Default)]
pub struct Emitter {
    log: Vec<Event>,
    memo: HashMap<(rowan::SyntaxKind, u16, usize), ParseResult>,
}

impl Emitter {
    fn start(&mut self, kind: rowan::SyntaxKind, clause_id: u16, position: usize) {
        let start_log_index = self.log.len();
        self.memo.insert(
            (kind, clause_id, position),
            ParseResult::Running { start_log_index },
        );
        self.log.push(Event::Start {
            kind,
            clause_id,
            position,
        });
    }

    fn commit(&mut self, kind: rowan::SyntaxKind, clause_id: u16, position: usize) {
        let memo = self.memo.get_mut(&(kind, clause_id, position)).unwrap();
        let end_log_index = self.log.len();
        *memo = ParseResult::Success {
            start_log_index: memo.start_log_index(),
            end_log_index,
        };
        self.log.push(Event::Commit);
    }

    fn abort(&mut self, kind: rowan::SyntaxKind, clause_id: u16, position: usize) {
        let memo = self.memo.get_mut(&(kind, clause_id, position)).unwrap();
        let end_log_index = self.log.len();
        *memo = ParseResult::Failure {
            start_log_index: memo.start_log_index(),
            end_log_index,
        };
        self.log.push(Event::Abort);
    }

    fn tree_between(
        &self,
        builder: &mut rowan::GreenNodeBuilder,
        start_log_index: usize,
        end_log_index: usize,
        token_kind: rowan::SyntaxKind,
    ) {
        if start_log_index >= end_log_index {
            return;
        }

        match &self.log[start_log_index] {
            Event::Start {
                kind,
                clause_id,
                position,
            } => match *self.memo.get(&(*kind, *clause_id, *position)).unwrap() {
                ParseResult::Success {
                    end_log_index: child_end,
                    ..
                } => {
                    if *clause_id == 0 {
                        builder.start_node(*kind);
                    }
                    self.tree_between(builder, start_log_index + 1, child_end, token_kind);
                    if *clause_id == 0 {
                        builder.finish_node();
                    }
                    self.tree_between(builder, child_end + 1, end_log_index, token_kind)
                }
                ParseResult::Failure {
                    end_log_index: child_end,
                    ..
                } => self.tree_between(builder, child_end + 1, end_log_index, token_kind),
                ParseResult::Running { .. } => unreachable!(),
            },
            Event::Consume { text } => {
                builder.token(token_kind, text.clone());
                self.tree_between(builder, start_log_index + 1, end_log_index, token_kind)
            }
            _ => unreachable!(),
        }
    }

    pub fn green_tree<P: Parser>(&self, position: usize) -> rowan::GreenNode {
        use rowan::Language;

        let result = self
            .memo
            .get(&(P::Language::kind_to_raw(P::KIND), 0, position))
            .unwrap();
        match *result {
            ParseResult::Success {
                start_log_index,
                end_log_index,
            } => {
                let mut builder = rowan::GreenNodeBuilder::new();
                self.tree_between(
                    &mut builder,
                    start_log_index,
                    end_log_index,
                    P::Language::kind_to_raw(P::Language::TOKEN),
                );
                builder.finish()
            }
            _ => todo!(),
        }
    }

    pub fn syntax_tree<P: Parser>(&self, position: usize) -> rowan::SyntaxNode<P::Language> {
        rowan::SyntaxNode::new_root(self.green_tree::<P>(position))
    }
}

pub trait Language: rowan::Language {
    const TOKEN: Self::Kind;
}

pub trait Parser {
    type Language: Language;
    const KIND: <Self::Language as rowan::Language>::Kind;

    fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool;
}
