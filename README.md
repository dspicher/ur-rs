Rust Uniform Resources
======================
[<img alt="build status" src="https://img.shields.io/github/workflow/status/dspicher/ur-rs/Rust/master?logo=github" height="20">](https://github.com/dspicher/ur-rs/actions)
[<img alt="build status" src="https://img.shields.io/codecov/c/gh/dspicher/ur-rs?logo=codecov" height="20">](https://codecov.io/gh/dspicher/ur-rs)
[<img alt="build status" src="https://img.shields.io/crates/v/ur.svg" height="20">](https://crates.io/crates/ur)

<!-- cargo-rdme start -->

`ur` is a crate to interact with ["uniform resource"](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md) encodings of binary data.
The encoding scheme is optimized for transport in URIs and QR codes.

### Encode binary data
The [`crate::bytewords`](https://docs.rs/ur/latest/ur/bytewords/) module defines multiple encoding styles.
The minimal style, demonstrated below, encodes each byte into two characters.
```rust
use ur::bytewords::{decode, encode, Style};
let data = "Some binary data".as_bytes();
let encoded = encode(data, &Style::Minimal);
assert_eq!(encoded, "gujljnihcxidinjthsjpkkcxiehsjyhsnsgdmkht");
let decoded = decode(&encoded, &Style::Minimal).unwrap();
assert_eq!(data, decoded);
```

### Split up payloads into uniform resource URIs
The encoder splits up payloads into chunks and encodes them into URIs.
The payload part of the URI contains additional information necessary for
decoding, as well as a checksum.
```rust
let data = String::from("Ten chars!").repeat(10);
let max_length = 5;
let mut encoder = ur::Encoder::new(data.as_bytes(), max_length, "bytes").unwrap();
let part = encoder.next_part().unwrap();
assert_eq!(part, "ur:bytes/1-20/lpadbbcsiecyvdidatkpfeghihjtcxiabdfevlms");
let part = encoder.next_part().unwrap();
assert_eq!(part, "ur:bytes/2-20/lpaobbcsiecyvdidatkpfeishsjpjkclwewffhad");
```

### Emit a stream of URs that can be recombined into the payload
Finally, those URIs can be consumed by a decoder to restore the original
payload. The receiver can start to receive at any time and miss arbitrary
transmissions. This is useful for example in the context of an animated
QR code.
```rust
let data = String::from("Ten chars!").repeat(10);
let max_length = 5;
let mut encoder = ur::Encoder::new(data.as_bytes(), max_length, "bytes").unwrap();
let mut decoder = ur::Decoder::default();
while !decoder.complete() {
    let part = encoder.next_part().unwrap();
    // Simulate some communication loss
    if encoder.current_index() & 1 > 0 {
        decoder.receive(&part).unwrap();
    }
}
assert_eq!(decoder.message().unwrap(), data.as_bytes());
```

<!-- cargo-rdme end -->

## Uniform Resources
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
