#![cfg_attr(not(feature = "std"), no_std)]
use core::str;
use libc::{self, c_int, size_t, strerror_r, strlen};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Errno(c_int);

impl Errno {
    // Add any necessary methods or implementations for the Errno type
}

fn from_utf8_lossy(input: &[u8]) -> &str {
    match str::from_utf8(input) {
        Ok(valid) => valid,
        Err(error) => unsafe { str::from_utf8_unchecked(&input[..error.valid_up_to()]) },
    }
}

pub fn with_description<F, T>(err: Errno, callback: F) -> T
where
    F: FnOnce(Result<&str, Errno>) -> T,
{
    let mut buf = [0u8; 1024];
    let c_str = unsafe {
        let rc = strerror_r(err.0, buf.as_mut_ptr() as *mut _, buf.len() as size_t);
        if rc != 0 {
            let fm_err = match rc < 0 {
                true => errno(),
                false => Errno(rc),
            };
            if fm_err != Errno(libc::ERANGE) {
                return callback(Err(fm_err));
            }
        }
        let c_str_len = strlen(buf.as_ptr() as *const _);
        &buf[..c_str_len]
    };
    callback(Ok(from_utf8_lossy(c_str)))
}

pub const STRERROR_NAME: &str = "strerror_r";

pub fn errno() -> Errno {
    unsafe { Errno(*errno_location()) }
}

pub fn set_errno(Errno(errno): Errno) {
    unsafe {
        *errno_location() = errno;
    }
}

extern "C" {
    #[cfg_attr(
        any(
            target_os = "macos",
            target_os = "ios",
            target_os = "tvos",
            target_os = "watchos",
            target_os = "visionos",
            target_os = "freebsd"
        ),
        link_name = "__error"
    )]
    #[cfg_attr(
        any(
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "bitrig",
            target_os = "android",
            target_os = "espidf",
            target_env = "newlib"
        ),
        link_name = "__errno"
    )]
    #[cfg_attr(
        any(target_os = "solaris", target_os = "illumos"),
        link_name = "___errno"
    )]
    #[cfg_attr(target_os = "haiku", link_name = "_errnop")]
    #[cfg_attr(
        any(
            target_os = "linux",
            target_os = "hurd",
            target_os = "redox",
            target_os = "dragonfly"
        ),
        link_name = "__errno_location"
    )]
    #[cfg_attr(target_os = "aix", link_name = "_Errno")]
    #[cfg_attr(target_os = "nto", link_name = "__get_errno_ptr")]
    fn errno_location() -> *mut c_int;
}
