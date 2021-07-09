use crate::source::Source;
use serde::Deserialize;
use std::collections::BTreeSet;
use ufmt::derive::uDebug;

#[derive(Deserialize, uDebug)]
pub struct Metadata {
    #[serde(default = "BTreeSet::new")]
    pub depends: BTreeSet<String>,
    pub source: BTreeSet<Source>,
}
