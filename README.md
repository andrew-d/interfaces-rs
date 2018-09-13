# interfaces-rs

[![Build Status](https://travis-ci.org/aep/interfaces-rs.svg?branch=master)](https://travis-ci.org/aep/interfaces-rs)
[![Crate](https://img.shields.io/crates/v/interfaces2.svg)](https://crates.io/crates/interfaces2)
[![Docs](https://docs.rs/interfaces2/badge.svg)](https://docs.rs/interfaces2)

This project consists of functions to work with network interfaces in a
cross-platform manner.

forked from the abondoned andrew-d/interfaces-rs

# Example

Add this to your `Cargo.toml`:

```
[dependencies]
interfaces2 = "0.0.4"
```

Then, in your crate:

```rust
extern crate interfaces2 as interfaces;

use interfaces::Interface;
```

# License

MIT or Apache 2.0
