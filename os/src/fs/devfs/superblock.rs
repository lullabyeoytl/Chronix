//! device file system super block

use alloc::sync::Arc;

use crate::{devices::BlockDevice, fs::{vfs::Inode, SuperBlock, SuperBlockInner}};

pub struct DevSuperBlock {
    inner: SuperBlockInner,
}

impl DevSuperBlock {
    pub fn new(inner: SuperBlockInner) -> Arc<dyn SuperBlock> {
        Arc::new(Self { inner })
    }
}

impl SuperBlock for DevSuperBlock {
    fn inner(&self) -> &SuperBlockInner {
        &self.inner
    }
    fn get_root_inode(&'static self, _name: &str) -> Arc<dyn Inode> {
        self.inner().root.get().unwrap().clone().inode().unwrap()
    }
}