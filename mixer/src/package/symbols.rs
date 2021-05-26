#[derive(Debug)]
pub struct Symbols {
    pub down: &'static str,
    pub tee: &'static str,
    pub ell: &'static str,
    pub right: &'static str,
}

impl Symbols {
    pub const fn ascii() -> Self {
        Self {
            down: "|",
            tee: "|",
            ell: "`",
            right: "-",
        }
    }

    pub const fn utf8() -> Self {
        Self {
            down: "│",
            tee: "├",
            ell: "└",
            right: "─",
        }
    }
}
