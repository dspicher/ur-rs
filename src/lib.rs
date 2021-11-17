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
