# waitforit

From my [`waitfor`](https://github.com/aeshirey/waitfor) app¹, this crate extracts out the functionality to wait for certain events, including:

* elapsed time
* file (non-)existence
* file updates (timestamp or file size)
* TCP host:port (un)availablity
* HTTP GET response codes
* Arbitrary user-defined (`fn() -> bool`)

## Usage
`waitforit` exposes the `Wait` and `Waits` structs. The former is some condition (eg, as above) that the user wants to wait to complete. The latter is simply a combination other conditions. Both structs expose two methods for checking their conditions:

* `.condition_met() -> bool` checks and (nearly immediately) returns whether the condition is met.
* `.wait(interval: Duration)` blocks until `.condition_met()` is true, checking every `interval`

All `Wait` conditions can be `!` negated (or, when manually constructed from the `Wait` enum's variant's, by specifying a `not:bool` parameter). For example, the `Wait::Exists` variant checks for the existence of a file (ie, `.wait` will block until that file exists). When negated, it is satisfied when the file doesn't exist.

## Crate Features
By default, this crate includes the [`ureq`](https://docs.rs/ureq/) and [`url`](https://docs.rs/url/) crates to support making HTTP requests. This increases the number of dependencies and compile time, so if you wish to disable these, you can do so in your Cargo.toml:

```toml
waitforit = { version = "0.1.0", default_features = false }
```

## Negations
Any `Wait` or `Waits` value can be negated:

```rust
let foo_exists = Wait::new_file_exists("foo.txt");
let foo_doesnt_exist = !foo_exists;
```

This even applies to the `Elapsed` variant which means the condition will be met _until_ the elapsed duration. This may prove useful when combining with other values. For example, wait for a file to be updated in the first ten seconds, and if that doesn't happen, then wait for the file to be deleted:

```rust
let filename = "my_dataset.json"; // assumed that this file exists
let first_10sec = !Wait::new_elapsed_from_duration(Duration::from_secs(10));
let file_updated = Wait::new_file_update(filename);
let file_not_exists = !Wait::new_file_exists(filename);

// either we update the file in the first 10 sec or we wait for it to be deleted
let w = (first_10sec & file_updated) | file_not_exists;
w.wait(Duration::from_secs(1));
```

## TODO
- [ ] Expand the `Custom` variant to be `Fn` or possibly `FnMut`?
- [ ] Monitor running processes (possibly with [`sysinfo` crate](https://docs.rs/sysinfo/)?)