Rust Uniform Resources
======================
[<img alt="build status" src="https://img.shields.io/github/workflow/status/dspicher/ur-rs/Rust/master?logo=github" height="20">](https://github.com/dspicher/ur-rs/actions)
[<img alt="build status" src="https://img.shields.io/codecov/c/gh/dspicher/ur-rs?logo=codecov" height="20">](https://codecov.io/gh/dspicher/ur-rs)
[<img alt="build status" src="https://img.shields.io/crates/v/ur.svg" height="20">](https://crates.io/crates/ur)

{{readme}}

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
- C++: [bc-ur](https://github.com/BlockchainCommons/bc-ur/)d
- Swift: [URKit](https://github.com/blockchaincommons/URKit)

## Contributing
Pull requests are welcome.

## License
This project is licensed under the terms of the [MIT](https://choosealicense.com/licenses/mit/) license.
