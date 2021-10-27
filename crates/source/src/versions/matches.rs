use super::{Entry, Iter};
use mix_version::Requirement;

pub struct Matches<'a> {
    pub(crate) iter: Iter<'a>,
    pub(crate) requirement: &'a Requirement,
}

impl<'a> Iterator for Matches<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find(|entry| entry.matches(self.requirement))
    }
}
