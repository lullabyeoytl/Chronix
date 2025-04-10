use crate::fs::{ext4::Ext4File, vfs::{inode::InodeMode, Dentry, DentryInner, DentryState, File, DCACHE}, OpenFlags, SuperBlock};

use alloc::{sync::Arc, vec::Vec};
use log::info;

use lwext4_rust::InodeTypes;

/// ext4 file system dentry implement for VFS
pub struct Ext4Dentry {
    inner: DentryInner,
}

unsafe impl Send for Ext4Dentry {}
unsafe impl Sync for Ext4Dentry {}

impl Ext4Dentry {
    pub fn new(
        name: &str,
        superblock: Arc<dyn SuperBlock>,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, superblock, parent)
        });
        dentry
    }
}

impl Dentry for Ext4Dentry {
    fn inner(&self) -> &DentryInner {
        &self.inner
    }
    fn new(&self,
        name: &str,
        superblock: Arc<dyn SuperBlock>,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, superblock, parent)
        });
        dentry
    }
    fn open(self: Arc<Self>, flags: OpenFlags) -> Option<Arc<dyn File>> {
        assert!(self.state() == DentryState::USED);
        let (readable, writable) = flags.read_write();
        Some(Arc::new(Ext4File::new(readable, writable, self.clone())))
    }
}
