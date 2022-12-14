use std::time::Duration;
use waitforit::Wait;

const CHECK_DURATION: Duration = Duration::from_secs(1);

// This example shows how negations of `Wait` (and `Waits`) work.
// It checks for existence/non-existence of a file named "something.lock",
// which, for demonstration, we presume will never exist.
fn main() {
    let ten_sec = Wait::new_elapsed_from_duration(Duration::from_secs(10));
    let lockfile = !Wait::new_file_exists("something.lock");

    // wait until ten seconds has passed and the lockfile is gone
    let w = ten_sec & lockfile;
    let start = std::time::Instant::now();
    w.wait(CHECK_DURATION);
    println!("Step 1 complete after {:?}", start.elapsed());

    // w      is     (ten seconds has passed) and (not(lockfile exists))
    // not(w) is not((ten seconds has passed) and (not(lockfile exists)))
    //        -> (not(ten seconds has passed) or (lockfile exists))
    let ten_sec = Wait::new_elapsed_from_duration(Duration::from_secs(10));
    let lockfile = !Wait::new_file_exists("something.lock");
    let w = ten_sec & lockfile;
    let not_w = !w;
    let start = std::time::Instant::now();
    not_w.wait(CHECK_DURATION);
    println!("Step 2 complete after {:?}", start.elapsed());
}
