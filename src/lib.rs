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
        emitter.log.push(Event::Start {
            id: P::ID,
            position: self.position,
        });
        let result = P::parse(self, emitter);
        if result {
            emitter.log.push(Event::Commit);
        } else {
            emitter.log.push(Event::Abort);
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

#[derive(Debug, Default)]
pub struct Emitter {
    log: Vec<Event>,
}

pub trait Parser {
    const ID: u16;

    fn parse(input: &mut Input<'_>, emitter: &mut Emitter) -> bool;
}
