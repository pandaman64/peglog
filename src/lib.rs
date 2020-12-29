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
        let id = P::ID;
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
    Start { id: u16, position: usize },
    Consume { text: rowan::SmolStr },
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
    memo: HashMap<(u16, usize), ParseResult>,
}

impl Emitter {
    fn start(&mut self, id: u16, position: usize) {
        let start_log_index = self.log.len();
        self.memo
            .insert((id, position), ParseResult::Running { start_log_index });
        self.log.push(Event::Start { id, position });
    }

    fn commit(&mut self, id: u16, position: usize) {
        let memo = self.memo.get_mut(&(id, position)).unwrap();
        let end_log_index = self.log.len();
        *memo = ParseResult::Success {
            start_log_index: memo.start_log_index(),
            end_log_index,
        };
        self.log.push(Event::Commit);
    }

    fn abort(&mut self, id: u16, position: usize) {
        let memo = self.memo.get_mut(&(id, position)).unwrap();
        let end_log_index = self.log.len();
        *memo = ParseResult::Failure {
            start_log_index: memo.start_log_index(),
            end_log_index,
        };
        self.log.push(Event::Abort);
    }

    fn tree_at(&self, start_log_index: usize, end_log_index: usize) -> rowan::GreenNode {
        use rowan::*;

        let mut builder = GreenNodeBuilder::new();
        let mut index = start_log_index;

        while index <= end_log_index {
            match &self.log[index] {
                Event::Start { id, position } => match *self.memo.get(&(*id, *position)).unwrap() {
                    ParseResult::Success { .. } => {
                        builder.start_node(SyntaxKind(*id));
                        index += 1;
                    }
                    ParseResult::Failure { end_log_index, .. } => {
                        index = end_log_index;
                    }
                    ParseResult::Running { .. } => unreachable!(),
                },
                Event::Consume { text } => {
                    // TODO: proper SyntaxKind
                    builder.token(SyntaxKind(u16::MAX), text.clone());
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

    pub fn tree(&self, id: u16, position: usize) -> rowan::GreenNode {
        let result = self.memo.get(&(id, position)).unwrap();
        match *result {
            ParseResult::Success {
                start_log_index,
                end_log_index,
            } => self.tree_at(start_log_index, end_log_index),
            _ => todo!(),
        }
    }
}

pub trait Parser {
    const ID: u16;

    fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool;
}
