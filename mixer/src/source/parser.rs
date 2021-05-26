use super::lexer;
use super::lexer::{Lexer, Token};
use super::Source;
use core::mem;
use ufmt::derive::uDebug;

#[derive(Eq, Ord, PartialEq, PartialOrd, uDebug)]
pub enum Error<'input> {
    /// an error occurred in the lexer
    Lexer(lexer::Error),
    /// needed more tokens for parsing, but none are available
    UnexpectedEnd,
    /// unexpected token
    UnexpectedToken(Token<'input>),
    /// unknown scheme
    UnknownScheme,
}

impl<'input> From<lexer::Error> for Error<'input> {
    fn from(value: lexer::Error) -> Self {
        Error::Lexer(value)
    }
}

/// source parser
pub struct Parser<'input> {
    /// token stream
    lexer: Lexer<'input>,
    /// lookahead
    lookahead: Option<Token<'input>>,
}

impl<'input> Parser<'input> {
    /// construct a new parser
    pub fn new(input: &'input str) -> Result<Self, Error<'input>> {
        let mut lexer = Lexer::new(input);

        let lookahead = if let Some(lookahead) = lexer.next() {
            Some(lookahead?)
        } else {
            None
        };

        Ok(Self { lexer, lookahead })
    }

    /// pop one token
    fn pop(&mut self) -> Result<Token<'input>, Error<'input>> {
        let lookahead = if let Some(lookahead) = self.lexer.next() {
            Some(lookahead?)
        } else {
            None
        };

        mem::replace(&mut self.lookahead, lookahead).ok_or(Error::UnexpectedEnd)
    }

    /// parse a colon `:`
    fn colon(&mut self) -> Result<(), Error<'input>> {
        match self.pop()? {
            Token::Colon => Ok(()),
            token => Err(Error::UnexpectedToken(token)),
        }
    }

    /// parse a slash `/`
    fn slash(&mut self) -> Result<(), Error<'input>> {
        match self.pop()? {
            Token::Slash => Ok(()),
            token => Err(Error::UnexpectedToken(token)),
        }
    }

    /// parse a scheme
    fn scheme(&mut self) -> Result<String, Error<'input>> {
        let result = match self.pop()? {
            Token::Segment(scheme) => Ok(scheme.into()),
            token => return Err(Error::UnexpectedToken(token)),
        };

        self.colon()?;

        result
    }

    /// parse a segment
    fn segment(&mut self) -> Result<String, Error<'input>> {
        match self.pop()? {
            Token::Segment(segment) => Ok(segment.into()),
            token => Err(Error::UnexpectedToken(token)),
        }
    }

    /// parse a segment and a slash
    fn segment_slash(&mut self) -> Result<String, Error<'input>> {
        let segment = self.segment()?;

        self.slash()?;

        Ok(segment)
    }

    /// parse a `user/repo`
    fn user_repo(&mut self) -> Result<(String, String), Error<'input>> {
        let user = self.segment_slash()?;
        let repo = self.segment()?;

        Ok((user, repo))
    }

    /// helper function to clean up code i guess?
    fn user_repo_with<F, O>(&mut self, callback: F) -> Result<O, Error<'input>>
    where
        F: FnOnce(String, String) -> O,
    {
        let (user, repo) = self.user_repo()?;

        Ok(callback(user, repo))
    }

    /// parse a source
    pub fn parse(&'input mut self) -> Result<Source, Error<'input>> {
        let scheme = self.scheme()?;

        match scheme.as_str() {
            "github" => self.user_repo_with(|user, repo| Source::Github { user, repo }),
            "gitlab" => self.user_repo_with(|user, repo| Source::Gitlab { user, repo }),
            "kernel" => self.user_repo_with(|user, repo| Source::Kernel { user, repo }),
            "savannah" => {
                let repo = self.segment()?;

                Ok(Source::Savannah { repo })
            }
            "sourceware" => {
                let repo = self.segment()?;

                Ok(Source::Sourceware { repo })
            }
            _ => Err(Error::UnknownScheme),
        }
    }
}
