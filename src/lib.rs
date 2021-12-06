//! `ur` is a crate to interact with ["uniform resource"](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md) encodings of binary data.
//! The encoding scheme is optimized for transport in URIs and QR codes.
//!
//! ### Encode binary data
//! The [`crate::bytewords`](crate::bytewords) module defines multiple encoding styles.
//! The minimal style, demonstrated below, encodes each byte into two characters.
//! ```
//! use ur::bytewords::{decode, encode, Style};
//! let data = "Some binary data".as_bytes();
//! let encoded = encode(data, &Style::Minimal);
//! assert_eq!(encoded, "gujljnihcxidinjthsjpkkcxiehsjyhsnsgdmkht");
//! let decoded = decode(&encoded, &Style::Minimal).unwrap();
//! assert_eq!(data, decoded);
//! ```
//!
//! ### Split up payloads into uniform resource URIs
//! The encoder splits up payloads into chunks and encodes them into URIs.
//! The payload part of the URI contains additional information necessary for
//! decoding, as well as a checksum.
//! ```
//! let data = String::from("Ten chars!").repeat(10);
//! let max_length = 5;
//! let mut encoder = ur::Encoder::new(data.as_bytes(), max_length, "bytes").unwrap();
//! let part = encoder.next_part().unwrap();
//! assert_eq!(part, "ur:bytes/1-20/lpadbbcsiecyvdidatkpfeghihjtcxiabdfevlms");
//! let part = encoder.next_part().unwrap();
//! assert_eq!(part, "ur:bytes/2-20/lpaobbcsiecyvdidatkpfeishsjpjkclwewffhad");
//! ```
//!
//! ### Emit a stream of URs that can be recombined into the payload
//! Finally, those URIs can be consumed by a decoder to restore the original
//! payload. The receiver can start to receive at any time and miss arbitrary
//! transmissions. This is useful for example in the context of an animated
//! QR code.
//! ```
//! let data = String::from("Ten chars!").repeat(10);
//! let max_length = 5;
//! let mut encoder = ur::Encoder::new(data.as_bytes(), max_length, "bytes").unwrap();
//! let mut decoder = ur::Decoder::default();
//! while !decoder.complete() {
//!     let part = encoder.next_part().unwrap();
//!     // Simulate some communication loss
//!     if encoder.current_index() & 1 > 0 {
//!         decoder.receive(&part).unwrap();
//!     }
//! }
//! assert_eq!(decoder.message().unwrap(), data.as_bytes());
//! ```

pub mod bytewords;
pub mod constants;
pub mod fountain;
pub mod sampler;
pub mod ur;
pub mod xoshiro;

pub use self::ur::Decoder;
pub use self::ur::Encoder;

#[must_use]
pub(crate) fn crc32() -> crc::Crc<u32> {
    crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC)
}
