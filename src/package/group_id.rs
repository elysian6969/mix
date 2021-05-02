use std::fmt;
use std::sync::Arc;

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct GroupId {
    group_id: Arc<String>,
}

impl GroupId {
    pub fn new(group_id: impl Into<String>) -> Self {
        Self {
            group_id: Arc::new(group_id.into()),
        }
    }

    pub fn as_str(&self) -> &str {
        self.group_id.as_str()
    }
}

impl fmt::Debug for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.group_id, f)
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.group_id, f)
    }
}
