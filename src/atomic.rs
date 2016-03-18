use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::mem;
use std::marker::PhantomData;

pub struct AtomicReference<T>(AtomicUsize, PhantomData<Arc<T>>);

impl<T> Drop for AtomicReference<T> {
    fn drop(&mut self) {
        let _: Arc<T> = unsafe { mem::transmute(self.0.load(Ordering::SeqCst)) };
    }
}

impl<T> AtomicReference<T> {
    pub fn new(t: Arc<T>) -> AtomicReference<T> {
        unsafe { AtomicReference(AtomicUsize::new(mem::transmute(t)), PhantomData) }
    }

    fn take(&self) -> usize {
        loop {
            match self.0.swap(0, Ordering::SeqCst) {
                0 => {}
                r => return r,
            }
        }
    }

    fn put(&self, ptr: usize) {
        debug_assert_eq!(0, self.0.load(Ordering::SeqCst));
        self.0.store(ptr, Ordering::SeqCst);
    }

    pub fn set(&self, t: Arc<T>) -> Arc<T> {
        let raw = self.take();
        self.put(unsafe { mem::transmute(t) });
        unsafe { mem::transmute(raw) }
    }

    pub fn get(&self) -> Arc<T> {
        let raw = self.take();
        let ret = unsafe { (*(&raw as *const _ as *const Arc<T>)).clone() };
        self.put(raw);
        ret
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn basic() {
        let r = AtomicReference::new(Arc::new(0));
        assert_eq!(*r.get(), 0);
        assert_eq!(*r.set(Arc::new(1)), 0);
        assert_eq!(*r.get(), 1);
    }
}
