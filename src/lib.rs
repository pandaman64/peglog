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
    Running,
    Success,
    Failure,
}

#[derive(Debug, Default)]
pub struct Emitter {
    log: Vec<Event>,
    memo: HashMap<(u16, usize), ParseResult>,
}

impl Emitter {
    fn start(&mut self, id: u16, position: usize) {
        self.memo.insert((id, position), ParseResult::Running);
        self.log.push(Event::Start { id, position });
    }

    fn commit(&mut self, id: u16, position: usize) {
        *self.memo.get_mut(&(id, position)).unwrap() = ParseResult::Success;
        self.log.push(Event::Commit);
    }

    fn abort(&mut self, id: u16, position: usize) {
        *self.memo.get_mut(&(id, position)).unwrap() = ParseResult::Failure;
        self.log.push(Event::Abort);
    }
}

pub trait Parser {
    const ID: u16;

    fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool;
}
