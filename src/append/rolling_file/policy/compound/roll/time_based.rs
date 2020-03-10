//! The time based roller.
//!
//! Requires the `time_based_roller` feature.

#[cfg(feature = "background_rotation")]
use parking_lot::Mutex;
#[cfg(feature = "file")]
use serde_derive::Deserialize;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
#[cfg(feature = "background_rotation")]
use std::sync::Arc;

use chrono::{NaiveDate, NaiveDateTime};

use crate::append::rolling_file::policy::compound::roll::Roll;
use crate::append::rolling_file::policy::compound::now_string;
#[cfg(feature = "file")]
use crate::file::{Deserialize, Deserializers};

/// Configuration for the time based window roller.
#[cfg(feature = "file")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeBasedRollerConfig {
    pattern: String,
    count: u32,
    fmt: String,
    scale: String,
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

/// A roller which maintains a time based of archived log files.
///
/// A `TimeBasedRoller` is configured with a filename pattern, a time format,
/// and a maximum file count. Each archived log file is associated with a time
/// string ordering it by age, starting at the current time. Archived log files are
/// named by substituting all instances of `{}` with the current time in the
/// filename pattern.
///
/// If the file extension of the pattern is `.gz` and the `gzip` Cargo feature
/// is enabled, the archive files will be gzip-compressed.
///
/// Note that this roller will have to rename every archived file every time the
/// log rolls over. Performance may be negatively impacted by specifying a large
/// count.
#[derive(Debug)]
pub struct TimeBasedRoller {
    pattern: String,
    fmt: String,
    scale: String,
    compression: Compression,
    count: u32,
    #[cfg(feature = "background_rotation")]
    lock: Arc<Mutex<()>>,
    mock_time: Option<String>,
}

impl TimeBasedRoller {
    /// Returns a new builder for the `TimeBasedRoller`.
    pub fn builder() -> TimeBasedRollerBuilder {
        TimeBasedRollerBuilder {}
    }
}

impl Roll for TimeBasedRoller {
    #[cfg(not(feature = "background_rotation"))]
    fn roll(&self, file: &Path) -> Result<(), Box<dyn Error + Sync + Send>> {
        if self.count == 0 {
            return fs::remove_file(file).map_err(Into::into);
        }

        rotate(
            self.pattern.clone(),
            self.compression.clone(),
            self.count as usize,
            self.fmt.clone(),
            now_string(&self.fmt),
            self.scale.clone(),
            file.to_path_buf(),
        )?;

        Ok(())
    }

    #[cfg(feature = "background_rotation")]
    fn roll(&self, file: &Path) -> Result<(), Box<dyn Error + Sync + Send>> {
        if self.count == 0 {
            return fs::remove_file(file).map_err(Into::into);
        }

        // rename the file
        let temp = make_temp_file_name(file);
        move_file(file, &temp)?;

        {
            // wait for the previous call to end
            let _lock = self.lock.lock();
        }

        let pattern = self.pattern.clone();
        let compression = self.compression.clone();
        let count = self.count;
        let lock = Arc::clone(&self.lock);
        let scale = self.scale.clone();
        let fmt = self.fmt.clone();
        let now_string = now_string(&self.fmt);
        // rotate in the separate thread
        std::thread::spawn(move || {
            let _lock = lock.lock();
            if let Err(e) = rotate(
                pattern,
                compression,
                count as usize,
                fmt,
                now_string,
                scale,
                temp,
            ) {
                use std::io::Write;
                let _ = writeln!(io::stderr(), "log4rs: {}", e);
            }
        });

        Ok(())
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

fn rotate(
    pattern: String,
    compression: Compression,
    count: usize,
    fmt: String,
    time_string: String,
    scale: String,
    file: PathBuf,
) -> io::Result<()> {
    let dst_0 = pattern.replace("{}", &time_string);

    if let Some(parent) = Path::new(&dst_0).parent() {
        fs::create_dir_all(parent)?;
    }

    // In the common case, all of the archived files will be in the same
    // directory, so avoid extra filesystem calls in that case.
    // for example: dst_0 "/some_dir/2020-03-08/some.log" pattern "/some_dir/{}/some.log")
    let parent_varies = match (Path::new(&dst_0).parent(), Path::new(&pattern).parent()) {
        (Some(a), Some(b)) => a != b,
        _ => false, // Only case that can actually happen is (None, None)
    };

    let _ = rm_outdated_pattern_files(&dst_0, pattern, fmt, scale, count, parent_varies)?;

    compression.compress(&file, &dst_0)?;
    Ok(())
}

fn rm_outdated_pattern_files(
    dst_0: &str,
    pattern: String,
    format: String,
    scale: String,
    count: usize,
    parent_varies: bool,
) -> io::Result<()> {
    let parent;
    let (parrent_opt, fmt) =
        get_pattern_files_parent_and_fmt(parent_varies, &dst_0, &pattern, &format);
    if let Some(p) = parrent_opt {
        parent = Path::new(p);
    } else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, dst_0));
    }

    let mut entries;
    entries = fs::read_dir(parent)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    entries = entries
        .into_iter()
        .filter(|p| {
            if let Some(p) = p.to_str() {
                if scale.to_lowercase() == "date" {
                    let e = NaiveDate::parse_from_str(p, &fmt).is_ok();
                    e
                } else {
                    NaiveDateTime::parse_from_str(p, &fmt).is_ok()
                }
            } else {
                false
            }
        })
        .collect::<Vec<_>>();

    entries.sort();
    entries.reverse();

    while entries.len() >= count {
        if let Some(oldest_entry) = entries.pop() {
            let path = Path::new(&oldest_entry);
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
    }
    Ok(())
}

fn get_pattern_files_parent_and_fmt<'a>(
    parent_varies: bool,
    dst_0: &'a str,
    pattern: &'a str,
    format: &str,
) -> (Option<&'a str>, String) {
    if parent_varies {
        // case 1: split /some_dir/{}/log -> ["/some_dir", "/log"]
        // case 2: split {}/log -> ["", "/log"]
        let split_vec = pattern.split("{}").collect::<Vec<_>>();
        let prefix = split_vec[0];
        let fmt = format!("{}{}", prefix, format);
        if prefix == "" {
            (Some("."), fmt)
        } else {
            (Some(prefix), fmt)
        }
    } else {
        // case 3: split /some_dir/some.{}.log -> ["/some_dir/log", ".log"]
        let fmt = pattern.replace("{}", &format);
        if let Some(p) = Path::new(dst_0).parent() {
            (p.to_str(), fmt)
        } else {
            (Some("."), fmt)
        }
    }
}

/// A builder for the `TimeBasedRoller`.
pub struct TimeBasedRollerBuilder;

impl TimeBasedRollerBuilder {
    /// Constructs a new `TimeBasedRoller`.
    ///
    /// `pattern` must contain at least one instance of `{}`, all of which will
    /// be replaced with an archived log file's time format.
    ///
    /// Note that the pattern is the full path to roll archived logs to.
    ///
    /// If the file extension of the pattern is `.gz` and the `gzip` Cargo
    /// feature is enabled, the archive files will be gzip-compressed.
    pub fn build(
        self,
        pattern: &str,
        count: u32,
        fmt: &str,
        scale: &str,
    ) -> Result<TimeBasedRoller, Box<dyn Error + Sync + Send>> {
        if !pattern.contains("{}") {
            return Err("pattern does not contain `{}`".into());
        }

        let compression = match Path::new(pattern).extension() {
            #[cfg(feature = "gzip")]
            Some(e) if e == "gz" => Compression::Gzip,
            #[cfg(not(feature = "gzip"))]
            Some(e) if e == "gz" => {
                return Err("gzip compression requires the `gzip` feature".into());
            }
            _ => Compression::None,
        };

        Ok(TimeBasedRoller {
            pattern: pattern.to_owned(),
            compression,
            fmt: fmt.to_owned(),
            count,
            scale: scale.to_owned(),
            #[cfg(feature = "background_rotation")]
            lock: Arc::new(Mutex::new(())),
            mock_time: Option::None,
        })
    }
}

/// A deserializer for the `TimeBasedRoller`.
///
/// # Configuration
///
/// ```yaml
/// kind: time_based
///
/// # The filename pattern for archived logs. Must contain at least one `{}`.
/// # Note that the pattern is the full path to roll archived logs to.
/// # If the file extension of the pattern is `.gz` and the `gzip` Cargo feature
/// # is enabled, the archive files will be gzip-compressed.
/// # Required.
/// pattern: archive/foo.{}.log
///
/// # The maximum number of archived logs to maintain. Required.
/// count: 5
///
/// ```
#[cfg(feature = "file")]
pub struct TimeBasedRollerDeserializer;

#[cfg(feature = "file")]
impl Deserialize for TimeBasedRollerDeserializer {
    type Trait = dyn Roll;

    type Config = TimeBasedRollerConfig;

    fn deserialize(
        &self,
        config: TimeBasedRollerConfig,
        _: &Deserializers,
    ) -> Result<Box<dyn Roll>, Box<dyn Error + Sync + Send>> {
        let builder = TimeBasedRoller::builder();
        Ok(Box::new(builder.build(
            &config.pattern,
            config.count,
            &config.fmt,
            &config.scale,
        )?))
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{Read, Write};
    use std::process::Command;
    use tempdir::TempDir;

    use super::*;
    use crate::append::rolling_file::policy::compound::roll::Roll;
    use crate::append::rolling_file::policy::compound::set_mock_time;

    static TIME_FMT: &str = "%Y-%m-%d";

    #[cfg(feature = "background_rotation")]
    fn wait_for_roller(roller: &TimeBasedRoller) {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _lock = roller.lock.lock();
    }

    #[cfg(not(feature = "background_rotation"))]
    fn wait_for_roller(_roller: &TimeBasedRoller) {}

    #[test]
    fn rotation() {
        let dir = TempDir::new("rotation").unwrap();

        let pattern = dir.path().join("foo.log");
        let roller = TimeBasedRoller::builder()
            .build(
                &format!("{}.{{}}", pattern.to_str().unwrap()),
                2,
                TIME_FMT,
                "date",
            )
            .unwrap();
        set_mock_time("2020-03-07");
        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file1").unwrap();

        roller.roll(&file).unwrap();
        wait_for_roller(&roller);
        assert!(!file.exists());
        let mut contents = vec![];
        File::open(dir.path().join("foo.log.2020-03-07"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"file1");

        File::create(&file).unwrap().write_all(b"file2").unwrap();

        set_mock_time("2020-03-08");
        roller.roll(&file).unwrap();
        wait_for_roller(&roller);
        assert!(!file.exists());
        contents.clear();
        File::open(dir.path().join("foo.log.2020-03-07"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"file1");
        contents.clear();
        File::open(dir.path().join("foo.log.2020-03-08"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"file2");

        File::create(&file).unwrap().write_all(b"file3").unwrap();

        set_mock_time("2020-03-09");
        roller.roll(&file).unwrap();
        wait_for_roller(&roller);
        assert!(!file.exists());
        contents.clear();
        assert!(!dir.path().join("foo.log.2020-03-07").exists()); // delete oldest
        File::open(dir.path().join("foo.log.2020-03-08"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"file2");
        contents.clear();
        File::open(dir.path().join("foo.log.2020-03-09"))
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"file3");
    }

    #[test]
    fn create_archive_unvaried() {
        let dir = TempDir::new("create_archive_unvaried").unwrap();

        let base = dir.path().join("log").join("archive");
        let pattern = base.join("foo.{}.log");
        let roller = TimeBasedRoller::builder()
            .build(pattern.to_str().unwrap(), 2, TIME_FMT, "date")
            .unwrap();

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file").unwrap();

        set_mock_time("2020-03-07");
        roller.roll(&file).unwrap();
        wait_for_roller(&roller);

        assert!(base.join("foo.2020-03-07.log").exists());

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file2").unwrap();

        set_mock_time("2020-03-08");
        roller.roll(&file).unwrap();
        wait_for_roller(&roller);

        assert!(base.join("foo.2020-03-07.log").exists());
        assert!(base.join("foo.2020-03-08.log").exists());
    }

    #[test]
    fn create_archive_varied() {
        let dir = TempDir::new("create_archive_unvaried").unwrap();

        let base = dir.path().join("log").join("archive");
        let pattern = base.join("{}").join("foo.log");
        let roller = TimeBasedRoller::builder()
            .build(pattern.to_str().unwrap(), 2, TIME_FMT, "date")
            .unwrap();

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file").unwrap();

        set_mock_time("2020-03-07");
        roller.roll(&file).unwrap();
        wait_for_roller(&roller);

        assert!(base.join("2020-03-07").join("foo.log").exists());

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file2").unwrap();

        set_mock_time("2020-03-08");
        roller.roll(&file).unwrap();
        wait_for_roller(&roller);

        assert!(!base.join("2020-03-07").join("foo.log").exists());
        assert!(base.join("2020-03-08").join("foo.log").exists());

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(b"file3").unwrap();

        set_mock_time("2020-03-09");
        roller.roll(&file).unwrap();
        wait_for_roller(&roller);

        assert!(!base.join("2020-03-08").join("foo.log").exists());
        assert!(base.join("2020-03-09").join("foo.log").exists());
    }

    #[test]
    #[cfg_attr(feature = "gzip", ignore)]
    fn unsupported_gzip() {
        let dir = TempDir::new("unsupported_gzip").unwrap();

        let pattern = dir.path().join("{}.gz");
        assert!(TimeBasedRoller::builder()
            .build(pattern.to_str().unwrap(), 2, TIME_FMT, "date")
            .is_err());
    }

    #[test]
    #[cfg_attr(not(feature = "gzip"), ignore)]
    // or should we force windows user to install gunzip
    #[cfg(not(windows))]
    fn supported_gzip() {
        let dir = TempDir::new("supported_gzip").unwrap();

        let pattern = dir.path().join("{}.gz");
        let roller = TimeBasedRoller::builder()
            .build(pattern.to_str().unwrap(), 2, TIME_FMT, "date")
            .unwrap();

        let contents = (0..10000).map(|i| i as u8).collect::<Vec<_>>();

        let file = dir.path().join("foo.log");
        File::create(&file).unwrap().write_all(&contents).unwrap();

        set_mock_time("2020-03-09");
        roller.roll(&file).unwrap();
        wait_for_roller(&roller);

        assert!(Command::new("gunzip")
            .arg(dir.path().join("2020-03-09.gz"))
            .status()
            .unwrap()
            .success());

        let mut file = File::open(dir.path().join("2020-03-09")).unwrap();
        let mut actual = vec![];
        file.read_to_end(&mut actual).unwrap();

        assert_eq!(contents, actual);
    }
}
