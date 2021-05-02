use crate::source::Source;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    #[serde(default = "BTreeSet::new")]
    pub depends: BTreeSet<String>,
    pub source: BTreeSet<Source>,
}
