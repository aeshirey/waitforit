use crate::wait::Wait;
use std::time::{Duration, Instant};

/// Handles waiting for one or more [Wait]s.
pub enum Waits {
    Single(Wait),
    Or(Box<(Waits, Waits)>),
    And(Box<(Waits, Waits)>),
}

impl Waits {
    /// Checks whether this condition - comprising all constituent [Wait]s - is satisfied.
    ///
    /// This is non-blocking, but depending on the conditions that comprise it, it may 
    /// have some associated delay (eg, an HTTP GET incurs TCP and possibly TLS handshake 
    /// latency). 
    /// 
    /// Short-circuit functionality may also affect the delay. For example:
    /// 
    /// ```
    /// let a = Wait::new_file_exists("foo.txt");
    /// let b = Wait::new_tcp_connect("extremely_slow_host:80", false);
    /// 
    /// let ab = (a.clone() | b.clone()).condition_met();
    /// let ba = (b | a).condition_met();
    /// ```
    pub fn condition_met(&self) -> bool {
        match self {
            Waits::Single(u) => u.condition_met(),
            Waits::Or(cc) => cc.0.condition_met() || cc.1.condition_met(),
            Waits::And(cc) => cc.0.condition_met() && cc.1.condition_met(),
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

impl From<Wait> for Waits {
    fn from(w: Wait) -> Self {
        Waits::Single(w)
    }
}

impl std::ops::BitOr for Wait {
    type Output = Waits;

    fn bitor(self, other: Wait) -> Self::Output {
        Waits::Or(Box::new((self.into(), other.into())))
    }
}

impl std::ops::BitAnd for Wait {
    type Output = Waits;

    fn bitand(self, other: Wait) -> Self::Output {
        Waits::And(Box::new((self.into(), other.into())))
    }
}

impl std::ops::BitOr<Waits> for Wait {
    type Output = Waits;

    fn bitor(self, other: Waits) -> Self::Output {
        Waits::Or(Box::new((self.into(), other)))
    }
}

impl std::ops::BitAnd<Waits> for Wait {
    type Output = Waits;

    fn bitand(self, other: Waits) -> Self::Output {
        Waits::And(Box::new((self.into(), other)))
    }
}

impl std::ops::BitOr<Wait> for Waits {
    type Output = Self;

    fn bitor(self, other: Wait) -> Self {
        Waits::Or(Box::new((self, other.into())))
    }
}

impl std::ops::BitAnd<Wait> for Waits {
    type Output = Self;

    fn bitand(self, other: Wait) -> Self {
        Waits::And(Box::new((self, other.into())))
    }
}

impl std::ops::BitAnd for Waits {
    type Output = Self;

    fn bitand(self, other: Waits) -> Self {
        Waits::And(Box::new((self, other)))
    }
}

impl std::ops::BitOr for Waits {
    type Output = Self;

    fn bitor(self, other: Waits) -> Self {
        Waits::Or(Box::new((self, other)))
    }
}
