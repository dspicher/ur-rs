//! `ur` is a crate to interact with "Uniform Resource" encodings of binary data.
//! The encoding scheme is optimized for transport in URIs and QR codes.
//!
//! # Encode binary data
//! ```
//! use ur::bytewords::{encode, Style};
//! let encoded = encode("Some binary data".as_bytes(), &Style::Minimal).unwrap();
//! assert_eq!(encoded, "gujljnihcxidinjthsjpkkcxiehsjyhsnsgdmkht");
//! ```
//!
//! # Split up payloads into uniform resource URIs
//!
//! This uses the minimal bytewords encoding scheme demonstrated above.
//! ```
//! let data = String::from("Some binary data").repeat(100);
//! let mut encoder = ur::Encoder::new(data.as_bytes(), 10, "bytes").unwrap();
//! let part = encoder.next_part().unwrap();
//! assert_eq!(part, "ur:bytes/1-160/lpadcsnbcfamfzcybkmuldbwgegujljnihcxidinjthsjpmezolsld");
//! ```
//!
//! # Emit a stream of URs that can be recombined into the payload
//!
//! This example is best understood in the context of an animated QR code
//! transport. The receiver can start to receive at any time, miss arbitrary
//! transmissions, and still successfully restores the payload.
//! ```
//! use ur::{Decoder, Encoder};
//! let data = String::from("Some binary data").repeat(100);
//! let mut encoder = Encoder::new(data.as_bytes(), 10, "bytes").unwrap();
//! let mut decoder = Decoder::default();
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
pub fn crc32() -> crc::Crc<u32> {
    crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC)
}
