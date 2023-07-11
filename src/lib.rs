// use std::os::unix::io::{AsRawFd, RawFd};

// use libc;

// fn splice(fd_in: RawFd, fd_out: RawFd, size: usize) -> isize {
//     unsafe {
//         libc::splice(
//             fd_in,
//             std::ptr::null_mut::<libc::loff_t>(),
//             fd_out,
//             std::ptr::null_mut::<libc::loff_t>(),
//             size,
//             libc::SPLICE_F_NONBLOCK | libc::SPLICE_F_MORE | libc::SPLICE_F_MOVE,
//         )
//     }
// }

// enum TransferState {
//     Running(u64),
//     ShuttingDown(u64),
//     Done(u64),
// }

// pub async fn splice_bidirectional<A, B>(a: &mut A, b: &mut B) -> Result<(u64, u64), std::io::Error>
// where
//   A: AsyncRead + AsyncWrite + Unpin + ?Sized,
//   B: AsyncRead + AsyncWrite + Unpin + ?Sized,
// {
//   let mut a_to_b = TransferState::Running(0);
//   let mut b_to_a = TransferState::Running(0);
//   poll_fn(|cx| {
//     let a_to_b = transfer_
