#[cfg(feature = "parking_lot")]
pub(crate) use parking_lot::{Mutex, MutexGuard};
#[cfg(not(feature = "parking_lot"))]
pub(crate) use std::sync::{Mutex, MutexGuard};

pub(crate) fn lock<T: ?Sized>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    #[cfg(feature = "parking_lot")]
    {
        mutex.lock()
    }
    #[cfg(not(feature = "parking_lot"))]
    {
        // Match parking_lot: a caught panic must not disable future logging.
        mutex
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
