pub trait AsHandle {
    type Handle;

    fn as_handle(&self) -> Self::Handle;
}
