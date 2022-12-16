//! `ur` is a crate to interact with ["uniform resource"](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md) encodings of binary data.
//! The encoding scheme is optimized for transport in URIs and QR codes.
//!
//! The [`ur::Encoder`] allows a byte payload to be transmissioned in
//! multiple stages, respecting maximum size requirements. Under the hood,
//! a [`fountain`](https://en.wikipedia.org/wiki/Fountain_code) encoder is used to create an unbounded stream of URIs,
//! subsets of which can be recombined at the receiving side into the payload:
//! ```
//! let data = String::from("Ten chars!").repeat(10);
//! let max_length = 5;
//! let scheme = "bytes";
//! let mut encoder = ur::Encoder::new(data.as_bytes(), max_length, scheme).unwrap();
//! let part = encoder.next_part().unwrap();
//! assert_eq!(
//!     part,
//!     "ur:bytes/1-20/lpadbbcsiecyvdidatkpfeghihjtcxiabdfevlms"
//! );
//! let mut decoder = ur::Decoder::default();
//! while !decoder.complete() {
//!     let part = encoder.next_part().unwrap();
//!     // Simulate some communication loss
//!     if encoder.current_index() & 1 > 0 {
//!         decoder.receive(&part).unwrap();
//!     }
//! }
//! assert_eq!(decoder.message().unwrap().as_deref(), Some(data.as_bytes()));
//! ```
//!
//! The following useful building blocks are also part of the public API:
//!  - The [`crate::bytewords`](crate::bytewords) module contains functionality
//!    to encode byte payloads into a suitable alphabet, achieving hexadecimal
//!    byte-per-character efficiency.
//!  - The [`crate::fountain`](crate::fountain) module provides an implementation
//!    of a fountain encoder, which splits up a byte payload into multiple segments
//!    and emits an unbounded stream of parts which can be recombined at the receiving
//!    decoder side.
#![forbid(unsafe_code)]

pub mod bytewords;
pub(crate) mod constants;
pub mod fountain;
pub(crate) mod sampler;
pub mod ur;
pub(crate) mod xoshiro;

pub use self::ur::decode;
pub use self::ur::encode;
pub use self::ur::Decoder;
pub use self::ur::Encoder;

#[must_use]
pub(crate) fn crc32() -> crc::Crc<u32> {
    crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC)
}
