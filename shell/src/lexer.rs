use core::str::CharIndices;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Token<'a> {
    /// #!
    Shebang(&'a str),

    /// '
    Apostrophe,
    /// `
    Backtick,
    /// "
    Quote,

    /// =
    Equals,
    /// -
    Hyphen,
    /// +
    Plus,

    /// .
    Dot,
    /// ,
    Comma,

    /// |
    Bar,

    /// @
    At,

    /// $
    Dollar,
    /// %
    Percent,

    /// &
    Ampersand,
    /// *
    Asterisk,

    /// ?
    Question,

    /// :
    Colon,
    /// ;
    Semicolon,

    /// !
    Exclamation,
    /// #
    Hash,
    /// /
    Slash,
    /// \
    Backslash,

    /// (
    ParenLeft,
    /// )
    ParenRight,

    /// {
    BraceLeft,
    /// }
    BraceRight,

    /// [
    BracketLeft,
    /// ]
    BracketRight,

    /// <
    ChevronLeft,
    /// >
    ChevronRight,

    /// '#'
    /// anything between
    /// '\n'
    Comment(&'a str),

    /// '_'
    /// 'a'..='z'
    /// 'A'..='Z'
    Alpha(&'a str),

    /// '_'
    /// 'a'..='z'
    /// 'A'..='Z'
    /// '0'..='9'
    Alphanumeric(&'a str),

    /// '0'..='9'
    Integer(&'a str),

    /// '0'..='9'
    /// '.'
    Float(&'a str),

    /// ' '
    /// '\t'
    /// '\r'
    /// '\n'
    Space(&'a str),
}

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    one: Option<(usize, char)>,
    two: Option<(usize, char)>,
    three: Option<(usize, char)>,
}

fn map_char((_position, character): (usize, char)) -> char {
    character
}

fn map_token<'a>((_position, token): (usize, Token<'a>)) -> Token<'a> {
    token
}

impl<'a> Lexer<'a> {
    /// construct a new lexer
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.char_indices();
        let one = chars.next();
        let two = chars.next();
        let three = chars.next();

        Self {
            input,
            chars,
            one,
            two,
            three,
        }
    }

    /// return one lookahead char
    fn one(&self) -> Option<char> {
        self.one.map(map_char)
    }

    /// return two lookahead chars
    fn two(&self) -> Option<(char, char)> {
        self.one()
            .and_then(|one| Some((one, self.two.map(map_char)?)))
    }

    /// return three lookaheaad chars
    fn three(&self) -> Option<(char, char, char)> {
        self.two()
            .and_then(|(one, two)| Some((one, two, self.three.map(map_char)?)))
    }

    /// increment once
    fn step(&mut self) {
        self.one = self.two;
        self.two = self.three;
        self.three = self.chars.next();
    }

    /// increment n times
    fn stepn(&mut self, n: usize) {
        for _ in 0..n {
            self.step();
        }
    }

    fn position(&self) -> usize {
        self.one.map(|(position, _character)| position).unwrap_or(0)
    }

    /// consume alpha
    fn alpha(&mut self, start: usize) -> Option<(usize, Token<'a>)> {
        while matches!(self.one(), Some('_' | 'a'..='z' | 'A'..='Z')) {
            self.step();
        }

        if matches!(self.one(), Some('0'..='9')) {
            self.step();

            return self.alphanumeric(start);
        }

        let end = Self::position(&self);

        if start < end {
            Some((start, Token::Alpha(&self.input[start..end])))
        } else {
            None
        }
    }

    /// consume alphanumber
    fn alphanumeric(&mut self, start: usize) -> Option<(usize, Token<'a>)> {
        while matches!(self.one(), Some('_' | 'a'..='z' | 'A'..='Z' | '0'..='9')) {
            self.step();
        }

        let end = Self::position(&self);

        if start < end {
            Some((start, Token::Alphanumeric(&self.input[start..end])))
        } else {
            None
        }
    }

    /// consume integer
    fn integer(&mut self, start: usize) -> Option<(usize, Token<'a>)> {
        while matches!(self.one(), Some('0'..='9')) {
            self.step();
        }

        if matches!(self.one(), Some('.')) {
            self.step();

            return self.float(start);
        }

        let end = Self::position(&self);

        if start < end {
            Some((start, Token::Integer(&self.input[start..end])))
        } else {
            None
        }
    }

    /// consume float
    fn float(&mut self, start: usize) -> Option<(usize, Token<'a>)> {
        while matches!(self.one(), Some('0'..='9')) {
            self.step();
        }

        let end = Self::position(&self);

        if start < end {
            Some((start, Token::Float(&self.input[start..end])))
        } else {
            None
        }
    }

    /// consume space
    fn space(&mut self, start: usize) -> Option<(usize, Token<'a>)> {
        while matches!(self.one(), Some(' ' | '\t' | '\r' | '\n')) {
            self.step();
        }

        let end = Self::position(&self);

        if start < end {
            Some((start, Token::Space(&self.input[start..end])))
        } else {
            None
        }
    }

    /// consume comment
    fn comment(&mut self, start: usize) -> Option<(usize, Token<'a>)> {
        while !matches!(self.one(), Some('\n')) {
            self.step();
        }

        let end = Self::position(&self);

        if start < end {
            Some((start, Token::Comment(&self.input[start..end])))
        } else {
            None
        }
    }

    /// consume shebang
    fn shebang(&mut self, start: usize) -> Option<(usize, Token<'a>)> {
        while !matches!(self.one(), Some('\n')) {
            self.step();
        }

        let end = Self::position(&self);

        if start < end {
            Some((start, Token::Shebang(&self.input[start..end])))
        } else {
            None
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = match self.two() {
            Some(('#', '!')) => {
                let start = Self::position(self);

                self.stepn(2);

                return self.shebang(start).map(map_token);
            }
            _ => None,
        };

        if token.is_some() {
            self.stepn(2);

            return token;
        }

        let token = match self.one() {
            Some('!') => Some(Token::Exclamation),
            Some('/') => Some(Token::Slash),
            Some('\\') => Some(Token::Backslash),
            Some('(') => Some(Token::ParenLeft),
            Some(')') => Some(Token::ParenRight),
            Some('{') => Some(Token::BraceLeft),
            Some('}') => Some(Token::BraceRight),
            Some('[') => Some(Token::BracketLeft),
            Some(']') => Some(Token::BracketRight),
            Some('<') => Some(Token::ChevronLeft),
            Some('>') => Some(Token::ChevronRight),
            Some('\'') => Some(Token::Apostrophe),
            Some('"') => Some(Token::Quote),
            Some('`') => Some(Token::Backtick),
            Some(':') => Some(Token::Colon),
            Some(';') => Some(Token::Semicolon),
            Some('=') => Some(Token::Equals),
            Some('-') => Some(Token::Hyphen),
            Some('+') => Some(Token::Plus),
            Some('.') => Some(Token::Dot),
            Some(',') => Some(Token::Comma),
            Some('$') => Some(Token::Dollar),
            Some('|') => Some(Token::Bar),
            Some('%') => Some(Token::Percent),
            Some('&') => Some(Token::Ampersand),
            Some('*') => Some(Token::Asterisk),
            Some('?') => Some(Token::Question),
            Some('@') => Some(Token::At),
            Some('#') => {
                let start = Self::position(self);

                self.step();

                return self.comment(start).map(map_token);
            }
            Some('_' | 'a'..='z' | 'A'..='Z') => {
                let start = Self::position(self);

                self.step();

                return self.alpha(start).map(map_token);
            }
            Some('0'..='9') => {
                let start = Self::position(self);

                self.step();

                return self.integer(start).map(map_token);
            }
            Some(' ' | '\t' | '\r' | '\n') => {
                let start = Self::position(self);

                self.step();

                return self.space(start).map(map_token);
            }
            _ => None,
        };

        if token.is_some() {
            self.step();

            return token;
        }

        None
    }
}
