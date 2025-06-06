use alloc::sync::Arc;

use crate::{devices::BlockDevice, fs::{simplefs::{dentry::SpDentry, inode::SpInode}, vfs::{fstype::{FSType, FSTypeInner, MountFlags}, Dentry, DentryState, DCACHE}, SuperBlock, SuperBlockInner}};

use super::superblock::ProcSuperBlock;


pub struct ProcFSType {
    inner: FSTypeInner,
}

impl ProcFSType {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: FSTypeInner::new("procfs"),
        })
    }
}

impl FSType for ProcFSType {
    fn inner(&self) -> &FSTypeInner {
        &self.inner
    }

    fn mount(&'static self, name: &str, parent: Option<Arc<dyn Dentry>>, _flags: MountFlags, dev: Option<Arc<dyn BlockDevice>>) -> Option<Arc<dyn Dentry>> {
        let fs_type = unsafe {
            let ptr: *const dyn FSType = self;
            Arc::from_raw(ptr)
        };
        let sb = ProcSuperBlock::new(SuperBlockInner::new(dev, fs_type.clone()));
        let root_inode = SpInode::new(Arc::downgrade(&sb));
        let root_dentry = SpDentry::new(name, parent.clone());
        root_dentry.set_inode(root_inode);
        root_dentry.set_state(DentryState::USED);
        sb.set_root_dentry(root_dentry.clone());
        DCACHE.lock().insert(root_dentry.path(), root_dentry.clone());
        self.add_sb(&root_dentry.path(), sb);
        Some(root_dentry)
    }

    fn kill_sb(&self) -> isize {
        todo!()
    }
}


