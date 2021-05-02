use super::{GroupId, Metadata, PackageId};

#[derive(Debug)]
pub struct Node {
    pub group_id: GroupId,
    pub package_id: PackageId,
    pub metadata: Metadata,
}

impl Node {
    pub fn new(group_id: GroupId, package_id: PackageId, metadata: Metadata) -> Self {
        Self {
            group_id,
            package_id,
            metadata,
        }
    }
}
