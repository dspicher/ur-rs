Rust Uniform Resources
======================
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dspicher/ur-rs/rust.yml?branch=master&logo=github" height="20">](https://github.com/dspicher/ur-rs/actions)
[<img alt="build status" src="https://img.shields.io/codecov/c/gh/dspicher/ur-rs?logo=codecov" height="20">](https://codecov.io/gh/dspicher/ur-rs)
[<img alt="build status" src="https://img.shields.io/crates/v/ur.svg" height="20">](https://crates.io/crates/ur)

<!-- cargo-rdme start -->

`ur` is a crate to interact with ["uniform resource"](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md) encodings of binary data.
The encoding scheme is optimized for transport in URIs and QR codes.

The [`ur::Encoder`] allows a byte payload to be transmissioned in
multiple stages, respecting maximum size requirements. Under the hood,
a [`fountain`](https://en.wikipedia.org/wiki/Fountain_code) encoder is used to create an unbounded stream of URIs,
subsets of which can be recombined at the receiving side into the payload:
```rust
let data = String::from("Ten chars!").repeat(10);
let max_length = 5;
let scheme = "bytes";
let mut encoder = ur::Encoder::new(data.as_bytes(), max_length, scheme).unwrap();
let part = encoder.next_part().unwrap();
assert_eq!(
    part,
    "ur:bytes/1-20/lpadbbcsiecyvdidatkpfeghihjtcxiabdfevlms"
);
let mut decoder = ur::Decoder::default();
while !decoder.complete() {
    let part = encoder.next_part().unwrap();
    // Simulate some communication loss
    if encoder.current_index() & 1 > 0 {
        decoder.receive(&part).unwrap();
    }
}
assert_eq!(decoder.message().unwrap().as_deref(), Some(data.as_bytes()));
```

The following useful building blocks are also part of the public API:
 - The [`crate::bytewords`](https://docs.rs/ur/latest/ur/bytewords/) module contains functionality
   to encode byte payloads into a suitable alphabet, achieving hexadecimal
   byte-per-character efficiency.
 - The [`crate::fountain`](https://docs.rs/ur/latest/ur/fountain/) module provides an implementation
   of a fountain encoder, which splits up a byte payload into multiple segments
   and emits an unbounded stream of parts which can be recombined at the receiving
   decoder side.

<!-- cargo-rdme end -->

## Usage

Add `ur` to the dependencies of your `Cargo.toml`:
```toml
[dependencies]
ur = "0.2"
```

## Examples

### Animated QR code
To run this example, execute
```bash
cargo run --example qr -- "This is my super awesome UR payload"
```
which will print out URIs and QR codes transmitting the provided payload.

## Background: Uniform Resources
[Uniform Resources](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md) are
> a proposed method of encoding binary data of arbitrary content and length so that it is suitable for transport in either URIs or QR codes.

The resulting constraints on the permissible encoding alphabet are nicely analyzed [here](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-003-uri-binary-compatibility.md).

The following building blocks interact to achieve this goal:
- [Bytewords](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-012-bytewords.md) map binary data to case-insensitive characters with a 4 bits/char efficiency (identical to hexadecimal encoding)
- Fragments for transmitting multi-part messages are constructed based on a [Luby transform](https://en.wikipedia.org/wiki/Luby_transform_code) (a particular kind of [fountain encoding](https://en.wikipedia.org/wiki/Fountain_code)), generating a potentially limitless sequence of fragments, small subsets of which can restore the original message
- [CBOR](https://tools.ietf.org/html/rfc7049) allows for self-describing byte payloads
- A properly seeded [Xoshiro](https://en.wikipedia.org/wiki/Xorshift#xoshiro_and_xoroshiro) pseudo-random generator allows the encoding and decoding parties to agree on which message parts were combined into a fountain encoding fragment

## Other implementations
This Rust implementation, in particular its test vectors, is based on the following reference implementations:
- C++: [bc-ur](https://github.com/BlockchainCommons/bc-ur/)
- Swift: [URKit](https://github.com/blockchaincommons/URKit)

## Contributing
Pull requests are welcome.

## License
This project is licensed under the terms of the [MIT](https://choosealicense.com/licenses/mit/) license.
