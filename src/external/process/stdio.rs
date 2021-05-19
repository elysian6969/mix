use std::convert::{TryFrom, TryInto};

pub struct Stdio {
    crate imp: Imp,
}

crate enum Imp {
    Std(std::process::Stdio),
    Pts,
}

impl<T: Into<std::process::Stdio>> From<T> for Imp {
    fn from(cfg: T) -> Self {
        Self::Std(cfg.into())
    }
}

impl Stdio {
    pub fn inherit() -> Self {
        Imp::from(std::process::Stdio::inherit()).into()
    }

    pub fn null() -> Self {
        Imp::from(std::process::Stdio::null()).into()
    }

    pub fn piped() -> Self {
        Imp::from(std::process::Stdio::piped()).into()
    }

    pub fn pts() -> Self {
        Imp::Pts.into()
    }
}

impl From<Imp> for Stdio {
    fn from(imp: Imp) -> Self {
        Self { imp }
    }
}

impl<T: Into<std::process::Stdio>> From<T> for Stdio {
    fn from(cfg: T) -> Self {
        Stdio::from(Imp::from(cfg))
    }
}
