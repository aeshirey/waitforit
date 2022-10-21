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

Most `Wait` conditions include a `not:bool` parameter for negation. For example, the `Wait::Exists` variant checks for the existence of a file (ie, `.wait` will block until that file exists). When `not=true`, then it will be negated (ie, `.wait` will block until the file _doesn't_ exist).


## Crate Features
By default, this crate includes the [`ureq`](https://docs.rs/ureq/) and [`url`](https://docs.rs/url/) crates to support making HTTP requests. This does increase the number of dependencies and compile time, so if you wish to disable these, you can do so in your Cargo.toml:

```toml
waitforit = { version = "0.1.0", default_features = false }
```

## TODO
- [ ] Expand the `Custom` variant to be `Fn` or possibly `FnMut`?
- [ ] Monitor running processes (possibly with [`sysinfo` crate](https://docs.rs/sysinfo/)?)
- [ ]¹ Update `waitfor` to make use of this crate