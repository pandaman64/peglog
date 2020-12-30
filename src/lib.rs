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

    pub fn parse<P: Parser>(&mut self, emitter: &mut Emitter) -> bool {
        use rowan::Language;

        let id = P::Language::kind_to_raw(P::KIND);
        let position = self.position;
        emitter.start(id, position);
        let result = P::parse(self, emitter);
        if result {
            emitter.commit(id, position);
        } else {
            emitter.abort(id, position);
        }
        result
    }
}

#[derive(Debug)]
pub enum Event {
    Start {
        id: rowan::SyntaxKind,
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
    memo: HashMap<(rowan::SyntaxKind, usize), ParseResult>,
}

impl Emitter {
    fn start(&mut self, id: rowan::SyntaxKind, position: usize) {
        let start_log_index = self.log.len();
        self.memo
            .insert((id, position), ParseResult::Running { start_log_index });
        self.log.push(Event::Start { id, position });
    }

    fn commit(&mut self, id: rowan::SyntaxKind, position: usize) {
        let memo = self.memo.get_mut(&(id, position)).unwrap();
        let end_log_index = self.log.len();
        *memo = ParseResult::Success {
            start_log_index: memo.start_log_index(),
            end_log_index,
        };
        self.log.push(Event::Commit);
    }

    fn abort(&mut self, id: rowan::SyntaxKind, position: usize) {
        let memo = self.memo.get_mut(&(id, position)).unwrap();
        let end_log_index = self.log.len();
        *memo = ParseResult::Failure {
            start_log_index: memo.start_log_index(),
            end_log_index,
        };
        self.log.push(Event::Abort);
    }

    fn tree_at<L: Language>(
        &self,
        start_log_index: usize,
        end_log_index: usize,
    ) -> rowan::GreenNode {
        use rowan::*;

        let mut builder = GreenNodeBuilder::new();
        let mut index = start_log_index;

        while index <= end_log_index {
            match &self.log[index] {
                Event::Start { id, position } => match *self.memo.get(&(*id, *position)).unwrap() {
                    ParseResult::Success { .. } => {
                        builder.start_node(*id);
                        index += 1;
                    }
                    ParseResult::Failure { end_log_index, .. } => {
                        index = end_log_index;
                    }
                    ParseResult::Running { .. } => unreachable!(),
                },
                Event::Consume { text } => {
                    // TODO: proper SyntaxKind
                    builder.token(L::kind_to_raw(L::TOKEN), text.clone());
                    index += 1;
                }
                Event::Commit => {
                    builder.finish_node();
                    index += 1;
                }
                Event::Abort => unreachable!(),
            }
        }

        builder.finish()
    }

    pub fn green_tree<P: Parser>(
        &self,
        position: usize,
    ) -> rowan::GreenNode {
        use rowan::Language;

        let result = self
            .memo
            .get(&(P::Language::kind_to_raw(P::KIND), position))
            .unwrap();
        match *result {
            ParseResult::Success {
                start_log_index,
                end_log_index,
            } => self.tree_at::<P::Language>(start_log_index, end_log_index),
            _ => todo!(),
        }
    }

    pub fn syntax_tree<P: Parser>(
        &self,
        position: usize,
    ) -> rowan::SyntaxNode<P::Language> {
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
