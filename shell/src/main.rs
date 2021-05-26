use self::lexer::Lexer;
use self::parser::Parser;
use std::{env, fs};

pub mod lexer;

pub mod parser {
    use super::lexer::{Lexer, Token};
    use std::path::PathBuf;

    #[derive(Debug)]
    pub struct Parser<'a> {
        input: &'a str,
        lexer: Lexer<'a>,
        one: Option<Token<'a>>,
        two: Option<Token<'a>>,
        three: Option<Token<'a>>,
    }

    impl<'a> Parser<'a> {
        /// construct new parser
        pub fn new(input: &'a str) -> Self {
            let mut lexer = Lexer::new(input);
            let one = lexer.next();
            let two = lexer.next();
            let three = lexer.next();

            Self {
                input,
                lexer,
                one,
                two,
                three,
            }
        }

        /// increment
        fn step(&mut self) {
            self.one = self.two;
            self.two = self.three;
            self.three = self.lexer.next();
        }

        /// returns one token
        fn one(&self) -> Option<Token<'a>> {
            self.one
        }

        /// returns two tokens
        fn two(&self) -> Option<(Token<'a>, Token<'a>)> {
            self.one().and_then(|one| Some((one, self.two?)))
        }

        /// returns three tokens
        fn three(&self) -> Option<(Token<'a>, Token<'a>, Token<'a>)> {
            self.two()
                .and_then(|(one, two)| Some((one, two, self.three?)))
        }

        /// optionally consume shebang
        pub fn shebang(&mut self) -> Option<PathBuf> {
            match self.one() {
                Some(Token::Shebang(shebang)) => {
                    self.step();

                    Some(shebang[2..].trim().into())
                }
                _ => None,
            }
        }

        /// parse script
        pub fn parse(&mut self) -> Script {
            let interpreter = self.shebang();

            Script { interpreter }
        }
    }

    #[derive(Debug)]
    pub struct Script {
        interpreter: Option<PathBuf>,
    }
}

fn main() {
    let filename = env::args().nth(1).unwrap();
    let input = fs::read_to_string(filename).unwrap();
    let tokens: Vec<_> = Lexer::new(&input).collect();
    let script = Parser::new(&input).parse();

    println!("{:?}", tokens);
    println!();
    println!("{:?}", script);
}
