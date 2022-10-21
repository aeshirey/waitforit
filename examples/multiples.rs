use std::time::Duration;

use waitforit::Wait;
fn main() {
    // Wait for foo.txt to exist
    let foo_exists = Wait::new_file_exists("foo.txt", false);

    // And for foo.txt to stop being updated (considering it done if it's not been updated in at least 10sec).
    let foo_done = Wait::new_file_update_since("foo.txt", true, Duration::from_secs(10));

    // Require these two conditions together (in order):
    let foo = foo_exists & foo_done;

    // No more than 30 seconds of elapsed time:
    let bar = Wait::new_elapsed_from_duration(Duration::from_secs(30));

    // Block until either foo or bar has completed, checking them every 1 second
    (foo | bar).wait(Duration::from_secs(1));
}
