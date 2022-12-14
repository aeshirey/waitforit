use std::time::Duration;
use waitforit::Wait;

fn main() {
    let filename = "my_dataset.json";

    // assume that our file exists
    assert!(std::path::Path::new(filename).exists());

    let first_10sec = !Wait::new_elapsed_from_duration(Duration::from_secs(10));
    let file_updated = Wait::new_file_update(filename);
    let file_not_exists = !Wait::new_file_exists(filename);

    // either we update the file in the first 10 sec or we wait for it to be deleted
    let w = (first_10sec & file_updated) | file_not_exists;
    w.wait(Duration::from_secs(1));
}
