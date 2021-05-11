use core::str::CharIndices;
use ufmt::derive::uDebug;

/// source tokens
#[derive(Eq, Ord, PartialEq, PartialOrd, uDebug)]
pub enum Token<'input> {
    /// `:`
    Colon,
    /// `/`
    Slash,
    /// Alphanumeric component, like `alpha1` or `79deadbe`
    Segment(&'input str),
}

#[derive(Eq, Ord, PartialEq, PartialOrd, uDebug)]
pub enum Error {
    /// unexpected character
    UnexpectedChar(char),
}

pub struct Lexer<'input> {
    input: &'input str,
    chars: CharIndices<'input>,
    lookahead: Option<(usize, char)>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        let mut chars = input.char_indices();
        let lookahead = chars.next();

        Self {
            input,
            chars,
            lookahead,
        }
    }

    crate fn step(&mut self) {
        self.lookahead = self.chars.next();
    }

    crate fn one(&mut self) -> Option<(usize, char)> {
        self.lookahead
    }

    crate fn segment(&mut self, offset: usize) -> Token<'input> {
        let mut end;

        loop {
            if let Some((offset, character)) = self.one() {
                end = offset;

                match character {
                    '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '-' => self.step(),
                    _ => break,
                }

                continue;
            } else {
                end = self.input.len();
            }

            break;
        }

        Token::Segment(&self.input[offset..end])
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<Token<'input>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((offset, character)) = self.one() {
            let token = match character {
                ':' => Token::Colon,
                '/' => Token::Slash,
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '-' => {
                    self.step();

                    return Some(Ok(self.segment(offset)));
                }
                character => return Some(Err(Error::UnexpectedChar(character))),
            };

            self.step();

            return Some(Ok(token));
        }

        None
    }
}
