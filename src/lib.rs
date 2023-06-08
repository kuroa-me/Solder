use std::os::unix::io::{AsRawFd, RawFd};

use libc;

fn splice(fd_in: RawFd, fd_out: RawFd, size: usize) -> isize {
    unsafe {
        libc::splice(
            fd_in,
            std::ptr::null_mut::<libc::loff_t>(),
            fd_out,
            std::ptr::null_mut::<libc::loff_t>(),
            size,
            libc::SPLICE_F_NONBLOCK | libc::SPLICE_F_MORE | libc::SPLICE_F_MOVE,
        )
    }
}
