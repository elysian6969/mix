use semver_parser::lexer;
use semver_parser::lexer::{Lexer, Token};
use semver_parser::version::{Identifier, Version};
use std::mem;

type Result<'input, T> = std::result::Result<T, Error<'input>>;

pub fn parse(input: &str) -> Result<'_, semver::Version> {
    Parser::new(input)?.version()
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Error<'input> {
    /// needed more tokens for parsing, but none are available
    UnexpectedEnd,
    /// unexpected token
    UnexpectedToken(Token<'input>),
    /// an error occurred in the lexer
    Lexer(lexer::Error),
    /// more input available
    MoreInput(Vec<Token<'input>>),
    /// encountered empty predicate in a set of predicates
    EmptyPredicate,
    /// encountered an empty range
    EmptyRange,
}

impl<'input> From<lexer::Error> for Error<'input> {
    fn from(value: lexer::Error) -> Self {
        Error::Lexer(value)
    }
}

// TODO: implememt uDebug for Error
// impl<'input> ufmt::uDebug for Error<'input> {
//    fn fmt<W>(&self, fmt: &mut ufmt::Formatter<'_, W>) -> fmt::Result
//    where
//        W: ufmt::uWrite + ?Sized,
//     {
//         use self::Error::*;
//
//        match *self {
//             UnexpectedEnd => fmt.write_str("expected more input"),
//             UnexpectedToken(ref token) => {
//                 ufmt::write!(fmt, "encountered unexpected token: {:?}", token)
//             }
//             Lexer(ref error) => ufmt::write!(fmt, "lexer error: {:?}", error),
//             MoreInput(ref tokens) => {
//                 ufmt::write!(fmt, "expected end of input, but got: {:?}", tokens)
//             }
//             EmptyPredicate => fmt.write_str("encountered empty predicate"),
//             EmptyRange => fmt.write_str("encountered empty range"),
//         }
//     }
// }

impl<'input> ufmt::uDisplay for Error<'input> {
    fn fmt<W>(&self, fmt: &mut ufmt::Formatter<'_, W>) -> std::result::Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        use self::Error::*;

        match *self {
            UnexpectedEnd => fmt.write_str("expected more input"),
            UnexpectedToken(ref token) => {
                ufmt::uwrite!(
                    fmt,
                    "encountered unexpected token: {:?}",
                    format!("{:?}", token)
                )
            }
            Lexer(ref error) => ufmt::uwrite!(fmt, "lexer error: {:?}", format!("{:?}", error)),
            MoreInput(ref tokens) => {
                ufmt::uwrite!(
                    fmt,
                    "expected end of input, but got: {:?}",
                    format!("{:?}", tokens)
                )
            }
            EmptyPredicate => fmt.write_str("encountered empty predicate"),
            EmptyRange => fmt.write_str("encountered empty range"),
        }
    }
}

/// version parser
pub struct Parser<'input> {
    /// token stream
    lexer: Lexer<'input>,
    /// lookahead
    lookahead: Option<Token<'input>>,
}

impl<'input> Parser<'input> {
    /// construct a new parser
    pub fn new(input: &'input str) -> Result<'input, Self> {
        let mut lexer = Lexer::new(input);

        let lookahead = if let Some(lookahead) = lexer.next() {
            Some(lookahead?)
        } else {
            None
        };

        Ok(Self { lexer, lookahead })
    }

    /// pop one token
    fn pop(&mut self) -> Result<'input, Token<'input>> {
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

    /// skip whitespace if present
    fn skip_whitespace(&mut self) -> Result<'input, ()> {
        match self.peek() {
            Some(&Token::Whitespace(_, _)) => self.pop().map(|_| ()),
            _token => Ok(()),
        }
    }

    /// parse a component
    ///
    /// returns none if the component is a wildcard
    pub fn component(&mut self) -> Result<'input, Option<u64>> {
        match self.pop()? {
            Token::Numeric(number) => Ok(Some(number)),
            ref token if token.is_wildcard() => Ok(None),
            token => Err(Error::UnexpectedToken(token)),
        }
    }

    /// parse a numeric
    pub fn numeric(&mut self) -> Result<'input, u64> {
        match self.pop()? {
            Token::Numeric(number) => Ok(number),
            token => Err(Error::UnexpectedToken(token)),
        }
    }

    /// optionally parse a dot, then a component
    pub fn dot_component(&mut self) -> Result<'input, (Option<u64>, bool)> {
        match self.peek() {
            Some(&Token::Dot) => {}
            _ => return Ok((None, false)),
        }

        // pop the peeked dot.
        self.pop()?;
        self.component()
            .map(|component| (component, component.is_none()))
    }

    /// parse a dot, then a numeric
    pub fn dot_numeric(&mut self) -> Result<'input, u64> {
        match self.pop()? {
            Token::Dot => {}
            token => return Err(Error::UnexpectedToken(token)),
        }

        self.numeric()
    }

    /// parse a patch, then a numeric
    pub fn patch_numeric(&mut self) -> Result<'input, u64> {
        match self.pop()? {
            Token::Dot => {}
            Token::AlphaNumeric("p") => {}
            token => return Err(Error::UnexpectedToken(token)),
        }

        self.numeric()
    }

    /// parse a string identifier
    ///
    /// `foo`, or `bar`, or `beta-1`
    pub fn identifier(&mut self) -> Result<'input, Identifier> {
        let identifier = match self.pop()? {
            Token::AlphaNumeric(identifier) => Identifier::AlphaNumeric(identifier.to_string()),
            Token::Numeric(numeric) => Identifier::Numeric(numeric),
            token => return Err(Error::UnexpectedToken(token)),
        };

        if let Some(&Token::Hyphen) = self.peek() {
            // pop the peeked hyphen
            self.pop()?;

            // concat with any following identifiers
            Ok(identifier
                .concat("-")
                .concat(&self.identifier()?.to_string()))
        } else {
            Ok(identifier)
        }
    }

    /// parse all pre-release identifiers, separated by dots
    ///
    /// `abcdef.1234`
    fn pre(&mut self) -> Result<'input, Vec<Identifier>> {
        match self.peek() {
            Some(&Token::Hyphen) => {}
            _token => return Ok(Vec::new()),
        }

        // pop the peeked hyphen
        self.pop()?;
        self.parts()
    }

    /// parse a dot-separated set of identifiers
    fn parts(&mut self) -> Result<'input, Vec<Identifier>> {
        let mut parts = vec![self.identifier()?];

        while let Some(&Token::Dot) = self.peek() {
            self.pop()?;
            parts.push(self.identifier()?);
        }

        Ok(parts)
    }

    /// parse optional build metadata
    ///
    /// `` (empty), or `+abcdef`
    fn plus_build_metadata(&mut self) -> Result<'input, Vec<Identifier>> {
        match self.peek() {
            Some(&Token::Plus) => {}
            _token => return Ok(Vec::new()),
        }

        // pop the plus
        self.pop()?;
        self.parts()
    }

    /// parse a version
    ///
    /// `1.0.0` or `3.0.0-beta.1`
    pub fn version(&mut self) -> Result<'input, semver::Version> {
        self.skip_whitespace()?;

        let major = self.numeric()?;
        let minor = self.dot_numeric()?;
        let patch = self.patch_numeric()?;
        let pre = self.pre()?;
        let build = self.plus_build_metadata()?;

        self.skip_whitespace()?;

        Ok(From::from(Version {
            major,
            minor,
            patch,
            pre,
            build,
        }))
    }

    /// check if we have reached the end of input
    pub fn is_eof(&mut self) -> bool {
        self.lookahead.is_none()
    }

    /// get the rest of the tokens in the parser
    ///
    /// useful for debugging
    pub fn tail(&mut self) -> Result<'input, Vec<Token<'input>>> {
        let mut out = Vec::new();

        if let Some(token) = self.lookahead.take() {
            out.push(token);
        }

        while let Some(token) = self.lexer.next() {
            out.push(token?);
        }

        Ok(out)
    }
}
