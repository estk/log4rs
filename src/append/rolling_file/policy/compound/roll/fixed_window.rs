//! The fixed-window roller.
//!
//! Requires the `fixed_window_roller` feature.

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use append::rolling_file::policy::compound::roll::Roll;
#[cfg(feature = "file")]
use file::{Deserialize, Deserializers};

/// Configuration for the fixed window roller.
#[cfg(feature = "file")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedWindowRollerConfig {
    pattern: String,
    base: Option<u32>,
    count: u32,
}

#[derive(Debug)]
enum Compression {
    None,
    #[cfg(feature = "gzip")] Gzip,
}

impl Compression {
    fn compress(&self, src: &Path, dst: &str) -> io::Result<()> {
        match *self {
            Compression::None => move_file(src, dst),
            #[cfg(feature = "gzip")]
            Compression::Gzip => {
                use flate2;
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

/// A roller which maintains a fixed window of archived log files.
///
/// A `FixedWindowRoller` is configured with a filename pattern, a base index,
/// and a maximum file count. Each archived log file is associated with a numeric
/// index ordering it by age, starting at the base index. Archived log files are
/// named by substituting all instances of `{}` with the file's index in the
/// filename pattern.
///
/// For example, if the filename pattern is `archive/foo.{}.log`, the base index
/// is 0 and the count is 2, the first log file will be archived as
/// `archive/foo.0.log`. When the next log file is archived, `archive/foo.0.log`
/// will be renamed to `archive/foo.1.log` and the new log file will be named
/// `archive/foo.0.log`. When the third log file is archived,
/// `archive/foo.1.log` will be deleted, `archive/foo.0.log` will be renamed to
/// `archive/foo.1.log`, and the new log file will be renamed to
/// `archive/foo.0.log`.
///
/// If the file extension of the pattern is `.gz` and the `gzip` Cargo feature
/// is enabled, the archive files will be gzip-compressed.
///
/// Note that this roller will have to rename every archived file every time the
/// log rolls over. Performance may be negatively impacted by specifying a large
/// count.
#[derive(Debug)]
pub struct FixedWindowRoller {
    pattern: String,
    compression: Compression,
    base: u32,
    count: u32,
}

impl FixedWindowRoller {
    /// Returns a new builder for the `FixedWindowRoller`.
    pub fn builder() -> FixedWindowRollerBuilder {
        FixedWindowRollerBuilder { base: 0 }
    }
}

impl Roll for FixedWindowRoller {
    fn roll(&self, file: &Path) -> Result<(), Box<Error + Sync + Send>> {
        if self.count == 0 {
            return fs::remove_file(file).map_err(Into::into);
        }

        let dst_0 = self.pattern.replace("{}", &self.base.to_string());

        if let Some(parent) = Path::new(&dst_0).parent() {
            fs::create_dir_all(parent)?;
        }

        // In the common case, all of the archived files will be in the same
        // directory, so avoid extra filesystem calls in that case.
        let parent_varies = match (
            Path::new(&dst_0).parent(),
            Path::new(&self.pattern).parent(),
        ) {
            (Some(a), Some(b)) => a != b,
            _ => false, // Only case that can actually happen is (None, None)
        };

        for i in (self.base..self.base + self.count - 1).rev() {
            let src = self.pattern.replace("{}", &i.to_string());
            let dst = self.pattern.replace("{}", &(i + 1).to_string());

            if parent_varies {
                if let Some(parent) = Path::new(&dst).parent() {
                    fs::create_dir_all(parent)?;
                }
            }

            move_file(&src, &dst)?;
        }

        self.compression.compress(file, &dst_0).map_err(Into::into)
    }
}

fn move_file<P>(src: P, dst: &str) -> io::Result<()>
where
    P: AsRef<Path>,
{
    // first try a rename
    match fs::rename(src.as_ref(), dst) {
        Ok(()) => return Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(_) => {}
    }

    // fall back to a copy and delete if src and dst are on different mounts
    fs::copy(src.as_ref(), dst).and_then(|_| fs::remove_file(src.as_ref()))
}

/// A builder for the `FixedWindowRoller`.
pub struct FixedWindowRollerBuilder {
    base: u32,
}

impl FixedWindowRollerBuilder {
    /// Sets the base index for archived log files.
    ///
    /// Defaults to 0.
    pub fn base(mut self, base: u32) -> FixedWindowRollerBuilder {
        self.base = base;
        self
    }

    /// Constructs a new `FixedWindowRoller`.
    ///
    /// `pattern` must contain at least one instance of `{}`, all of which will
    /// be replaced with an archived log file's index.
    ///
    /// If the file extension of the pattern is `.gz` and the `gzip` Cargo
    /// feature is enabled, the archive files will be gzip-compressed.
    pub fn build(
        self,
        pattern: &str,
        count: u32,
    ) -> Result<FixedWindowRoller, Box<Error + Sync + Send>> {
        if !pattern.contains("{}") {
            return Err("pattern does not contain `{}`".into());
        }

        let compression = match Path::new(pattern).extension() {
            #[cfg(feature = "gzip")]
            Some(e) if e == "gz" =>
            {
                Compression::Gzip
            }
            #[cfg(not(feature = "gzip"))]
            Some(e) if e == "gz" =>
            {
                return Err("gzip compression requires the `gzip` feature".into());
            }
            _ => Compression::None,
        };

        Ok(FixedWindowRoller {
            pattern: pattern.to_owned(),
            compression: compression,
            base: self.base,
            count: count,
        })
    }
}

/// A deserializer for the `FixedWindowRoller`.
///
/// # Configuration
///
/// ```yaml
/// kind: fixed_window
///
/// # The filename pattern for archived logs. Must contain at least one `{}`.
/// # If the file extension of the pattern is `.gz` and the `gzip` Cargo feature
/// # is enabled, the archive files will be gzip-compressed.
/// # Required.
/// pattern: archive/foo.{}.log
///
/// # The maximum number of archived logs to maintain. Required.
/// count: 5
///
/// # The base value for archived log indices. Defaults to 0.
/// base: 1
/// ```
#[cfg(feature = "file")]
pub struct FixedWindowRollerDeserializer;

#[cfg(feature = "file")]
impl Deserialize for FixedWindowRollerDeserializer {
    type Trait = Roll;

    type Config = FixedWindowRollerConfig;

    fn deserialize(
        &self,
        config: FixedWindowRollerConfig,
        _: &Deserializers,
    ) -> Result<Box<Roll>, Box<Error + Sync + Send>> {
        let mut builder = FixedWindowRoller::builder();
        if let Some(base) = config.base {
            builder = builder.base(base);
        }

        Ok(Box::new(builder.build(&config.pattern, config.count)?))
    }
}

#[cfg(test)]
mod test {
    use tempdir::TempDir;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::process::Command;

    use append::rolling_file::policy::compound::roll::Roll;
    use super::*;

    #[test]
    fn rotation() {
        let dir = TempDir::new("rotation").unwrap();

        let base = dir.path().to_str().unwrap();
        let roller = FixedWindowRoller::builder()
            .build(&format!("{}/foo.log.{{}}", base), 2)
            .unwrap();

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file1").unwrap();

        roller.roll(&file).unwrap();
        assert!(!file.exists());
        let mut contents = vec![];
        File::open(dir.path().join("foo.log.0"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"file1");

        File::create(&file).unwrap().write_all(b"file2").unwrap();

        roller.roll(&file).unwrap();
        assert!(!file.exists());
        contents.clear();
        File::open(dir.path().join("foo.log.1"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"file1");
        contents.clear();
        File::open(dir.path().join("foo.log.0"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"file2");

        File::create(&file).unwrap().write_all(b"file3").unwrap();

        roller.roll(&file).unwrap();
        assert!(!file.exists());
        contents.clear();
        assert!(!dir.path().join("foo.log.2").exists());
        File::open(dir.path().join("foo.log.1"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"file2");
        contents.clear();
        File::open(dir.path().join("foo.log.0"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"file3");
    }

    #[test]
    fn rotation_no_trivial_base() {
        let dir = TempDir::new("rotation_no_trivial_base").unwrap();
        let base = 3;
        let fname = "foo.log";
        let fcontent = b"something";
        let expected_fist_roll = format!("{}.{}", fname, base);

        let base_dir = dir.path().to_str().unwrap();
        let roller = FixedWindowRoller::builder()
            .base(base)
            .build(&format!("{}/{}.{{}}", base_dir, fname), 2)
            .unwrap();

        let file = dir.path().join(fname);
        File::create(&file).unwrap().write_all(fcontent).unwrap();

        roller.roll(&file).unwrap();
        assert!(!file.exists());

        let mut contents = vec![];

        let first_roll = dir.path().join(&expected_fist_roll);

        assert!(first_roll.as_path().exists());

        File::open(first_roll)
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, fcontent);

        // Sanity check general behaviour
        roller.roll(&file).unwrap();
        assert!(!file.exists());
        contents.clear();
        File::open(dir.path().join(&format!("{}.{}", fname, base + 1)))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"something");
    }

    #[test]
    fn create_archive_unvaried() {
        let dir = TempDir::new("create_archive_unvaried").unwrap();

        let base = dir.path().join("log").join("archive");
        let pattern = base.join("foo.{}.log");
        let roller = FixedWindowRoller::builder()
            .build(pattern.to_str().unwrap(), 2)
            .unwrap();

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file").unwrap();

        roller.roll(&file).unwrap();

        assert!(base.join("foo.0.log").exists());

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file2").unwrap();

        roller.roll(&file).unwrap();

        assert!(base.join("foo.0.log").exists());
        assert!(base.join("foo.1.log").exists());
    }

    #[test]
    fn create_archive_varied() {
        let dir = TempDir::new("create_archive_unvaried").unwrap();

        let base = dir.path().join("log").join("archive");
        let pattern = base.join("{}").join("foo.log");
        let roller = FixedWindowRoller::builder()
            .build(pattern.to_str().unwrap(), 2)
            .unwrap();

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file").unwrap();

        roller.roll(&file).unwrap();

        assert!(base.join("0").join("foo.log").exists());

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file2").unwrap();

        roller.roll(&file).unwrap();

        assert!(base.join("0").join("foo.log").exists());
        assert!(base.join("1").join("foo.log").exists());
    }

    #[test]
    #[cfg_attr(feature = "gzip", ignore)]
    fn unsupported_gzip() {
        let dir = TempDir::new("unsupported_gzip").unwrap();

        let pattern = dir.path().join("{}.gz");
        assert!(
            FixedWindowRoller::builder()
                .build(pattern.to_str().unwrap(), 2)
                .is_err()
        );
    }

    #[test]
    #[cfg_attr(not(feature = "gzip"), ignore)]
    // or should we force windows user to install gunzip
    #[cfg(not(windows))]
    fn supported_gzip() {
        let dir = TempDir::new("supported_gzip").unwrap();

        let pattern = dir.path().join("{}.gz");
        let roller = FixedWindowRoller::builder()
            .build(pattern.to_str().unwrap(), 2)
            .unwrap();

        let contents = (0..10000).map(|i| i as u8).collect::<Vec<_>>();

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(&contents).unwrap();

        roller.roll(&file).unwrap();

        assert!(
            Command::new("gunzip")
                .arg(dir.path().join("0.gz"))
                .status()
                .unwrap()
                .success()
        );

        let mut file = File::open(dir.path().join("0")).unwrap();
        let mut actual = vec![];
        file.read_to_end(&mut actual).unwrap();

        assert_eq!(contents, actual);
    }
}
