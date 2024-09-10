#![no_std]
#![feature(extract_if)]
#[macro_use]
extern crate alloc;

use core::marker::PhantomData;

use alloc::{string::String, sync::Arc, vec::Vec};
use fs_base::{
    DirEntry, Errno, FSPage, FSTrait, FileSystem, FileType, FsResult, INodeInterface, Metadata,
    OpenFlags, Stat, StatMode, TimeSpec,
};
use lock_api::{Mutex, RawMutex};

pub struct RamFs<R: RawMutex, F: FSTrait> {
    root: Arc<FileContainer<R, F>>,
}

impl<R: RawMutex, F: FSTrait> RamFs<R, F> {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            root: Arc::new(FileContainer::Dir(RamDirInner {
                name: String::from(""),
                children: Mutex::<R, _>::new(Vec::new()),
            })),
        })
    }
}

impl<R: RawMutex + Sync + Send + 'static, F: FSTrait> FileSystem for RamFs<R, F> {
    fn root_dir(&self) -> Arc<dyn INodeInterface> {
        self.root.clone()
    }

    fn name(&self) -> &str {
        "ramfs"
    }
}

pub struct RamDirInner<R: RawMutex, F: FSTrait> {
    name: String,
    children: Mutex<R, Vec<Arc<FileContainer<R, F>>>>,
}

// TODO: use frame insteads of Vec.
pub struct RamFileInner<R: RawMutex, F: FSTrait> {
    name: String,
    len: Mutex<R, usize>,
    pages: Mutex<R, Vec<FSPage<F>>>,
    times: Mutex<R, [TimeSpec; 3]>, // ctime, atime, mtime.
    fs_trait: PhantomData<F>,
}

#[allow(dead_code)]
pub struct RamLinkInner<R: RawMutex> {
    name: String,
    link_file: Mutex<R, String>,
}

pub enum FileContainer<R: RawMutex, F: FSTrait> {
    File(RamFileInner<R, F>),
    Dir(RamDirInner<R, F>),
    Link(RamLinkInner<R>),
}

impl<R: RawMutex + Send + Sync + 'static, F: FSTrait> FileContainer<R, F> {
    #[inline]
    fn filename(&self) -> &str {
        match self {
            FileContainer::File(file) => &file.name,
            FileContainer::Dir(dir) => &dir.name,
            FileContainer::Link(link) => &link.name,
        }
    }
}

impl<R: RawMutex + Send + Sync + 'static, F: FSTrait> INodeInterface for FileContainer<R, F> {
    fn open(&self, name: &str, flags: OpenFlags) -> FsResult<Arc<dyn INodeInterface>> {
        let dir = match self {
            FileContainer::Dir(dir) => dir,
            _ => return Err(Errno::ENOTDIR),
        };
        if flags.contains(OpenFlags::CREAT) {
            // Find file, return VfsError::AlreadyExists if file exists
            dir.children
                .lock()
                .iter()
                .find(|x| x.filename() == name)
                .map_or(Ok(()), |_| Err(Errno::EEXIST))?;
            let new_file = Arc::new(FileContainer::File(RamFileInner {
                name: String::from(name),
                // content: Mutex::new(Vec::new()),
                times: Mutex::new([Default::default(); 3]),
                len: Mutex::new(0),
                pages: Mutex::new(vec![]),
                fs_trait: PhantomData::default(),
            }));

            dir.children.lock().push(new_file.clone());

            Ok(new_file)
        } else {
            dir.children
                .lock()
                .iter()
                .find(|x| x.filename() == name)
                .cloned()
                .map(|x| x as Arc<dyn INodeInterface>)
                .ok_or(Errno::ENOENT)
        }
    }

    fn mkdir(&self, name: &str) -> FsResult<Arc<dyn INodeInterface>> {
        let dir = match self {
            FileContainer::Dir(dir) => dir,
            _ => return Err(Errno::ENOTDIR),
        };
        // Find file, return VfsError::AlreadyExists if file exists
        dir.children
            .lock()
            .iter()
            .find(|x| x.filename() == name)
            .map_or(Ok(()), |_| Err(Errno::EEXIST))?;

        let new_dir = Arc::new(FileContainer::Dir(RamDirInner {
            name: String::from(name),
            children: Mutex::new(Vec::new()),
        }));

        dir.children.lock().push(new_dir.clone());

        Ok(new_dir)
    }

    fn rmdir(&self, name: &str) -> FsResult<()> {
        let dir = match self {
            FileContainer::Dir(dir) => dir,
            _ => return Err(Errno::ENOTDIR),
        };
        // TODO: identify whether the dir is empty(through metadata.childrens)
        // return DirectoryNotEmpty if not empty.
        let len = dir
            .children
            .lock()
            .extract_if(|x| match x.as_ref() {
                FileContainer::Dir(x) => x.name == name,
                _ => false,
            })
            .count();
        match len > 0 {
            true => Ok(()),
            false => Err(Errno::ENOENT),
        }
    }

    fn read_dir(&self) -> FsResult<Vec<DirEntry>> {
        let dir = match self {
            FileContainer::Dir(dir) => dir,
            _ => return Err(Errno::ENOTDIR),
        };
        Ok(dir
            .children
            .lock()
            .iter()
            .map(|x| match x.as_ref() {
                FileContainer::File(file) => DirEntry {
                    filename: file.name.clone(),
                    // len: file.content.lock().len(),
                    len: *file.len.lock(),
                    file_type: FileType::File,
                },
                FileContainer::Dir(dir) => DirEntry {
                    filename: dir.name.clone(),
                    len: 0,
                    file_type: FileType::Directory,
                },
                FileContainer::Link(link) => DirEntry {
                    filename: link.name.clone(),
                    len: 0,
                    file_type: FileType::Link,
                },
            })
            .collect())
    }

    fn remove(&self, name: &str) -> FsResult<()> {
        let dir = match self {
            FileContainer::Dir(dir) => dir,
            _ => return Err(Errno::ENOTDIR),
        };
        let len = dir
            .children
            .lock()
            .extract_if(|x| match x.as_ref() {
                FileContainer::File(x) => x.name == name,
                FileContainer::Dir(_) => false,
                FileContainer::Link(x) => x.name == name,
            })
            .count();

        match len > 0 {
            true => Ok(()),
            false => Err(Errno::ENOENT),
        }
    }

    fn unlink(&self, name: &str) -> FsResult<()> {
        self.remove(name)
    }

    fn metadata(&self) -> FsResult<Metadata> {
        /*
            // Link metadata
            Ok(Metadata {
                filename: &self.inner.name,
                inode: self as *const RamLink<R> as usize,
                file_type: FileType::Link,
                size: self.inner.name.len(),
                childrens: 0,
            })
        */
        /*
            // File metadata
            Ok(Metadata {
                filename: &self.inner.name,
                inode: 0,
                file_type: FileType::File,
                size: *self.inner.len.lock(),
                childrens: 0,
            })
        */
        todo!()
        // Ok(Metadata {
        //     filename: &self.inner.name,
        //     inode: 0,
        //     file_type: FileType::Directory,
        //     size: 0,
        //     childrens: self.inner.children.lock().len(),
        // })
    }

    fn stat(&self, stat: &mut Stat) -> FsResult<()> {
        /*
            // Link
            let metadata = self.metadata()?;
            stat.ino = metadata.inode as _;
            stat.blksize = 4096;
            stat.blocks = 8;
            stat.size = metadata.size as _;
            stat.uid = 0;
            stat.gid = 0;
            stat.mode = StatMode::LINK;
            Ok(())
        */
        /*
            // File
            let metadata = self.metadata()?;
            stat.ino = metadata.inode as _;
            stat.mode = StatMode::FILE; // TODO: add access mode
            stat.nlink = 1;
            stat.uid = 0;
            stat.gid = 0;
            stat.size = metadata.size as _;
            stat.blksize = 512;
            stat.blocks = 0;
            stat.rdev = 0; // TODO: add device id

            stat.atime = self.inner.times.lock()[1];
            stat.mtime = self.inner.times.lock()[2];
            Ok(())
        */
        let metadata = self.metadata()?;
        stat.ino = metadata.inode as _;
        stat.mode = StatMode::DIR; // TODO: add access mode
        stat.nlink = 1;
        stat.uid = 0;
        stat.gid = 0;
        stat.size = 0;
        stat.blksize = 512;
        stat.blocks = 0;
        stat.rdev = 0; // TODO: add device id
        stat.mtime = Default::default();
        stat.atime = Default::default();
        stat.ctime = Default::default();
        Ok(())
    }

    fn link(&self, name: &str, src: &str) -> FsResult<()> {
        let dir = match self {
            FileContainer::Dir(dir) => dir,
            _ => return Err(Errno::ENOTDIR),
        };
        // Find file, return VfsError::AlreadyExists if file exists
        dir.children
            .lock()
            .iter()
            .find(|x| x.filename() == name)
            .map_or(Ok(()), |_| Err(Errno::EEXIST))?;

        let new_link = Arc::new(FileContainer::Link(RamLinkInner {
            name: String::from(name),
            link_file: Mutex::new(String::from(src)),
        }));

        dir.children.lock().push(new_link);

        Ok(())
    }

    fn readat(&self, mut offset: usize, buffer: &mut [u8]) -> FsResult<usize> {
        let file = match self {
            FileContainer::File(file) => file,
            FileContainer::Dir(_) => return Err(Errno::EISDIR),
            _ => return Err(Errno::EBADF),
        };
        let mut buffer_off = 0;
        // let file_size = self.inner.content.lock().len();
        let file_size = *file.len.lock();
        let pages = file.pages.lock();
        match offset >= file_size {
            true => Ok(0),
            false => {
                let read_len = core::cmp::min(buffer.len(), file_size - offset);
                let mut last_len = read_len;
                loop {
                    let page_offset = offset % F::PAGE_SIZE;
                    let curr_size = core::cmp::min(F::PAGE_SIZE - page_offset, last_len);
                    if curr_size == 0 {
                        break;
                    }
                    let page_data = pages[offset / F::PAGE_SIZE].get_buffer();
                    buffer[buffer_off..buffer_off + curr_size]
                        .copy_from_slice(&page_data[page_offset..page_offset + curr_size]);
                    offset += curr_size;
                    last_len -= curr_size;
                    buffer_off += curr_size;
                }
                Ok(read_len)
            }
        }
    }

    fn writeat(&self, mut offset: usize, buffer: &[u8]) -> FsResult<usize> {
        let file = match self {
            FileContainer::File(file) => file,
            FileContainer::Dir(_) => return Err(Errno::EISDIR),
            _ => return Err(Errno::EBADF),
        };
        let mut buffer_off = 0;
        // Get the needed pages.
        let page_idx_max = (offset + buffer.len() + F::PAGE_SIZE - 1) / F::PAGE_SIZE;

        let mut pages = file.pages.lock();

        for _ in pages.len()..page_idx_max {
            pages.push(F::alloc_page(1));
        }

        let mut wsize = buffer.len();
        loop {
            let page_offset = offset % F::PAGE_SIZE;
            let curr_size = core::cmp::min(F::PAGE_SIZE - page_offset, wsize);
            if curr_size == 0 {
                break;
            }
            let page_data = pages[offset / F::PAGE_SIZE].get_buffer();
            page_data[page_offset..page_offset + curr_size]
                .copy_from_slice(&buffer[buffer_off..buffer_off + curr_size]);
            offset += curr_size;
            buffer_off += curr_size;
            wsize -= curr_size;
        }

        let file_size = *file.len.lock();
        if offset > file_size {
            *file.len.lock() = offset;
        }
        Ok(buffer.len())
    }

    fn truncate(&self, size: usize) -> FsResult<()> {
        // self.inner.content.lock().drain(size..);
        let file = match self {
            FileContainer::File(file) => file,
            FileContainer::Dir(_) => return Err(Errno::EISDIR),
            _ => return Err(Errno::EBADF),
        };
        // Chaneg file Size
        *file.len.lock() = size;

        let mut pages = file.pages.lock();
        let target_pages = (size + F::PAGE_SIZE - 1) / F::PAGE_SIZE;

        match target_pages <= pages.len() {
            // If target_pages is smaller than current page number.
            // Should drop pages that are bigger than target page number.
            // And fill 0 after the size.
            true => {
                pages.drain(target_pages..);

                if size % F::PAGE_SIZE != 0 {
                    let offset = size % F::PAGE_SIZE;
                    pages.last().unwrap().get_buffer()[offset..].fill(0);
                }
            }
            // If target_pages is bigger than current page number.
            // Allocate new pages to extend the size.
            false => {
                for _ in pages.len()..target_pages {
                    pages.push(F::alloc_page(1));
                }
            }
        }
        Ok(())
    }

    fn utimes(&self, times: &mut [TimeSpec]) -> FsResult<()> {
        let f_times = match self {
            FileContainer::File(file) => &file.times,
            FileContainer::Dir(_dir) => return Err(Errno::EISDIR),
            _ => return Err(Errno::EBADF),
        };
        if times[0].nsec != TimeSpec::UTIME_OMIT {
            f_times.lock()[1] = times[0];
        }
        if times[1].nsec != TimeSpec::UTIME_OMIT {
            f_times.lock()[2] = times[1];
        }
        // Ok(())
        todo!()
    }
}
