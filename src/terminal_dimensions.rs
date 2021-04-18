use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
use std::mem;

pub fn terminal_size() -> winsize {
    unsafe {
        let mut winsize = mem::zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut winsize);
        winsize
    }
}
