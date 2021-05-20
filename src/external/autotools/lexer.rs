use std::path::Path;
use std::str::SplitWhitespace;

#[derive(Debug)]
pub enum Token<'a> {
    And,
    Build,
    Cached,
    Check,
    Declared,
    Disable,
    Ellipsis,
    Enable,
    Exists,
    For,
    Generate,
    Host,
    If,
    In,
    Is,
    Link,
    No,
    Not,
    Of,
    Or,
    Presence,
    Size,
    System,
    Target,
    To,
    Type,
    Want,
    Whether,
    Usability,
    Use,
    Yes,
    You,
    Header(&'a Path),
    Ident(&'a str),
}

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    split: SplitWhitespace<'a>,
    lookahead: Option<&'a str>,
    insert_ellipsis: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut split = input.split_whitespace();
        let lookahead = split.next();

        Self {
            input,
            split,
            lookahead,
            insert_ellipsis: false,
        }
    }

    fn step(&mut self) {
        self.lookahead = self.split.next();
    }

    fn current(&mut self) -> Option<&'a str> {
        self.lookahead
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.insert_ellipsis {
            self.insert_ellipsis = false;

            return Some(Token::Ellipsis);
        }

        let ident = self.current()?;
        let trimmed = ident.trim_end_matches("...");

        self.insert_ellipsis = trimmed.len() < ident.len();
        self.step();

        let token = match trimmed {
            header if header.ends_with(".h") => Token::Header(Path::new(trimmed)),
            "usability" => Token::Usability,
            "(cached)" => Token::Cached,
            "checking" | "Checking" => Token::Check,
            "creating" => Token::Generate,
            "declared" => Token::Declared,
            "presence" => Token::Presence,
            "disable" => Token::Disable,
            "whether" => Token::Whether,
            "enable" => Token::Enable,
            "exists" => Token::Exists,
            "target" => Token::Target,
            "system" => Token::System,
            "build" => Token::Build,
            "host" => Token::Host,
            "link" => Token::Link,
            "size" => Token::Size,
            "type" => Token::Type,
            "want" => Token::Want,
            "and" => Token::And,
            "for" => Token::For,
            "not" => Token::Not,
            "use" => Token::Use,
            "yes" | "Yes" | "yes." | "Yes." => Token::Yes,
            "you" => Token::You,
            "if" => Token::If,
            "in" => Token::In,
            "is" => Token::Is,
            "no" | "No" | "no." | "No." => Token::No,
            "of" => Token::Of,
            "or" => Token::Or,
            "to" => Token::To,
            ident => Token::Ident(trimmed),
        };

        token.into()
    }
}
