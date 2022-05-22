# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
 - Added `ur::ur::decode` to the public API to decode a single `ur` URI.
 - Added `ur::ur::encode` and `ur::ur::decode` to the root library path.
 - Bumped the Rust edition to 2021. #113
 - Added an enum indicating whether the UR was single- or multip-part to `ur::ur::decode` https://github.com/dspicher/ur-rs/pull/121

## [0.2.0] - 2021-12-08
 - The public API has been greatly restricted
 - All public methods and structs are documented and should be much more stable going forward
 - Introduced fuzz testing

## [0.1.0] - 2021-08-23
 - Initial release