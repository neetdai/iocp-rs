use windows_sys::Win32::System::IO::OVERLAPPED;

pub struct Context {
    over_lapped_ptr: *mut OVERLAPPED,
}

impl Context {
    fn new(over_lapped_ptr: *mut OVERLAPPED) -> Self {
        Self { over_lapped_ptr }
    }

    fn set_offset(&mut self, offset: u64) {
        let low_offset = (offset & (u32::MAX as u64)) as u32;
        let high_offset = (offset >> 32) as u32;

        let mut over_lapped = unsafe { &mut (*self.over_lapped_ptr) };
        over_lapped.Anonymous.Anonymous.Offset = low_offset;
        over_lapped.Anonymous.Anonymous.OffsetHigh = high_offset;
    }

    pub(crate) fn over_lapped_ptr(&self) -> *mut OVERLAPPED {
        self.over_lapped_ptr
    }
}
