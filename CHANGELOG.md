# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.4.1](https://github.com/dspicher/ur-rs/releases/tag/0.4.1) - 2023-10-16
 - Take a reference to custom UR type identifiers

## [0.4.0](https://github.com/dspicher/ur-rs/releases/tag/0.4.0) - 2023-08-04
 - Added support for `no-std` environments. https://github.com/dspicher/ur-rs/pull/183
 - Introduced a type-safe `ur::Type` enum and a `ur::Encoder::bytes` shorthand constructor (see the below migration guide). https://github.com/dspicher/ur-rs/pull/186
 - Added `wasm` example. https://github.com/dspicher/ur-rs/pull/191

### Migration guide

Replace `Encoder` constructors with `bytes` schemes:
```rust
ur::Encoder::new(data, max_length, "bytes")
```

with:
```rust
ur::Encoder::bytes(data, max_length)
```

Leave all other `Encoder` constructors as they are:
```rust
ur::Encoder::new(data, max_length, "my-scheme")
```

## [0.3.0](https://github.com/dspicher/ur-rs/releases/tag/0.3.0) - 2023-01-07
 - Added `ur::ur::decode` to the public API to decode a single `ur` URI. https://github.com/dspicher/ur-rs/pull/112
 - Added `ur::ur::encode` and `ur::ur::decode` to the root library path. https://github.com/dspicher/ur-rs/pull/112
 - Bumped the Rust edition to 2021. https://github.com/dspicher/ur-rs/pull/113
 - Added an enum indicating whether the UR was single- or multip-part to `ur::ur::decode`. https://github.com/dspicher/ur-rs/pull/121
 - Migrated from `anyhow` errors to a custom error enum. https://github.com/dspicher/ur-rs/pull/159
 - Remove `std::fmt::Display` implementation of `Part`. https://github.com/dspicher/ur-rs/pull/160

## [0.2.0](https://github.com/dspicher/ur-rs/releases/tag/0.2.0) - 2021-12-08
 - The public API has been greatly restricted
 - All public methods and structs are documented and should be much more stable going forward
 - Introduced fuzz testing

## 0.1.0 - 2021-08-23
 - Initial release
