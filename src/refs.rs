use std::ops::Deref;
use std::pin::Pin;

#[derive(Debug)]
pub struct GCRef<T> {
    pub(crate) ptr: *const T,
}

impl<T> Clone for GCRef<T> {
    fn clone(&self) -> GCRef<T> {
        GCRef { ptr: self.ptr }
    }
}

impl<T> Deref for GCRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: It seems this doesn't count as moving the data out of the
        // pointer. The returned reference will be to the same area as the
        // pointer.
        unsafe { &*self.ptr }
    }
}

impl<T> From<GCRef<T>> for Pin<GCRef<T>> {
    fn from(refr: GCRef<T>) -> Pin<GCRef<T>> {
        // SAFETY: The data will remain on the heap until it is garbage
        // collected, at which point it's lifetime must be over.
        unsafe { Pin::new_unchecked(refr) }
    }
}
