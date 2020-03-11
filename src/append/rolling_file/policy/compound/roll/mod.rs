//! Rollers

use std::{error::Error, fmt, fs, io, path::Path};

#[cfg(feature = "background_rotation")]
use std::path::PathBuf;

#[cfg(feature = "file")]
use crate::file::Deserializable;

#[cfg(feature = "delete_roller")]
pub mod delete;
#[cfg(feature = "fixed_window_roller")]
pub mod fixed_window;
#[cfg(feature = "time_based_roller")]
pub mod time_based;

/// A trait which processes log files after they have been rolled over.
pub trait Roll: fmt::Debug + Send + Sync + 'static {
    /// Processes the log file.
    ///
    /// At the time that this method has been called, the log file has already
    /// been closed.
    ///
    /// If this method returns successfully, there *must* no longer be a file
    /// at the specified location.
    fn roll(&self, file: &Path) -> Result<(), Box<dyn Error + Sync + Send>>;
}

#[cfg(feature = "file")]
impl Deserializable for dyn Roll {
    fn name() -> &'static str {
        "roller"
    }
}

#[derive(Clone, Debug)]
enum Compression {
    None,
    #[cfg(feature = "gzip")]
    Gzip,
}

impl Compression {
    fn compress(&self, src: &Path, dst: &str) -> io::Result<()> {
        match *self {
            Compression::None => move_file(src, dst),
            #[cfg(feature = "gzip")]
            Compression::Gzip => {
                #[cfg(feature = "flate2")]
                use flate2::write::GzEncoder;
                use std::fs::File;

                let mut i = File::open(src)?;

                let o = File::create(dst)?;
                let mut o = GzEncoder::new(o, flate2::Compression::default());

                io::copy(&mut i, &mut o)?;
                drop(o.finish()?);
                drop(i); // needs to happen before remove_file call on Windows

                fs::remove_file(src)
            }
        }
    }
}

fn move_file<P, Q>(src: P, dst: Q) -> io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    // first try a rename
    match fs::rename(src.as_ref(), dst.as_ref()) {
        Ok(()) => return Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(_) => {}
    }

    // fall back to a copy and delete if src and dst are on different mounts
    fs::copy(src.as_ref(), dst.as_ref()).and_then(|_| fs::remove_file(src.as_ref()))
}

#[cfg(feature = "background_rotation")]
fn make_temp_file_name<P>(file: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let mut n = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs();
    let mut temp = file.as_ref().to_path_buf();
    temp.set_extension(format!("{}", n));
    while temp.exists() {
        n += 1;
        temp.set_extension(format!("{}", n));
    }
    temp
}
