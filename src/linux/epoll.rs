//! An OS thread for catching file descriptor changes.

use std::os::raw;
use std::mem;
use std::task;
use std::ptr;
use std::thread;
use std::sync::atomic;

type Ptr = Option<task::Waker>;

const EPOLLIN: u32 = 0x001;
const EPOLLOUT: u32 = 0x004;

const EPOLL_CTL_ADD: i32 = 1;
const EPOLL_CTL_DEL: i32 = 2;

#[repr(C)]
union EpollData {
   ptr: *mut raw::c_void,
   fd: raw::c_int,
   uint32: u32,
   uint64: u64,
}

#[repr(C)]
struct EpollEvent {
    events: u32,        /* Epoll events */
    data: EpollData,    /* User data variable */
}

extern "C" {
    fn epoll_create1(flags: raw::c_int) -> raw::c_int;
    fn close(fd: raw::c_int) -> raw::c_int;
    fn epoll_ctl(epfd: raw::c_int, op: raw::c_int, fd: raw::c_int,
        event: *mut EpollEvent) -> raw::c_int;
    fn epoll_wait(epfd: raw::c_int, events: *mut EpollEvent,
        maxevents: raw::c_int, timeout: raw::c_int) -> raw::c_int;
}

static mut EPOLL: mem::MaybeUninit<Epoll> = mem::MaybeUninit::uninit();
static mut ONCE: std::sync::Once = std::sync::Once::new();
static RUNNING: atomic::AtomicBool = atomic::AtomicBool::new(true);

/// Start the Epoll Thread, if not already started, and return it.
pub(super) fn start() -> Epoll {
    unsafe {
    ONCE.call_once(|| {
        let epoll = Epoll::new().unwrap();
        EPOLL = mem::MaybeUninit::new(epoll.clone());
        // Spawn and detach the thread.
        thread::spawn(move || {
            while RUNNING.load(atomic::Ordering::SeqCst) {
                let mut ev = mem::MaybeUninit::<EpollEvent>::uninit();
                // Wait for something to happen.
                if epoll_wait(epoll.fd, ev.as_mut_ptr(), 1 /* Get one event at a  time */, -1 /* wait indefinitely */) < 0 {
                    // Ignore error
                    continue;
                }
                // Wake waiting thread if it's waiting.
                if let Some(waker) = (*ev.assume_init().data.ptr.cast::<Ptr>()).take() {
                    waker.wake();
                }
            }
            epoll.free();
        });
    });
    }

    unsafe { EPOLL.assume_init() }
}

/// An Epoll handle.
#[derive(Copy, Clone)]
pub(super) struct Epoll {
    // File descriptor for this epoll instance.
    fd: raw::c_int,
}

impl Epoll {
    /// Create a new epoll instance.
    fn new() -> Result<Self, ()> {
        let fd = unsafe {
            epoll_create1(0 /* no flags */)
        };
        Self::error(fd)?;
        Ok(Epoll { fd })
    }

    /// Add file descriptor to epoll
    pub fn add(&mut self, fd: raw::c_int, waker: task::Waker) -> Result<(), ()> {
        let ptr: Box<Ptr> = Box::new(Some(waker));

        self.ctl(fd, EPOLL_CTL_ADD, Box::into_raw(ptr).cast())
    }

    /// Remove file descriptor from epoll
    pub fn del(&mut self, fd: raw::c_int) -> Result<(), ()> {
        self.ctl(fd, EPOLL_CTL_DEL, ptr::null_mut())
    }

    /// Exit the thread.
    pub fn exit(&mut self) {
        RUNNING.store(false, atomic::Ordering::SeqCst);
    }

    // Add, delete or modify this epoll instance.
    fn ctl(&mut self, op: raw::c_int, fd: raw::c_int, ptr: *mut raw::c_void) -> Result<(), ()> {
        let ret = unsafe {
            epoll_ctl(self.fd, op, fd, &mut EpollEvent {
                events: EPOLLIN | EPOLLOUT,
                data: EpollData { ptr },
            })
        };
        Self::error(ret)
    }

    // Convert a C error (negative on error) into a result.
    fn error(err: raw::c_int) -> Result<(), ()> {
        if err < 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    // Free the Epoll instance
    fn free(self) {
        // close() should never fail.
        let ret = unsafe {
            close(self.fd)
        };
        Self::error(ret).unwrap();
    }
}
