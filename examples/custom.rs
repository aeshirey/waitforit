use std::process::{Command, Stdio};
use waitforit::Wait;

fn main() {
    Wait::new_custom(hundred_lines_or_more).condition_met();
}

fn hundred_lines_or_more() -> bool {
    // Run `wc -l Cargo.toml`
    let stdout = match Command::new("wc")
        .arg("-l")
        .arg("Cargo.toml")
        .stdout(Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(_) => return false,
    }
    .stdout;

    // Get the stdout as a string
    let stdout = String::from_utf8(stdout).expect("invalid stdout");

    // Grab the first part, which should be numeric
    let lines = stdout
        .split(' ')
        .next()
        .unwrap()
        .parse::<usize>()
        .expect("Parse number");

    // Our condition is met if we have 100+ lines
    lines >= 100
}
