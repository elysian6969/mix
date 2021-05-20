use super::lexer::{Lexer, Token};
use std::path::{Path, PathBuf};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    lookahead: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let lookahead = lexer.next();

        Self { lexer, lookahead }
    }

    fn step(&mut self) {
        self.lookahead = self.lexer.next();
    }

    fn current(&self) -> &Option<Token<'a>> {
        &self.lookahead
    }

    /// optionally consume a check
    fn consume_check(&mut self) -> Option<()> {
        match self.current() {
            Some(Token::Check) => Some(self.step()),
            _ => None,
        }
    }

    /// optionally consume an cached
    fn consume_cached(&mut self) -> Option<()> {
        match self.current() {
            Some(Token::Cached) => Some(self.step()),
            _ => None,
        }
    }

    /// optionally consume an ellipsis
    fn consume_ellipsis(&mut self) -> Option<()> {
        match self.current() {
            Some(Token::Elipsis) => Some(self.step()),
            _ => None,
        }
    }

    /// optionally consume a for
    fn consume_for(&mut self) -> Option<()> {
        match self.current() {
            Some(Token::For) => Some(self.step()),
            _ => None,
        }
    }

    /// optionally consume header
    fn consume_header(&mut self) -> Option<PathBuf> {
        match self.current() {
            Some(Token::Header(header)) => {
                let result = Some(header.to_path_buf());

                self.step();

                result
            }
            _ => None,
        }
    }

    /// optionally consume ident
    fn consume_ident(&mut self) -> Option<String> {
        match self.current() {
            Some(Token::Ident(ident)) => {
                let result = Some(ident.to_string());

                self.step();

                result
            }
            _ => None,
        }
    }

    /// optionally consume presence
    fn consume_presence(&mut self) -> Option<()> {
        match self.current() {
            Some(Token::Presence) => Some(self.step()),
            _ => None,
        }
    }

    /// optionally consume a type
    fn consume_type(&mut self) -> Option<()> {
        match self.current() {
            Some(Token::Type) => Some(self.step()),
            _ => None,
        }
    }

    /// optionally consume a status
    fn consume_status(&mut self) -> Option<bool> {
        let result = match self.current() {
            Some(Token::Yes) => Some(true),
            Some(Token::No) => Some(false),
            _ => None,
        };

        if result.is_some() {
            self.step();
        }

        result
    }

    /// optionally consume usability
    fn consume_usability(&mut self) -> Option<()> {
        match self.current() {
            Some(Token::Usability) => Some(self.step()),
            _ => None,
        }
    }

    /// optionally consume the rest of a check pattern
    fn consume_check_rest(&mut self) -> Option<bool> {
        let _ = self.consume_ellipsis();
        let _ = self.consume_cached();

        self.consume_status()
    }

    /// optionally consume type pattern
    fn consume_check_type(&mut self) -> Option<String> {
        self.consume_type()?;
        self.consume_ident()
    }

    /// optionally consume presence or usability
    fn comsume_ignore(&mut self) -> Option<()> {
        self.consume_presence().or(self.consume_usability())
    }

    /// optionally consume check patterns
    fn consume_check_pattern(&mut self) -> Option<Status> {
        self.consume_check()?;
        self.consume_for()?;

        /*let pattern = self
        .consume_header()
        .map(Pattern::Header)
        .or_else(|| self.consume_ident().map(Pattern::Ident))
        .or_else(|| self.consume_check_type().map(Pattern::Type))?;*/

        self.step();
        dbg!(self.current());

        return Some(Status::Check(Check::None));

        /*let ignore = self.comsume_ignore().is_some();

        if ignore {
            return Some(Status::Check(Check::None));
        }

        let status = self.consume_check_rest();
        let check = match pattern {
            Pattern::Header(header) => Check::Header(header, status),
            Pattern::Ident(ident) => Check::Any(ident, status),
            Pattern::Type(ident) => Check::Type(ident, status),
        };

        Some(Status::Check(check))*/
    }

    pub fn parse(&mut self) -> Status {
        match self.consume_check_pattern() {
            Some(status) => status,
            _ => Status::None,
        }
    }
}

#[derive(Debug)]
pub enum Status {
    Check(Check),
    None,
}

#[derive(Debug)]
pub enum Check {
    Any(String, Option<bool>),
    Header(PathBuf, Option<bool>),
    Type(String, Option<bool>),
    None,
}

#[derive(Debug)]
enum Pattern {
    Header(PathBuf),
    Ident(String),
    Type(String),
}
