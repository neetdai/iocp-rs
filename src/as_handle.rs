use windows_sys::Win32::Foundation::HANDLE;

pub trait AsHandle {
    fn as_handle(&self) -> HANDLE;
}
