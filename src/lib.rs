//!Linux Kernel 64 Bindings
//!
//!These bindings are exposed as an _extended_ interface to the Linux
//!kernel which are not neccesarily covered in the `libc` crate. 
//!
//!The bindings here are not _SAFE_ in the strict sense of Rust. They
//!require you understand the kernel interface. I will attempt to provide
//!explainations, and links to the relevant man pages.
//!
//!This bindings are ONLY valid for Linux AMD64. If you attempt to use
//!them on other platforms this will break.
use std::io;
use std::mem;
use std::os::unix::io::AsRawFd;
use std::fs::File;

///So I don't have to type out io::Error all the time.
pub type OSFault = io::Error;

extern {
    fn simple_rand(ptr: *mut u8, len: usize) -> u64;
    fn simple_urand(ptr: *mut u8, len: usize) -> u64;
    fn cow(ptr: *mut u8, len: usize) -> u64;
    fn memmap_ro(fd: i32, len: u64) -> u64;
}

/// Random
///
/// This is a high level interface to the `getrandom` call. Which allows for
/// developers to read either `/dev/random` or `/dev/urandom`. The goal is
/// to avoid using file descriptors to read system randomness. This reduces
/// system load.
///
///Here is the [MANUAL](http://man7.org/linux/man-pages/man2/getrandom.2.html)
///
/// You should use URand. Both kernel buffers DevRand, and URand go though
/// the same identical whitening process, and read from the same entropy
/// poll. The only difference is DevRand does not actively re-populate
/// itself. The kernel governs this, so it can block for long periods of
/// time.
///
/// #/dev/urandom is BAD THO
///
/// If somebody told you `/dev/random` is more secure read [this
/// link](http://www.2uo.de/myths-about-urandom/). It includes a number
/// of links as well as a discussions by crypto and kernel experts. As well
/// as easy to understand diagrams of the post 4.8Linux Kernel Random Pool.
/// 
/// #TL;DR
/// * They both read from the same pool.
/// * They both have the same source.
/// * They both use the same CSPRNG.
/// * They both use the same whitening.
/// * True system randomness is for academic masturbation.
///
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Rand {
    DevRand,
    URand
}
impl Rand {
    ///Fetch Randomness
    ///
    ///This will make the call to `getrandom`. It also provides clean up
    ///and tear down of the unsafe and FFI interfaces.
    pub fn fetch(&self,data: &mut [u8]) -> Result<(),OSFault> {
        unsafe{
            let (ptr,len): (*mut u8,usize) = mem::transmute(data);
            let flag = match *self {
                Rand::URand => simple_urand(ptr,len),
                Rand::DevRand => simple_rand(ptr,len)
            };
            if flag == 0 {
                Ok(())
            } else {
                Err(OSFault::last_os_error())
            }
        }
    }
}
impl Default for Rand {
    ///Default
    ///
    ///Will return the URand option. See the header text as to why.
    fn default() -> Self {
        Rand::URand
    }
}

///Copy On Write
///
///This creates a copy on write allocation, tied to the memory lifetime
///of the pre-exising allocation.
///
///COW in a nutshell allocates a new virtual memory space, that points
///to the old virtual memory space. When you write/modify in the new space
///the kernel copies the old data into a new page, and quitely updates your
///mappings.
///
///Effectively this allows you to version a slice. 
///
///#NOTES
/// * Don't pass this a vector, if it resizes you can get a segfault
/// * You WILL have to do some manual memory management.
/// * Be Careful
/// * mmap cares about alignment, so a too long/short array may trigger an
/// error
pub fn copy_on_write<'a,T>(buf: &'a [T])
-> Result<&'a mut[T],OSFault>
where T: Sized {
    unsafe {
        let sized = mem::size_of::<T>();
        let (ptr,len): (*mut u8,usize) = mem::transmute(buf);
        let alloc_len = len*sized;
        let flag = cow(ptr,alloc_len);
        if flag == 0 {
            let tup: (u64,usize) = (flag,len);
            Ok(mem::transmute(tup))
        } else {
            Err(OSFault::last_os_error())
        }
    }
}

///Memory Map
///
///Load a file into virtual memory. Memory Mapping by-passes the normal
///read/write paradigm. Instead you _map_ the file as an array in RAM.
///
///The entire file isn't loaded into memory, only a few small bits around
///where you are currently reading/writing. The kernel does book keeping
///to create the illusion that file is fully loaded.
///
///This method is over slower then loading the entire file into RAM if you
///need to skip around a TON and visit every nook and cranny unpredictably.
///If you are reading the file in-order or re-reading the same spot
///multiple times this is an efficient method.
pub fn read_only_memmap<'a>( file: &File)
-> Result<&'a [u8],OSFault>
{
    unsafe{
        let metadata = file.metadata()?;
        let len = metadata.len();
        let fd = file.as_raw_fd();
        let val = memmap_ro(fd,len);
        if val == 0 {
            let tup: (u64,u64) = (val,len);
            Ok(mem::transmute(tup))
        } else {
            Err(OSFault::last_os_error())
        }
    }
}
