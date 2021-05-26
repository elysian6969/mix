use super::lexer;
use super::lexer::{Lexer, Token};
use super::Atom;
use core::mem;
use semver::VersionReq;

pub enum Error<'input> {
    /// an error occurred in the lexer
    Lexer(lexer::Error),
    /// an error occured in the version parser
    Semver(semver::ReqParseError),
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

impl<'input> From<semver::ReqParseError> for Error<'input> {
    fn from(value: semver::ReqParseError) -> Self {
        Error::Semver(value)
    }
}

struct Semver<'semver>(&'semver semver::ReqParseError);

impl<'semver> ufmt::uDebug for Semver<'semver> {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let buffer = format!("{:?}", self.0);

        ufmt::uwrite!(f, "{}", buffer)
    }
}

impl<'input> ufmt::uDebug for Error<'input> {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        match *self {
            Error::Lexer(ref error) => f.debug_tuple("Error::Lexer")?.field(error)?.finish()?,
            Error::Semver(ref error) => f
                .debug_tuple("Error::Semver")?
                .field(&Semver(error))?
                .finish()?,
            Error::UnexpectedEnd => f.debug_tuple("Error::UnexpectedEnd")?.finish()?,
            Error::UnexpectedToken(ref error) => f
                .debug_tuple("Error::UnknownToken")?
                .field(error)?
                .finish()?,
            Error::UnknownScheme => f.debug_tuple("Error::UnknownScheme")?.finish()?,
        }

        Ok(())
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

    /// peek one token
    fn peek(&mut self) -> Option<&Token<'input>> {
        self.lookahead.as_ref()
    }

    /// optionally parse a colon `:`
    fn colon(&mut self) -> Result<bool, Error<'input>> {
        match self.peek() {
            Some(&Token::Colon) => {
                let _ = self.pop();

                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// optionally parse a slash `/`
    fn slash(&mut self) -> Result<bool, Error<'input>> {
        match self.peek() {
            Some(&Token::Slash) => self.pop().map(|_| true),
            _ => Ok(false),
        }
    }

    /// parse a segment
    fn segment(&mut self) -> Result<String, Error<'input>> {
        match self.pop()? {
            Token::Segment(segment) => Ok(segment.into()),
            token => Err(Error::UnexpectedToken(token)),
        }
    }

    /// parse a source
    pub fn parse(&mut self) -> Result<Atom, Error<'_>> {
        let mut group = None;
        let mut package = self.segment()?;
        let mut version = VersionReq::any();

        if self.slash()? {
            group = Some(package);
            package = self.segment()?;
        }

        if self.colon()? {
            version = VersionReq::parse(self.lexer.as_str())?;
        }

        Ok(Atom {
            group,
            package,
            version,
        })
    }
}
