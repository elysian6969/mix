// altered from semver-parser

use std::str::CharIndices;
use ufmt::derive::uDebug;

macro_rules! scan_while {
    ($slf:expr, $start:expr, $first:pat $(| $rest:pat)*) => {{
        let mut __end = $start;

        loop {
            if let Some((index, character)) = $slf.peek() {
                match character {
                    $first $(| $rest)* => $slf.step(),
                    _ => {
                        __end = index;
                        break;
                    }
                }

                continue;
            } else {
                __end = $slf.input.len();
                break;
            }
        }

        __end
    }}
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, uDebug, Debug)]
pub enum Kind<'a> {
    /// `/`
    Slash,
    /// `:`
    Colon,
    /// `=`
    Eq,
    /// `>`
    Gt,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>=`
    GtEq,
    /// '^`
    Caret,
    /// '~`
    Tilde,
    /// '*`
    Star,
    /// `.`
    Dot,
    /// `,`
    Comma,
    /// `-`
    Hyphen,
    /// `_`
    Underscore,
    /// `+`
    Plus,
    /// '||'
    Or,
    /// whitespace
    Whitespace,
    /// numeric
    Numeric(u64),
    /// alpha,
    Alpha(&'a str),
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, uDebug, Debug)]
pub struct Token<'a> {
    start: usize,
    end: usize,
    kind: Kind<'a>,
}

impl<'a> Token<'a> {
    crate const fn new(start: usize, end: usize, kind: Kind<'a>) -> Self {
        Self { start, end, kind }
    }

    crate const fn new_one(start: usize, kind: Kind<'a>) -> Self {
        Self::new(start, start + 1, kind)
    }

    crate const fn new_two(start: usize, kind: Kind<'a>) -> Self {
        Self::new(start, start + 2, kind)
    }

    /// return kind
    pub fn kind(&self) -> &Kind<'a> {
        &self.kind
    }

    /// is junk piror to an actual version
    pub fn is_junk(&self) -> bool {
        !matches!(self.kind(), Kind::Numeric(_))
    }

    /// is a seperator
    pub fn is_seperator(&self) -> bool {
        matches!(self.kind(), Kind::Dot | Kind::Hyphen | Kind::Underscore)
    }

    /// is whitespace
    pub fn is_whitespace(&self) -> bool {
        matches!(self.kind(), Kind::Whitespace)
    }

    /// is wildcard
    pub fn is_wildcard(&self) -> bool {
        matches!(
            self.kind(),
            Kind::Star | Kind::Alpha("X") | Kind::Alpha("x")
        )
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, uDebug, Debug)]
pub enum Error {
    /// overflowing numeric
    OverflowingNumeric,
    /// unexpected character
    UnexpectedChar(char),
}

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    // lookahead
    l1: Option<(usize, char)>,
    l2: Option<(usize, char)>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &str) -> Lexer {
        let mut chars = input.char_indices();
        let l1 = chars.next();
        let l2 = chars.next();

        Lexer {
            input,
            chars,
            l1,
            l2,
        }
    }

    fn step(&mut self) {
        self.l1 = self.l2;
        self.l2 = self.chars.next();
    }

    fn step_n(&mut self, n: usize) {
        for _ in 0..n {
            self.step();
        }
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.l1
    }

    fn peek2(&mut self) -> Option<(usize, char, char)> {
        self.l1
            .and_then(|(start, l1)| self.l2.map(|(_, l2)| (start, l1, l2)))
    }

    /// consume alpha
    fn alpha(&mut self, start: usize) -> Result<Token<'a>, Error> {
        let end = scan_while!(self, start, 'A'..='Z' | 'a'..='z');

        Ok(Token::new(start, end, Kind::Alpha(&self.input[start..end])))
    }

    /// consume numeric
    fn numeric(&mut self, start: usize) -> Result<Token<'a>, Error> {
        let end = scan_while!(self, start, '0'..='9');
        let input = &self.input[start..end];
        let mut chars = input.chars();

        match (chars.next(), chars.next()) {
            (Some('0'), None) => Ok(Token::new_one(start, Kind::Numeric(0))),
            (Some('0'), Some(character)) => Err(Error::UnexpectedChar(character)),
            _ => match input.parse::<u64>() {
                Ok(numeric) => Ok(Token::new(start, end, Kind::Numeric(numeric))),
                Err(_) => Err(Error::OverflowingNumeric),
            },
        }
    }

    /// consume whitespace
    fn whitespace(&mut self, start: usize) -> Result<Token<'a>, Error> {
        let end = scan_while!(self, start, ' ' | '\t' | '\n' | '\r');

        Ok(Token::new(start, end, Kind::Whitespace))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // two subsequent char tokens.
        if let Some((start, a, b)) = self.peek2() {
            let kind = match (a, b) {
                ('<', '=') => Some(Kind::LtEq),
                ('>', '=') => Some(Kind::GtEq),
                ('|', '|') => Some(Kind::Or),
                _ => None,
            };

            if let Some(kind) = kind {
                self.step_n(2);

                return Some(Ok(Token::new_two(start, kind)));
            }
        }

        // single char and start of numeric tokens.
        if let Some((start, a)) = self.peek() {
            let kind = match a {
                ' ' | '\t' | '\n' | '\r' => {
                    self.step();

                    return Some(self.whitespace(start));
                }
                '/' => Kind::Slash,
                ':' => Kind::Colon,
                '=' => Kind::Eq,
                '>' => Kind::Gt,
                '<' => Kind::Lt,
                '^' => Kind::Caret,
                '~' => Kind::Tilde,
                '*' => Kind::Star,
                '.' => Kind::Dot,
                ',' => Kind::Comma,
                '-' => Kind::Hyphen,
                '_' => Kind::Underscore,
                '+' => Kind::Plus,
                '0'..='9' => {
                    self.step();

                    return Some(self.numeric(start));
                }
                'a'..='z' | 'A'..='Z' => {
                    self.step();

                    return Some(self.alpha(start));
                }
                c => return Some(Err(Error::UnexpectedChar(c))),
            };

            self.step();

            return Some(Ok(Token::new_one(start, kind)));
        };

        None
    }
}
