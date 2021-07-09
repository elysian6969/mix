use super::lexer::{Kind, Lexer, Token};
use super::{lexer, Version};
use semver::{BuildMetadata, Prerelease};
use std::mem;
use ufmt::derive::uDebug;

#[derive(Debug)]
pub enum Error<'a> {
    /// needed more tokens for parsing, but none are available
    UnexpectedEnd,
    /// unexpected token
    UnexpectedToken(Token<'a>),
    /// an error occurred in the lexer
    Lexer(lexer::Error),
    /// more input available
    MoreInput(Vec<Token<'a>>),
    /// encountered empty predicate in a set of predicates
    EmptyPredicate,
    /// encountered an empty range
    EmptyRange,

    Semver(semver::Error),
}

impl<'a> From<lexer::Error> for Error<'a> {
    fn from(value: lexer::Error) -> Self {
        Error::Lexer(value)
    }
}

impl<'a> From<semver::Error> for Error<'a> {
    fn from(value: semver::Error) -> Self {
        Error::Semver(value)
    }
}

impl<'a> ufmt::uDebug for Error<'a> {
    fn fmt<W>(&self, fmt: &mut ufmt::Formatter<'_, W>) -> std::result::Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        match self {
            Self::UnexpectedEnd => fmt.write_str("expected more input"),
            Self::UnexpectedToken(token) => {
                ufmt::uwrite!(fmt, "encountered unexpected token: {:?}", token)
            }
            Self::Lexer(error) => ufmt::uwrite!(fmt, "lexer error: {:?}", error),
            Self::MoreInput(tokens) => {
                ufmt::uwrite!(fmt, "expected end of input, but got: {:?}", tokens)
            }
            Self::EmptyPredicate => fmt.write_str("encountered empty predicate"),
            Self::EmptyRange => fmt.write_str("encountered empty range"),
            Self::Semver(error) => fmt.write_str(&format!("{error:?}")),
        }
    }
}

/// version parser
pub struct Parser<'a> {
    /// token stream
    lexer: Lexer<'a>,
    /// lookahead
    lookahead: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    /// construct a new parser
    pub fn new(input: &'a str) -> Result<Self, Error<'a>> {
        let mut lexer = Lexer::new(input);

        let lookahead = match lexer.next() {
            Some(lookahead) => Some(lookahead?),
            None => None,
        };

        Ok(Self { lexer, lookahead })
    }

    /// pop one token
    fn pop(&mut self) -> Result<Token<'a>, Error<'a>> {
        let lookahead = match self.lexer.next() {
            Some(lookahead) => Some(lookahead?),
            None => None,
        };

        mem::replace(&mut self.lookahead, lookahead).ok_or(Error::UnexpectedEnd)
    }

    /// peek one token
    fn peek(&mut self) -> Option<&Token<'a>> {
        self.lookahead.as_ref()
    }

    /// skip junk
    fn skip_junk(&mut self) -> Result<(), Error<'a>> {
        while self.peek().map(|token| token.is_junk()) == Some(true) {
            self.pop()?;
        }

        Ok(())
    }

    /// parse a component
    ///
    /// returns none if the component is a wildcard
    pub fn component(&mut self) -> Result<Option<u64>, Error<'a>> {
        let token = self.pop()?;

        match token.kind() {
            Kind::Numeric(number) => Ok(Some(*number)),
            _ if token.is_wildcard() => Ok(None),
            _ => Err(Error::UnexpectedToken(token)),
        }
    }

    /// optionally parse a dot, then a component
    pub fn dot_component(&mut self) -> Result<(Option<u64>, bool), Error<'a>> {
        match self.peek().map(|token| token.kind()) {
            Some(Kind::Dot) => {}
            _ => return Ok((None, false)),
        }

        // pop the peeked dot.
        self.pop()?;
        self.component()
            .map(|component| (component, component.is_none()))
    }

    /// optionally parse a seperator, then a component
    pub fn seperator_component(&mut self) -> Result<(Option<u64>, bool), Error<'a>> {
        match self.peek().map(|token| token.is_seperator()) {
            Some(true) => {}
            _ => return Ok((None, false)),
        }

        // pop the peeked dot.
        self.pop()?;
        self.component()
            .map(|component| (component, component.is_none()))
    }

    /// optionally parse a patch, then a numeric
    pub fn patch_numeric(&mut self) -> Result<u64, Error<'a>> {
        match self.peek().map(|token| token.kind()) {
            Some(Kind::Dot) => {}
            Some(Kind::Hyphen) => {}
            Some(Kind::Underscore) => {}
            Some(Kind::Alpha("p")) => {}
            _ => return Ok(0),
        }

        self.pop()?;
        self.numeric().or(Ok(0))
    }

    /// parse a numeric
    pub fn numeric(&mut self) -> Result<u64, Error<'a>> {
        let token = self.pop()?;

        match token.kind() {
            Kind::Numeric(number) => Ok(*number),
            _ => Err(Error::UnexpectedToken(token)),
        }
    }

    /// parse a dot, then a numeric
    pub fn dot_numeric(&mut self) -> Result<u64, Error<'a>> {
        let token = self.pop()?;

        match token.kind() {
            Kind::Dot => {}
            _ => return Err(Error::UnexpectedToken(token)),
        }

        self.numeric()
    }

    /// optionally parse a seperator, then a numeric
    pub fn seperator_numeric(&mut self) -> Result<u64, Error<'a>> {
        match self.peek() {
            Some(token) if token.is_seperator() => {}
            _ => return Ok(0),
        }

        self.pop()?;
        self.numeric()
    }

    /// parse an string identifier
    ///
    /// `foo`, or `bar`, or `beta-1`
    pub fn identifier(&mut self) -> Result<String, Error<'a>> {
        let token = self.peek();
        let ident = match token.map(|token| token.kind()) {
            Some(Kind::Alpha(alpha)) => {
                let mut ident = alpha.to_string();

                // pop the alpha
                self.pop()?;

                loop {
                    match self.peek().map(|token| token.kind()) {
                        // FIXME: unreachable on first iteration
                        Some(Kind::Alpha(alpha)) => {
                            ident.push_str(&alpha.to_string());

                            // pop the alpha
                            self.pop()?;
                        }
                        Some(Kind::Numeric(numeric)) => {
                            ident.push_str(&numeric.to_string());

                            // pop the numeric
                            self.pop()?;
                        }
                        _ => break,
                    }
                }

                ident
            }
            Some(Kind::Numeric(numeric)) => numeric.to_string(),
            Some(_) => return Err(Error::UnexpectedToken(*token.unwrap())),
            None => return Err(Error::UnexpectedEnd),
        };

        if let Some(Kind::Hyphen) = self.peek().map(|token| token.kind()) {
            // pop the peeked hyphen
            self.pop()?;

            // concat with any following identifiers
            let mut ident = ident.to_string();

            ident.push('-');
            ident.push_str(&self.identifier()?.to_string());

            Ok(ident)
        } else {
            Ok(ident)
        }
    }

    /// parse all pre-release identifiers, separated by dots
    ///
    /// `abcdef.1234`
    fn pre(&mut self) -> Result<String, Error<'a>> {
        if let Some(Kind::Hyphen) = self.peek().map(|token| token.kind()) {
            // pop the hyphen
            self.pop()?;
        }

        self.parts()
    }

    /// parse a dot-separated set of identifiers
    fn parts(&mut self) -> Result<String, Error<'a>> {
        let mut parts = vec![self.identifier()?];

        while let Some(Kind::Dot) = self.peek().map(|token| token.kind()) {
            // pop the dot
            self.pop()?;

            parts.push(self.identifier()?);
        }

        Ok(parts.into_iter().collect())
    }

    /// optionally parse build metadata
    ///
    /// `+abcdef`
    fn plus_build_metadata(&mut self) -> Result<String, Error<'a>> {
        match self.peek().map(|token| token.kind()) {
            Some(Kind::Plus) => {
                // pop the plus
                self.pop()?;
            }
            _ => return Ok(String::new()),
        }

        self.parts()
    }

    /// parse any version
    pub fn any_version(&mut self) -> Result<Version, Error<'a>> {
        self.skip_junk()?;

        let major = self.numeric()?;
        let minor = self.seperator_numeric()?;
        let patch = self.patch_numeric()?;

        let pre = match self.pre() {
            Ok(pre) => pre,
            Err(_) => String::new(),
        };

        let build = match self.plus_build_metadata() {
            Ok(build) => build,
            Err(_) => String::new(),
        };

        Ok(Version {
            major,
            minor,
            patch,
            pre: Prerelease::new(&pre)?,
            build: BuildMetadata::new(&build)?,
        })
    }

    /// parse perfect version
    pub fn perfect_version(&mut self) -> Result<Version, Error<'a>> {
        let major = self.numeric()?;
        let minor = self.dot_numeric()?;
        let patch = self.dot_numeric()?;

        Ok(Version {
            major,
            minor,
            patch,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        })
    }

    /// check if we have reached the end of input
    pub fn is_eof(&mut self) -> bool {
        self.lookahead.is_none()
    }

    /// get the rest of the tokens in the parser
    pub fn rest(&mut self) -> Result<Vec<Token<'a>>, Error<'a>> {
        let mut tokens = Vec::new();

        if let Some(token) = self.lookahead.take() {
            tokens.push(token);
        }

        while let Some(token) = self.lexer.next() {
            tokens.push(token?);
        }

        Ok(tokens)
    }
}
