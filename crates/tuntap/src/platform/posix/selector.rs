use std::{mem::MaybeUninit, time::Duration};

use crate::dev::Device;

pub struct Selector {
    nfds: i32,
    r_sets: MaybeUninit<libc::fd_set>,
}

impl Selector {
    pub fn new() -> Self {
        let mut r_sets = MaybeUninit::uninit();
        unsafe {
            libc::FD_ZERO(r_sets.as_mut_ptr());
        }
        Self {
            nfds: 0,
            r_sets,
        }
    }

    /// registor the tap fd in to select fd bit set.
    pub fn register(&mut self, tap: &impl Device) {
        let fd = tap.fd().0;
        unsafe {
            if self.nfds <= fd + 1 {
                self.nfds = fd + 1
            }
            libc::FD_SET(fd, self.r_sets.as_mut_ptr());
        }
    }

    /// unregistor the tap fd.
    pub fn unregister(&mut self, tap: &impl Device) {
        let fd = tap.fd().0;
        unsafe {
            libc::FD_CLR(fd, self.r_sets.as_mut_ptr());
        }
    }

    pub fn poll(&mut self, t: Duration) -> i32 {
        let mut timeout: MaybeUninit<libc::timeval> = MaybeUninit::zeroed();
        let mircos = t.as_micros();
        unsafe {
            (&mut *timeout.as_mut_ptr()).tv_usec = (mircos%1000_000) as _;
            (&mut *timeout.as_mut_ptr()).tv_sec = (mircos/1000_000) as _;
        }
        // inital the from r_sets, the select system call will change the context in r_sets.
        let mut r_sets = self.r_sets.clone();
        unsafe {
            libc::select(
                self.nfds, 
                r_sets.as_mut_ptr(), 
                std::ptr::null_mut(), 
                std::ptr::null_mut(), 
                timeout.as_mut_ptr()
            )
        }
    }
}