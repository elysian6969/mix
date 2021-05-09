use std::ops::Deref;
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

impl ufmt::uDebug for GroupId {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        ufmt::uDisplay::fmt(self.group_id.deref(), f)
    }
}

impl ufmt::uDisplay for GroupId {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        ufmt::uDisplay::fmt(self.group_id.deref(), f)
    }
}
