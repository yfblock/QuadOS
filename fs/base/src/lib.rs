//! File System base module.
//! Includes the interface definitions, struct definitions and useful methods(TODO).

#![no_std]

extern crate alloc;

use core::marker::PhantomData;

use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{collections::linked_list::LinkedList, sync::Weak};
use lock_api::{Mutex, RawMutex, RawRwLock, RwLock};
pub use syscalls::Errno;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct OpenFlags: usize {
        // reserve 3 bits for the access mode
        const RDONLY      = 0;
        const WRONLY      = 1;
        const RDWR        = 2;
        const ACCMODE     = 3;
        const CREAT       = 0o100;
        const EXCL        = 0o200;
        const NOCTTY      = 0o400;
        const TRUNC       = 0o1000;
        const APPEND      = 0o2000;
        const NONBLOCK    = 0o4000;
        const DSYNC       = 0o10000;
        const SYNC        = 0o4010000;
        const RSYNC       = 0o4010000;
        const DIRECTORY   = 0o200000;
        const NOFOLLOW    = 0o400000;
        const CLOEXEC     = 0o2000000;

        const ASYNC       = 0o20000;
        const DIRECT      = 0o40000;
        const LARGEFILE   = 0o100000;
        const NOATIME     = 0o1000000;
        const PATH        = 0o10000000;
        const TMPFILE     = 0o20200000;
    }
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct StatMode: u32 {
        const NULL  = 0;
        /// Type
        const TYPE_MASK = 0o170000;
        /// FIFO
        const FIFO  = 0o010000;
        /// character device
        const CHAR  = 0o020000;
        /// directory
        const DIR   = 0o040000;
        /// block device
        const BLOCK = 0o060000;
        /// ordinary regular file
        const FILE  = 0o100000;
        /// symbolic link
        const LINK  = 0o120000;
        /// socket
        const SOCKET = 0o140000;

        /// Set-user-ID on execution.
        const SET_UID = 0o4000;
        /// Set-group-ID on execution.
        const SET_GID = 0o2000;

        /// Read, write, execute/search by owner.
        const OWNER_MASK = 0o700;
        /// Read permission, owner.
        const OWNER_READ = 0o400;
        /// Write permission, owner.
        const OWNER_WRITE = 0o200;
        /// Execute/search permission, owner.
        const OWNER_EXEC = 0o100;

        /// Read, write, execute/search by group.
        const GROUP_MASK = 0o70;
        /// Read permission, group.
        const GROUP_READ = 0o40;
        /// Write permission, group.
        const GROUP_WRITE = 0o20;
        /// Execute/search permission, group.
        const GROUP_EXEC = 0o10;

        /// Read, write, execute/search by others.
        const OTHER_MASK = 0o7;
        /// Read permission, others.
        const OTHER_READ = 0o4;
        /// Write permission, others.
        const OTHER_WRITE = 0o2;
        /// Execute/search permission, others.
        const OTHER_EXEC = 0o1;
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct PollEvent: u16 {
        const NONE = 0;
        const POLLIN = 0x001;
        const POLLPRI = 0x002;
        const POLLOUT = 0x004;
        const POLLRDNORM = 0x040;
        const POLLRDBAND = 0x080;
        const POLLWRNORM = 0x100;
        const POLLWRBAND = 0x200;
        const POLLMSG = 0x400;
        const POLLREMOVE = 0x1000;
        const POLLRDHUP = 0x2000;
        const POLLERR = 0x008;
        const POLLHUP = 0x010;
        const POLLNVAL = 0x020;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FileType {
    File,
    Directory,
    Device,
    Socket,
    Link,
}

#[derive(Debug, Copy, Clone)]
pub enum SeekFrom {
    SET(usize),
    CURRENT(isize),
    END(isize),
}

#[derive(Debug, Clone)]
pub struct Metadata<'a> {
    pub filename: &'a str,
    pub inode: usize,
    pub file_type: FileType,
    pub size: usize,
    pub childrens: usize,
}

pub struct DirEntry {
    pub filename: String,
    pub len: usize,
    pub file_type: FileType,
}

pub trait FileSystem: Send + Sync {
    fn root_dir(&self) -> Arc<dyn INodeInterface>;
    fn name(&self) -> &str;
    fn flush(&self) -> FsResult<()> {
        Err(Errno::EPERM)
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct TimeSpec {
    /// Seconds
    pub sec: u64,
    /// Nanoseconds
    pub nsec: u64,
}

/// Implements Some useful methods for [TimeSpec]
impl TimeSpec {
    /// Get the seconds for now.
    pub const UTIME_NOW: u64 = 0x3fffffff;
    /// Keep the filed time. Not need to change.
    pub const UTIME_OMIT: u64 = 0x3ffffffe;
    pub fn to_nsec(&self) -> u64 {
        self.sec * 1_000_000_000 + self.nsec
    }
}

#[repr(C)]
#[derive(Debug)]
#[cfg(not(target_arch = "x86_64"))]
pub struct Stat {
    pub dev: u64,        // 设备号
    pub ino: u64,        // inode
    pub mode: StatMode,  // 设备mode
    pub nlink: u32,      // 文件links
    pub uid: u32,        // 文件uid
    pub gid: u32,        // 文件gid
    pub rdev: u64,       // 文件rdev
    pub __pad: u64,      // 保留
    pub size: u64,       // 文件大小
    pub blksize: u32,    // 占用块大小
    pub __pad2: u32,     // 保留
    pub blocks: u64,     // 占用块数量
    pub atime: TimeSpec, // 最后访问时间
    pub mtime: TimeSpec, // 最后修改时间
    pub ctime: TimeSpec, // 最后创建时间
}

#[repr(C)]
#[derive(Debug)]
#[cfg(target_arch = "x86_64")]
pub struct Stat {
    pub dev: u64,        // 设备号
    pub ino: u64,        // inode
    pub nlink: u64,      // 文件links
    pub mode: StatMode,  // 设备mode
    pub uid: u32,        // 文件uid
    pub gid: u32,        // 文件gid
    pub _pad0: u32,      // reserved field
    pub rdev: u64,       // 文件rdev
    pub size: u64,       // 文件大小
    pub blksize: u32,    // 占用块大小
    pub __pad2: u32,     // 保留
    pub blocks: u64,     // 占用块数量
    pub atime: TimeSpec, // 最后访问时间
    pub mtime: TimeSpec, // 最后修改时间
    pub ctime: TimeSpec, // 最后创建时间
}

#[repr(C)]
pub struct StatFS {
    pub ftype: u64,   // 文件系统的类型
    pub bsize: u64,   // 经优化后的传输块的大小
    pub blocks: u64,  // 文件系统数据块总数
    pub bfree: u64,   // 可用块数
    pub bavail: u64,  // 普通用户能够获得的块数
    pub files: u64,   // 文件结点总数
    pub ffree: u64,   // 可用文件结点数
    pub fsid: u64,    // 文件系统标识
    pub namelen: u64, // 文件名的最大长度
}

/// The result alias for the [core::result::Result<T, Errno>]
/// This result used in methods of the trait [INodeInterface].
pub type FsResult<T> = core::result::Result<T, Errno>;

/// The file trait [INodeInterface].
/// You can implement it for General-file, directory and so on.
/// All method in the trait return [FsResult], Error is [Errno].
pub trait INodeInterface: Sync + Send {
    /// Get the metadata of the file.
    fn metadata(&self) -> FsResult<Metadata> {
        Err(Errno::EACCES)
    }

    /// Read data to buffer in the offset.
    fn readat(&self, _offset: usize, _buffer: &mut [u8]) -> FsResult<usize> {
        Err(Errno::EACCES)
    }

    /// Write the buffer in the offset.
    fn writeat(&self, _offset: usize, _buffer: &[u8]) -> FsResult<usize> {
        Err(Errno::EACCES)
    }

    /// Create a new directory with name
    fn mkdir(&self, _name: &str) -> FsResult<Arc<dyn INodeInterface>> {
        Err(Errno::EACCES)
    }

    /// Remove a directory with name
    fn rmdir(&self, _name: &str) -> FsResult<()> {
        Err(Errno::EACCES)
    }

    /// Rename a file with name
    fn remove(&self, _name: &str) -> FsResult<()> {
        Err(Errno::EACCES)
    }

    /// Get the all files(not includes files in the sub-directory) in the current directory
    fn read_dir(&self) -> FsResult<Vec<DirEntry>> {
        Err(Errno::EACCES)
    }

    /// Open a file in the current directory(not includes files in the sub-directory)
    fn open(&self, _name: &str, _flags: OpenFlags) -> FsResult<Arc<dyn INodeInterface>> {
        Err(Errno::EACCES)
    }

    /// IOCTL for device and socket
    fn ioctl(&self, _command: usize, _arg: usize) -> FsResult<usize> {
        Err(Errno::EACCES)
    }

    /// truncate the file with size
    fn truncate(&self, _size: usize) -> FsResult<()> {
        Err(Errno::EACCES)
    }

    /// Flush the file cache.
    fn flush(&self) -> FsResult<()> {
        Err(Errno::EACCES)
    }

    /// Get the linked path of the link file.
    fn resolve_link(&self) -> FsResult<String> {
        Err(Errno::EACCES)
    }

    /// Create a link file in the current directory with filename(name) and link path(src).
    ///
    /// Hardlink the link file
    fn link(&self, _name: &str, _src: &str) -> FsResult<()> {
        Err(Errno::EACCES)
    }

    /// Create a symbolic link file in the current directory with filename(name) and link path(src).
    ///
    /// Soft link
    fn sym_link(&self, _name: &str, _src: &str) -> FsResult<()> {
        Err(Errno::EACCES)
    }

    /// Remove the link.
    fn unlink(&self, _name: &str) -> FsResult<()> {
        Err(Errno::EACCES)
    }

    /// Get the [Stat] information of the file or directory.
    fn stat(&self, _stat: &mut Stat) -> FsResult<()> {
        Err(Errno::EACCES)
    }

    /// get filesystem statistics
    fn statfs(&self, _statfs: &mut StatFS) -> FsResult<()> {
        Err(Errno::EACCES)
    }

    /// Change the file's time information
    fn utimes(&self, _times: &mut [TimeSpec]) -> FsResult<()> {
        Err(Errno::EACCES)
    }

    /// Poll a device or socket file.
    fn poll(&self, _events: PollEvent) -> FsResult<PollEvent> {
        Err(Errno::EACCES)
    }
}

/// Dentry Node represents
pub struct Dentry<R: RawMutex, F: FSTrait> {
    file: Arc<dyn INodeInterface>,
    inode: Mutex<R, BTreeMap<String, LinkedList<Arc<Dentry<R, F>>>>>,
    parent: Weak<Dentry<R, F>>,
}

impl<R: RawMutex, F: FSTrait> Dentry<R, F> {
    pub(self) fn open(
        self: Arc<Self>,
        file_name: &str,
        flags: OpenFlags,
    ) -> FsResult<Arc<Dentry<R, F>>> {
        match file_name {
            ".." => self.parent.upgrade().ok_or(Errno::ENOENT),
            "." => Ok(self.clone()),
            _ => {
                let mut map = self.inode.lock();
                match map.get_mut(file_name).ok_or(Errno::ENOENT) {
                    Ok(dentry) => dentry.back().ok_or(Errno::ENOENT).cloned(),
                    Err(_) => {
                        let file = self.file.open(file_name, flags)?;
                        let mut list = LinkedList::new();
                        let dentry = Arc::new(Dentry {
                            file,
                            inode: Mutex::new(BTreeMap::new()),
                            parent: Arc::downgrade(&self),
                        });
                        list.push_back(dentry.clone());
                        map.insert(String::from(file_name), list);
                        Ok(dentry)
                    }
                }
            }
        }
    }
}

/// File System Tree.
pub struct FileTree<R: RawMutex, W: RawRwLock, F: FSTrait>(Arc<FileTreeInner<R, W, F>>);

/// File System Tree, includes fs and tree
pub struct FileTreeInner<R: RawMutex, W: RawRwLock, F: FSTrait> {
    /// FS fields, includes fs and its mount path for permanent access.
    fs: RwLock<W, LinkedList<(String, Arc<dyn FileSystem>)>>,
    /// FileSystem Tree, Root Inode
    tree: Mutex<R, LinkedList<Arc<Dentry<R, F>>>>,
}

impl<R: RawMutex + 'static, W: RawRwLock, F: FSTrait> FileTree<R, W, F> {
    /// Create a new blank file system tree.
    ///
    /// Clean even not contains root node and root fs.
    pub fn new() -> Self {
        Self(Arc::new(FileTreeInner {
            fs: RwLock::new(LinkedList::new()),
            tree: Mutex::new(LinkedList::new()),
        }))
    }

    /// Mount a file system to the specified path.
    ///
    /// TODO: Mount a directory to the specific path.
    pub fn mount(&self, path: &str, fs: Arc<dyn FileSystem>) -> FsResult<()> {
        if path == "/" || path == "" || path == "." {
            let root_dir = fs.root_dir();
            self.0.fs.write().push_back((String::from(path), fs));
            self.0.tree.lock().push_back(Arc::new(Dentry {
                file: root_dir,
                inode: Mutex::new(BTreeMap::new()),
                parent: Weak::new(),
            }));
        } else {
            let file = self
                .root()
                // Check File Exists.
                .open(path, OpenFlags::DIRECTORY)?
                // Get Parent Dir to mount.
                .open("..", OpenFlags::DIRECTORY)?;
            file.dentry
                .inode
                .lock()
                .get_mut(path.split("/").last().unwrap())
                .unwrap()
                .push_back(Arc::new(Dentry {
                    file: fs.root_dir(),
                    inode: Mutex::new(BTreeMap::new()),
                    parent: Arc::downgrade(&file.dentry),
                }));
        }
        Ok(())
    }

    pub fn root(&self) -> DentryFile<R, W, F> {
        let dentry = self.0.root();
        DentryFile {
            file: dentry.file.clone(),
            dentry,
            path: String::from("/"),
            fs: self.0.clone(),
        }
    }
}

impl<R: RawMutex, W: RawRwLock, F: FSTrait> FileTreeInner<R, W, F> {
    /// Get the root directory.
    #[inline]
    pub(self) fn root(self: &Arc<Self>) -> Arc<Dentry<R, F>> {
        self.tree
            .lock()
            .back()
            .cloned()
            .expect("Here is not a valid root directory")
    }
}

/// Dentry File. This will be used in the task File Descriptor.
#[derive(Clone)]
pub struct DentryFile<R: RawMutex, W: RawRwLock, F: FSTrait> {
    file: Arc<dyn INodeInterface>,
    path: String,
    dentry: Arc<Dentry<R, F>>,
    fs: Arc<FileTreeInner<R, W, F>>,
}

impl<R: RawMutex + 'static, W: RawRwLock, F: FSTrait> DentryFile<R, W, F> {
    /// Open a file in the current directory(not includes files in the sub-directory)
    #[inline]
    pub fn open(&self, name: &str, flags: OpenFlags) -> FsResult<DentryFile<R, W, F>> {
        // TODO: Split path and file, Open file through entry tree.
        let (mut dentry, skip) = match name.starts_with("/") {
            true => (self.fs.root(), 1),
            false => (self.dentry.clone(), 0),
        };

        let mut paths = name.split("/").skip(skip).peekable();
        while let Some(path) = paths.next() {
            dentry = match paths.peek() {
                Some(_) => dentry.open(path, OpenFlags::DIRECTORY)?,
                None => dentry.open(path, flags)?,
            }
        }
        Ok(DentryFile {
            file: dentry.file.clone(),
            path: format!("{}/{}", self.path, name),
            dentry,
            fs: self.fs.clone(),
        })
    }

    /// Get the metadata of the file.
    #[inline]
    pub fn metadata(&self) -> FsResult<Metadata> {
        self.file.metadata()
    }

    /// Read data to buffer in the offset.
    #[inline]
    pub fn readat(&self, offset: usize, buffer: &mut [u8]) -> FsResult<usize> {
        self.file.readat(offset, buffer)
    }

    /// Write the buffer in the offset.
    #[inline]
    pub fn writeat(&self, offset: usize, buffer: &[u8]) -> FsResult<usize> {
        self.file.writeat(offset, buffer)
    }

    /// Create a new directory with name
    #[inline]
    pub fn mkdir(&self, name: &str) -> FsResult<Arc<dyn INodeInterface>> {
        self.file.mkdir(name)
    }

    /// Remove a directory with name
    #[inline]
    pub fn rmdir(&self, name: &str) -> FsResult<()> {
        self.file.rmdir(name)
    }

    /// Rename a file with name
    #[inline]
    pub fn remove(&self, name: &str) -> FsResult<()> {
        self.file.remove(name)
    }

    /// Get the all files(not includes files in the sub-directory) in the current directory
    #[inline]
    pub fn read_dir(&self) -> FsResult<Vec<DirEntry>> {
        self.file.read_dir()
    }

    /// IOCTL for device and socket
    #[inline]
    pub fn ioctl(&self, command: usize, arg: usize) -> FsResult<usize> {
        self.file.ioctl(command, arg)
    }

    /// truncate the file with size
    #[inline]
    pub fn truncate(&self, size: usize) -> FsResult<()> {
        self.file.truncate(size)
    }

    /// Flush the file cache.
    #[inline]
    pub fn flush(&self) -> FsResult<()> {
        self.file.flush()
    }

    /// Get the linked path of the link file.
    #[inline]
    pub fn resolve_link(&self) -> FsResult<String> {
        self.file.resolve_link()
    }

    /// Create a link file in the current directory with filename(name) and link path(src).
    ///
    /// Hardlink the link file
    #[inline]
    pub fn link(&self, name: &str, src: &str) -> FsResult<()> {
        self.file.link(name, src)
    }

    /// Create a symbolic link file in the current directory with filename(name) and link path(src).
    ///
    /// Soft link
    #[inline]
    pub fn sym_link(&self, name: &str, src: &str) -> FsResult<()> {
        self.file.sym_link(name, src)
    }

    /// Remove the link.
    #[inline]
    pub fn unlink(&self, name: &str) -> FsResult<()> {
        self.file.unlink(name)
    }

    /// Get the [Stat] information of the file or directory.
    #[inline]
    pub fn stat(&self, stat: &mut Stat) -> FsResult<()> {
        self.file.stat(stat)
    }

    /// get filesystem statistics
    #[inline]
    pub fn statfs(&self, statfs: &mut StatFS) -> FsResult<()> {
        self.file.statfs(statfs)
    }

    /// Change the file's time information
    #[inline]
    pub fn utimes(&self, times: &mut [TimeSpec]) -> FsResult<()> {
        self.file.utimes(times)
    }

    /// Poll a device or socket file.
    #[inline]
    pub fn poll(&self, events: PollEvent) -> FsResult<PollEvent> {
        self.file.poll(events)
    }
}

/// FSPages Container, first arg is address, second arg is page count.
pub struct FSPage<F: FSTrait>(usize, usize, PhantomData<F>);

impl<F: FSTrait> FSPage<F> {
    /// Get the buffer for the page.
    #[inline]
    pub fn get_buffer(&self) -> &'static mut [u8] {
        F::get_buffer_from_phys(self.0)
    }

    /// Create a new FSPage instance.
    pub fn new(addr: usize, page_count: usize) -> Self {
        Self(addr, page_count, PhantomData::default())
    }
}

/// Drop pages when FSPage is dropped
impl<F: FSTrait> Drop for FSPage<F> {
    fn drop(&mut self) {
        F::dealloc_page(self.0, self.1)
    }
}

/// The traits includes Generic methods for FileSystem implementations.
pub trait FSTrait<F: FSTrait = Self>: Sync + Send + 'static {
    /// Page size for the File system subsystem.
    const PAGE_SIZE: usize = 4096;

    /// Allocate for count physical pages
    fn alloc_page(count: usize) -> FSPage<F>;
    /// Deallocate count physical pages starting from address
    fn dealloc_page(addr: usize, count: usize);
    /// Convert physical address to virtual address
    fn phys_to_virt(phys: usize) -> usize;
    /// Convert virtual address to physical address
    fn virt_to_phys(virt: usize) -> usize;
    /// Get buffer from physical address
    fn get_buffer_from_phys(physical: usize) -> &'static mut [u8] {
        let vaddr = Self::phys_to_virt(physical);
        unsafe { core::slice::from_raw_parts_mut(vaddr as *mut u8, Self::PAGE_SIZE) }
    }
}
