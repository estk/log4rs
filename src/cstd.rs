#[cfg(feature = "async-std")]
pub use async_std::{fs, io};
#[cfg(not(feature = "async-std"))]
pub use std::{fs, io};
