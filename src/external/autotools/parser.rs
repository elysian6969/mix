use super::lexer::{Lexer, Token};
use std::path::PathBuf;

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

    fn consume_if(&mut self, token: Token<'_>) -> Option<()> {
        if self.current() == &Some(token) {
            Some(())
        } else {
            None
        }
    }

    /// optionally consume a check
    fn consume_check(&mut self) -> Option<()> {
        self.consume_if(Token::Check)
    }

    /// optionally consume an cached
    fn consume_cached(&mut self) -> Option<()> {
        self.consume_if(Token::Cached)
    }

    /// optionally consume an ellipsis
    fn consume_ellipsis(&mut self) -> Option<()> {
        self.consume_if(Token::Ellipsis)
    }

    /// optionally consume a for
    fn consume_for(&mut self) -> Option<()> {
        self.consume_if(Token::For)
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
        self.consume_if(Token::Presence)
    }

    /// optionally consume a type
    fn consume_type(&mut self) -> Option<()> {
        self.consume_if(Token::Type)
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
        self.consume_if(Token::Usability)
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
        self.consume_presence().or_else(|| self.consume_usability())
    }

    /// optionally consume check patterns
    fn consume_check_pattern(&mut self) -> Option<Status> {
        self.consume_check()?;
        self.consume_for()?;

        let pattern = self
            .consume_header()
            .map(Pattern::Header)
            .or_else(|| self.consume_ident().map(Pattern::Ident))
            .or_else(|| self.consume_check_type().map(Pattern::Type))?;

        let ignore = self.comsume_ignore().is_some();

        if ignore {
            return Some(Status::Skip);
        }

        let status = self.consume_check_rest();
        let check = match pattern {
            Pattern::Header(header) => Check::Header(header, status),
            Pattern::Ident(ident) => Check::Any(ident, status),
            Pattern::Type(ident) => Check::Type(ident, status),
        };

        Some(Status::Check(check))
    }

    /// consume builder
    fn consume_builder(&mut self) -> Option<()> {
        match self.consume_ident()?.as_str() {
            "builder" | "Builder" => Some(()),
            _ => None,
        }
    }

    /// consume builder pattern
    fn consume_builder_pattern(&mut self) -> Option<Status> {
        self.consume_builder()?;

        Some(Status::Skip)
    }

    pub fn parse(&mut self) -> Status {
        let status = self
            .consume_check_pattern()
            .or_else(|| self.consume_builder_pattern());

        match status {
            Some(status) => status,
            _ => Status::None,
        }
    }
}

#[derive(Debug)]
pub enum Status {
    Check(Check),
    Skip,
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
