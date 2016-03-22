use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::mem;
use std::marker::PhantomData;

pub struct AtomicReference<T>(AtomicUsize, PhantomData<Arc<T>>);

impl<T> Drop for AtomicReference<T> {
    fn drop(&mut self) {
        self.take();
    }
}

impl<T> AtomicReference<T> {
    pub fn new(t: Arc<T>) -> AtomicReference<T> {
        unsafe { AtomicReference(AtomicUsize::new(mem::transmute(t)), PhantomData) }
    }

    fn take(&self) -> Arc<T> {
        loop {
            match self.0.swap(0, Ordering::SeqCst) {
                0 => {}
                r => return unsafe { mem::transmute(r) },
            }
        }
    }

    fn put(&self, t: Arc<T>) {
        debug_assert_eq!(0, self.0.load(Ordering::SeqCst));
        self.0.store(unsafe { mem::transmute(t) }, Ordering::SeqCst);
    }

    pub fn set(&self, t: Arc<T>) -> Arc<T> {
        let old = self.take();
        self.put(t);
        old
    }

    pub fn get(&self) -> Arc<T> {
        let t = self.take();
        self.put(t.clone());
        t
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
