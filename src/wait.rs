use std::{
    cell::Cell,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

#[cfg(feature = "http")]
use url::Url;

/// Waits for some condition to be met.
#[derive(Clone, Debug)]
pub enum Wait {
    /// Waits until `end_instant`. This can be negated, in which case it will
    /// only trigger until the specified instant.
    Elapsed { end_instant: Instant, not: bool },

    /// Waits until `path` exists (or with `not`, until it no longer exists)
    Exists { not: bool, path: PathBuf },

    /// Waits until a file is updated (or with `not`, until it stops being updated)
    Update {
        not: bool,
        path: PathBuf,
        last_update: Cell<Option<SystemTime>>,
    },

    /// Waits until a file hasn't been updated in some specified [Duration] (or
    /// with `not`, until it hasn't been updated in at least some Duration).
    UpdateSince {
        not: bool,
        path: PathBuf,
        trigger_duration: Duration,
    },

    /// Waits until a connection can be made to `host` (or with `not`, until a
    /// connection can no longer be made).
    TcpHost { not: bool, host: String },

    /// Waits until an HTTP GET to `url` returns `status` (or with `not`, until
    /// it no longer returns that code)
    #[cfg(feature = "http")]
    HttpGet { not: bool, url: String, status: u16 },

    /// Waits until a file's size has been changed (or with `not`, until it
    /// stops changing). Nothing is implied about the direction of change.
    FileSize {
        not: bool,
        path: PathBuf,
        size_bytes: Cell<Option<u64>>,
    },

    /// Waits until the specified `fn` (not `Fn`) returns true.
    Custom { f: fn() -> bool, not: bool },
    // Pid { pid: u64, },
    // FileOpen(??), // Check if a handle is open on a particular file (ie, when a file is done being modified)
}

impl Wait {
    /// Creates a new `Wait` that will complete at `end_instant`.
    ///
    /// Negating this type does nothing.
    pub fn new_elapsed(end_instant: Instant) -> Self {
        Self::Elapsed {
            end_instant,
            not: false,
        }
    }

    /// Creates a new `Wait` that will complete after `duration` has passed,
    /// starting immediately.
    pub fn new_elapsed_from_duration(duration: Duration) -> Self {
        Self::Elapsed {
            end_instant: std::time::Instant::now() + duration,
            not: false,
        }
    }

    /// Creates a new `Wait` that completes when an HTTP GET to `url` returns
    /// the specified `status` code.
    ///
    /// When negated, this completes when an HTTP GET to `url` returns any
    /// other status value.
    #[cfg(feature = "http")]
    pub fn new_http_get<T>(url: T, status: u16) -> Self
    where
        T: Into<String>,
    {
        Self::HttpGet {
            not: false,
            url: url.into(),
            status,
        }
    }

    /// Creates a new `Wait` that completes when a TCP connection can be
    /// established to `host`.
    ///
    /// When `not` is specified, this completes when a TCP connection can no
    /// longer be established.
    pub fn new_tcp_connect<T>(host: T) -> Self
    where
        T: Into<String>,
    {
        Self::TcpHost {
            not: false,
            host: host.into(),
        }
    }

    /// Creates a new `Wait` that completes when the specified file exists.
    ///
    /// When negated, this completes when the file doesn't exist.
    pub fn new_file_exists<T>(path: T) -> Self
    where
        T: Into<PathBuf>,
    {
        Self::Exists {
            not: false,
            path: path.into(),
        }
    }

    /// Creates a new `Wait` that completes when the specified file is updated
    /// (according to its [metadata](std::fs::Metadata)'s modified time). In
    /// other words: as soon as the file is updated, this completes.
    ///
    /// When negated, this completes when the metadata has not been updated in
    /// two consecutive cycles.
    ///
    /// Contrast this with [Self::new_file_update_since], which completes when a
    /// specified duration has passed after the file was last updated.
    pub fn new_file_update<T>(path: T) -> Self
    where
        T: Into<PathBuf>,
    {
        Self::Update {
            not: false,
            path: path.into(),
            last_update: Cell::new(None),
        }
    }

    /// Creates a new `Wait` that completes when the specified file is updated
    /// (according to its [metadata](std::fs::Metadata)'s modified time).
    ///
    /// When negated, this completes when the metadata has not been updated in
    /// two consecutive cycles.
    pub fn new_file_update_since<T>(path: T, trigger_duration: Duration) -> Self
    where
        T: Into<PathBuf>,
    {
        Self::UpdateSince {
            not: false,
            path: path.into(),
            trigger_duration,
        }
    }

    /// Creates a new `Wait` that completes when the specified file's size is
    /// updated (according to its [metadata](std::fs::Metadata)'s length).
    ///
    /// When negated, this completes when the file's length has not been
    /// updated in two consecutive cycles.
    ///
    /// If metadata can't be retrieved for this file, this
    pub fn new_file_size<T>(path: T) -> Self
    where
        T: Into<PathBuf>,
    {
        Self::FileSize {
            not: false,
            path: path.into(),
            size_bytes: Cell::new(None),
        }
    }

    pub fn new_custom(f: fn() -> bool) -> Self {
        Self::Custom { f, not: false }
    }

    //

    /// Checks whether this condition is met.
    ///
    /// This is non-blocking, but depending on the variant may have some associated
    /// delay (eg, an HTTP GET incurs TCP and possibly TLS handshake latency).
    pub fn condition_met(&self) -> bool {
        match self {
            Wait::Elapsed { end_instant, not } => {
                if *not {
                    *end_instant >= Instant::now()
                } else {
                    *end_instant < Instant::now()
                }
            }
            Wait::Exists { not: true, path } => !Path::new(path).exists(),
            Wait::Exists { not: false, path } => Path::new(path).exists(),
            #[cfg(feature = "http")]
            Wait::HttpGet { not, url, status } => {
                let result = ureq::get(url).call();
                if *not {
                    *status != result.status()
                } else {
                    *status == result.status()
                }
            }
            Wait::TcpHost { not: false, host } => std::net::TcpStream::connect(host).is_ok(),
            Wait::TcpHost { not: true, host } => std::net::TcpStream::connect(host).is_err(),
            Wait::Update {
                not,
                path,
                last_update,
            } => {
                // "Update" checks that the file has (not) been updated in the last 'trigger_duration'
                let current_modified = match get_modified_time(path) {
                    Some(systime) => systime,
                    None => return true, // Can't get the modified time, so we'll assume the condition is met.
                };

                match last_update.get() {
                    Some(last_updated) => {
                        let is_updated = last_updated != current_modified;

                        if *not {
                            // We want to trigger when the file *isn't* updating.
                            if is_updated {
                                // Shouldn't trigger yet, but we should update the last known modified date
                                last_update.set(Some(current_modified));
                                false
                            } else {
                                // File hasn't updated, so we should trigger
                                true
                            }
                        } else {
                            // Since not==false: iff the file is updated, we trigger.
                            // Triggering on an updated mod time doesn't ever need to update the value
                            is_updated
                        }
                    }
                    None => {
                        // Haven't tracked the time yet. We'll hang onto it now for the next iteration
                        last_update.set(Some(current_modified));
                        false
                    }
                }
            }
            Wait::UpdateSince {
                not,
                path,
                trigger_duration,
            } => {
                // "Update" checks that the file has (not) been updated in the last 'trigger_duration'
                let last_updated = match get_modified_time(path) {
                    Some(up) => up,
                    None => return true, // Can't get the modified time, so we'll assume the condition is met.
                };

                let since_last_update = match SystemTime::now().duration_since(last_updated) {
                    Ok(d) => d,
                    Err(_) => return true, // can't calculate the duration, so we'll assume the condition is met
                };

                let is_recently_updated = since_last_update < *trigger_duration;

                // The condition is met if:
                // 1. we're looking for a recent update and it was recently updated, or
                // 2. it's not recently updated and we're not looking for it, which means
                // condition_met := is_recently_updated XOR not_negating
                is_recently_updated ^ not
            }

            Wait::FileSize {
                not,
                path,
                size_bytes: bytes,
            } => {
                match (bytes.get(), get_file_size(path)) {
                    // Can't get the file size. This is probably due to file non-existence,
                    // so we'll assume the condition is met
                    (_, None) => true,
                    // Sizes are different when not negating -- condition is met
                    (Some(prev), Some(curr)) if !*not && prev != curr => true,
                    // Size hasn't changed when negating -- condition is met
                    (Some(prev), Some(curr)) if *not && prev == curr => true,
                    // First time or subsequent with changing values - save the (new) size and try again
                    (_, curr) => {
                        bytes.set(curr);
                        false
                    }
                }
            }

            Wait::Custom { f, not } => {
                if *not {
                    !(f)()
                } else {
                    (f)()
                }
            } //Wait::Pid { pid: _ } => todo!(),
        }
    }

    /// Wait for the completion of this condition. This will block the thread.
    pub fn wait(&self, interval: Duration) {
        loop {
            let start = Instant::now();
            if self.condition_met() {
                return;
            }

            let loop_time = start.elapsed();
            if interval > loop_time {
                std::thread::sleep(interval - loop_time);
            }
        }
    }
}

impl std::ops::Not for Wait {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        let not = match &mut self {
            Wait::Elapsed { not, .. } => not,
            Wait::Exists { not, .. } => not,
            Wait::HttpGet { not, .. } => not,
            Wait::TcpHost { not, .. } => not,
            Wait::Update { not, .. } => not,
            Wait::UpdateSince { not, .. } => not,
            Wait::FileSize { not, .. } => not,
            Wait::Custom { not, .. } => not,
        };

        *not = !*not;

        self

        /*
        match self {
            Wait::Elapsed { end_instant, not } => Wait::Elapsed {
                end_instant,
                not: !not,
            },
            Wait::Exists { not, path } => Wait::Exists { not: !not, path },
            Wait::HttpGet { not, url, status } => Wait::HttpGet {
                not: !not,
                url,
                status,
            },
            Wait::TcpHost { not, host } => Wait::TcpHost { not: !not, host },
            Wait::Update {
                not,
                path,
                last_update,
            } => Wait::Update {
                not: !not,
                path,
                last_update,
            },
            Wait::UpdateSince {
                not,
                path,
                trigger_duration,
            } => Wait::UpdateSince {
                not: !not,
                path,
                trigger_duration,
            },
            Wait::FileSize {
                not,
                path,
                size_bytes,
            } => Wait::FileSize {
                not: !not,
                path,
                size_bytes,
            },
            Wait::Custom { f, not } => Wait::Custom { f, not: !not },
        }
        */
    }
}

fn get_modified_time(path: &Path) -> Option<SystemTime> {
    let meta = path.metadata().ok()?;
    meta.modified().ok()
}

fn get_file_size(path: &Path) -> Option<u64> {
    let meta = path.metadata().ok()?;
    Some(meta.len())
}

/// Parses a simple human-readable duration, returning a `Duration`
///
/// "3h10m" -> 11400 seconds
pub fn parse_duration(duration: &str) -> Option<Duration> {
    let mut total_delay = 0;

    let mut acc = 0;
    for c in duration.chars() {
        match c {
            '0'..='9' => {
                acc *= 10;
                acc += c.to_digit(10).unwrap();
            }
            'd' => {
                // days
                total_delay += acc * 86400;
                acc = 0;
            }
            'h' => {
                // hours
                total_delay += acc * 3600;
                acc = 0;
            }
            'm' => {
                // minutes
                total_delay += acc * 60;
                acc = 0;
            }
            's' => {
                // seconds
                total_delay += acc;
                acc = 0;
            }
            _ => return None,
        }
    }

    total_delay += acc;

    let d = Duration::from_secs(total_delay as u64);

    Some(d)
}

/// Parses an input argument for an HTTP GET into the expected status code and URL to hit.
///
/// The URL is validated with the `url` crate, if possible, cleaning potential errors.
/// If that fails, the URL is used as-is.
#[cfg(feature = "http")]
pub fn parse_http_get(urlarg: &str) -> (u16, String) {
    let urlbytes = urlarg.chars().collect::<Vec<_>>();

    let (status_code, urlarg) = if urlarg.len() > 4
        && urlbytes[0..3].iter().all(|c| c.is_numeric())
        && urlbytes[3] == ','
    {
        let code = 100 * (urlbytes[0] as u16 - '0' as u16)
            + 10 * (urlbytes[1] as u16 - '0' as u16)
            + (urlbytes[2] as u16 - '0' as u16);

        (code, &urlarg[4..])
    } else {
        (200, urlarg)
    };

    if let Some(url) = parse_url(urlarg) {
        (status_code, url.to_string())
    } else {
        (status_code, urlarg.to_string())
    }
}

/// Tries to parse a URL using the `url` crate.
#[cfg(feature = "http")]
fn parse_url(urlarg: &str) -> Option<Url> {
    let violations = std::cell::RefCell::new(Vec::new());
    let url = Url::options()
        .syntax_violation_callback(Some(&|v| {
            violations.borrow_mut().push(v);
        }))
        .parse(urlarg)
        .ok()?;

    Some(url)
}

/// Checks that the input appears to be a valid `hostname:port` input, where `port`
/// is a u16.
pub fn validate_tcp(hostarg: &str) -> bool {
    // Assume that the last location of ':' is the delimiter for the port
    let last_colon = hostarg.char_indices().filter(|(_i, c)| c == &':').last();
    if let Some((i, _c)) = last_colon {
        // Everything after the colon should be a u16 port number
        let port = &hostarg[i + 1..];
        port.parse::<u16>().is_ok()
    } else {
        // There's no ':' in the input, so assume this isn't a host to which we can connect
        false
    }
}

mod tests {
    #[test]
    fn valid_tcp() {
        assert!(super::validate_tcp("localhost:80"));
        assert!(!super::validate_tcp("localhost"));

        assert!(super::validate_tcp("127.0.0.1:80"));
        assert!(!super::validate_tcp("127.0.0.1"));

        assert!(super::validate_tcp("127.0.0.1:8000"));
        assert!(super::validate_tcp("127.0.0.1:65534"));
        assert!(super::validate_tcp("127.0.0.1:65535"));
        assert!(!super::validate_tcp("127.0.0.1:65536"));
        assert!(!super::validate_tcp("127.0.0.1:-1"));
    }
}
