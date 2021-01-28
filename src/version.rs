use semver::{AlphaNumeric, Version};
use semver_parser::lexer::{Error, Lexer, Token};

fn parse_part<'input>(
    input: &'input [Result<Token<'input>, Error>],
) -> (u64, &'input [Result<Token<'input>, Error>]) {
    let (part, rest) = match input {
        [Ok(Token::Numeric(numeric)), Ok(Token::Dot), rest @ ..]
        | [Ok(Token::Numeric(numeric)), rest @ ..] => (Some(*numeric), rest),
        rest => (None, rest),
    };

    (part.unwrap_or(0), rest)
}

fn parse_pre<'input>(
    input: &'input [Result<Token<'input>, Error>],
) -> (Option<&str>, &'input [Result<Token<'input>, Error>]) {
    match &input {
        [Ok(Token::Hyphen), Ok(Token::AlphaNumeric(pre)), rest @ ..]
        | [Err(Error::UnexpectedChar('/')), Ok(Token::AlphaNumeric(pre)), rest @ ..] => {
            (Some(pre), rest)
        }
        rest => (None, rest),
    }
}

fn skip_junk<'input>(
    input: &'input [Result<Token<'input>, Error>],
) -> &'input [Result<Token<'input>, Error>] {
    let offset = input.iter().position(|token| match token {
        Ok(Token::Numeric(_)) => true,
        _ => false,
    });

    if let Some(offset) = offset {
        &input[offset..]
    } else {
        input
    }
}

pub fn parse(input: &str) -> Version {
    let input: Vec<_> = Lexer::new(&input[..]).collect();

    let input = skip_junk(&input[..]);
    let (major, input) = parse_part(&input[..]);
    let (minor, input) = parse_part(&input[..]);
    let (patch, input) = parse_part(&input[..]);
    let (pre, _) = parse_pre(&input[..]);

    let mut version = Version::new(major, minor, patch);

    if let Some(pre) = pre {
        version.pre.push(AlphaNumeric(pre.to_owned()));
    }

    version
}
