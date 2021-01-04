# interfaces-rs

[![Actions Status](https://github.com/andrew-d/interfaces-rs/workflows/Regression/badge.svg)](https://github.com/andrew-d/interfaces-rs/actions)
[![Crate](https://img.shields.io/crates/v/interfaces.svg)](https://crates.io/crates/interfaces)
[![Docs](https://docs.rs/interfaces/badge.svg)](https://docs.rs/interfaces)

This project consists of functions to work with network interfaces in a
cross-platform manner.

# Example

Add this to your `Cargo.toml`:

```
[dependencies]
interfaces = "0.0.5"
```

Then, in your crate:

```rust
extern crate interfaces;

use interfaces::Interface;
```

# License

MIT or Apache 2.0
