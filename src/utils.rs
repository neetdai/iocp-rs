use std::{
    io::{Error, Result},
    time::Duration,
};
use windows_sys::Win32::System::Threading::INFINITE;

pub(crate) fn cvt(ret: i32) -> Result<i32> {
    if ret == 0 {
        Err(Error::last_os_error())
    } else {
        Ok(ret)
    }
}

pub(crate) fn dur_to_ms(timeout: Option<Duration>) -> u32 {
    let func = |dur: Duration| -> u32 {
        dur.as_secs()
            .checked_mul(1000)
            .and_then(|ms| ms.checked_add((dur.subsec_nanos() as u64) / 1_000_000))
            .and_then(|ms| {
                ms.checked_add(if dur.subsec_nanos() % 1_000_000 > 0 {
                    1
                } else {
                    0
                })
            })
            .map(|ms| {
                if ms > u32::max_value() as u64 {
                    INFINITE
                } else {
                    ms as u32
                }
            })
            .unwrap_or(INFINITE)
    };

    timeout.map(func).unwrap_or(INFINITE)
}

pub(crate) fn len<T>(list: &[T]) -> u32 {
    list.len().min(u32::MAX as usize) as u32
}
