use super::Iter;
use crate::{Requirement, Version};

pub struct Matches<'a> {
    pub(crate) iter: Iter<'a>,
    pub(crate) requirement: &'a Requirement,
}

impl<'a> Iterator for Matches<'a> {
    type Item = &'a Version;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find(|version| version.matches(&self.requirement))
    }
}
