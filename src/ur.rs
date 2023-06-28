//! Split up big payloads into constantly sized URIs which can be recombined by a decoder.
//!
//! The `ur` module provides thin wrappers around fountain en- and decoders
//! which turn these fountain parts into URIs. To this end the fountain part
//! attributes (data, checksum, indexes being used, etc.) are combined with
//! CBOR into a self-describing byte payload and encoded with the `bytewords`
//! encoding into URIs suitable for web transport and QR codes.
//! ```
//! let data = String::from("Ten chars!").repeat(10);
//! let max_length = 5;
//! let mut encoder = ur::Encoder::bytes(data.as_bytes(), max_length).unwrap();
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

extern crate alloc;
use alloc::{string::String, vec::Vec};

/// Errors that can happen during encoding and decoding of URs.
#[derive(Debug)]
pub enum Error {
    Bytewords(crate::bytewords::Error),
    Fountain(crate::fountain::Error),
    /// Invalid scheme.
    InvalidScheme,
    /// No type specified.
    TypeUnspecified,
    /// Invalid characters.
    InvalidCharacters,
    /// Invalid indices in multi-part UR.
    InvalidIndices,
    /// Tried to decode a single-part UR as multi-part.
    NotMultiPart,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bytewords(e) => write!(f, "{e}"),
            Self::Fountain(e) => write!(f, "{e}"),
            Self::InvalidScheme => write!(f, "Invalid scheme"),
            Self::TypeUnspecified => write!(f, "No type specified"),
            Self::InvalidCharacters => write!(f, "Type contains invalid characters"),
            Self::InvalidIndices => write!(f, "Invalid indices"),
            Self::NotMultiPart => write!(f, "Can't decode single-part UR as multi-part"),
        }
    }
}

impl From<crate::bytewords::Error> for Error {
    fn from(e: crate::bytewords::Error) -> Self {
        Self::Bytewords(e)
    }
}

impl From<crate::fountain::Error> for Error {
    fn from(e: crate::fountain::Error) -> Self {
        Self::Fountain(e)
    }
}

/// Encodes a data payload into a single URI
///
/// # Examples
///
/// ```
/// assert_eq!(
///     ur::ur::encode(b"data", &ur::Type::Bytes),
///     "ur:bytes/iehsjyhspmwfwfia"
/// );
/// ```
#[must_use]
pub fn encode(data: &[u8], ur_type: &Type) -> String {
    let body = crate::bytewords::encode(data, crate::bytewords::Style::Minimal);
    encode_ur(&[ur_type.encoding(), body])
}

#[must_use]
fn encode_ur(items: &[String]) -> String {
    alloc::format!("{}:{}", "ur", items.join("/"))
}

/// The type of uniform resource.
pub enum Type {
    /// A `bytes` uniform resource.
    Bytes,
    /// A custom uniform resource.
    Custom(String),
}

impl Type {
    fn encoding(&self) -> String {
        match self {
            Self::Bytes => "bytes".into(),
            Self::Custom(s) => s.clone(),
        }
    }
}

/// A uniform resource encoder with an underlying fountain encoding.
///
/// # Examples
///
/// See the [`crate::ur`] module documentation for an example.
pub struct Encoder {
    fountain: crate::fountain::Encoder,
    ur_type: Type,
}

impl Encoder {
    /// Creates a new [`bytes`] [`Encoder`] for given a message payload.
    ///
    /// The emitted fountain parts will respect the maximum fragment length argument.
    ///
    /// # Examples
    ///
    /// See the [`crate::ur`] module documentation for an example.
    ///
    /// # Errors
    ///
    /// If an empty message or a zero maximum fragment length is passed, an error
    /// will be returned.
    ///
    /// [`bytes`]: Type::Bytes
    pub fn bytes(message: &[u8], max_fragment_length: usize) -> Result<Self, Error> {
        Ok(Self {
            fountain: crate::fountain::Encoder::new(message, max_fragment_length)?,
            ur_type: Type::Bytes,
        })
    }

    /// Creates a new [`custom`] [`Encoder`] for given a message payload.
    ///
    /// The emitted fountain parts will respect the maximum fragment length argument.
    ///
    /// # Errors
    ///
    /// If an empty message or a zero maximum fragment length is passed, an error
    /// will be returned.
    ///
    /// [`custom`]: Type::Custom
    pub fn new(
        message: &[u8],
        max_fragment_length: usize,
        s: impl Into<String>,
    ) -> Result<Self, Error> {
        Ok(Self {
            fountain: crate::fountain::Encoder::new(message, max_fragment_length)?,
            ur_type: Type::Custom(s.into()),
        })
    }

    /// Returns the URI corresponding to next fountain part.
    ///
    /// # Examples
    ///
    /// See the [`crate::ur`] module documentation for an example.
    ///
    /// # Errors
    ///
    /// If serialization fails an error will be returned.
    pub fn next_part(&mut self) -> Result<String, Error> {
        let part = self.fountain.next_part();
        let body = crate::bytewords::encode(&part.cbor()?, crate::bytewords::Style::Minimal);
        Ok(encode_ur(&[
            self.ur_type.encoding(),
            part.sequence_id(),
            body,
        ]))
    }

    /// Returns the current count of already emitted parts.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut encoder = ur::Encoder::bytes(b"data", 5).unwrap();
    /// assert_eq!(encoder.current_index(), 0);
    /// encoder.next_part().unwrap();
    /// assert_eq!(encoder.current_index(), 1);
    /// ```
    #[must_use]
    pub const fn current_index(&self) -> usize {
        self.fountain.current_sequence()
    }

    /// Returns the number of segments the original message has been split up into.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut encoder = ur::Encoder::bytes(b"data", 3).unwrap();
    /// assert_eq!(encoder.fragment_count(), 2);
    /// ```
    #[must_use]
    pub fn fragment_count(&self) -> usize {
        self.fountain.fragment_count()
    }
}

/// An enum used to indicate whether a UR is single- or
/// multip-part. See e.g. [`decode`] where it is returned.
#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    SinglePart,
    MultiPart,
}

/// Decodes a single URI (either single- or multi-part)
/// into a tuple consisting of the [`Kind`] and the data
/// payload.
///
/// # Examples
///
/// ```
/// assert_eq!(
///     ur::ur::decode("ur:bytes/iehsjyhspmwfwfia").unwrap(),
///     (ur::ur::Kind::SinglePart, b"data".to_vec())
/// );
/// assert_eq!(
///     ur::ur::decode("ur:bytes/1-2/iehsjyhspmwfwfia").unwrap(),
///     (ur::ur::Kind::MultiPart, b"data".to_vec())
/// );
/// ```
///
/// # Errors
///
/// This function errors for invalid inputs, for example
/// an invalid scheme different from "ur" or an invalid number
/// of "/" separators.
pub fn decode(value: &str) -> Result<(Kind, Vec<u8>), Error> {
    let strip_scheme = value.strip_prefix("ur:").ok_or(Error::InvalidScheme)?;
    let (type_, strip_type) = strip_scheme.split_once('/').ok_or(Error::TypeUnspecified)?;

    if !type_
        .trim_start_matches(|c: char| c.is_ascii_alphanumeric() || c == '-')
        .is_empty()
    {
        return Err(Error::InvalidCharacters);
    }

    match strip_type.rsplit_once('/') {
        None => Ok((
            Kind::SinglePart,
            crate::bytewords::decode(strip_type, crate::bytewords::Style::Minimal)?,
        )),
        Some((indices, payload)) => {
            let (idx, idx_total) = indices.split_once('-').ok_or(Error::InvalidIndices)?;
            if idx.parse::<u16>().is_err() || idx_total.parse::<u16>().is_err() {
                return Err(Error::InvalidIndices);
            }

            Ok((
                Kind::MultiPart,
                crate::bytewords::decode(payload, crate::bytewords::Style::Minimal)?,
            ))
        }
    }
}

/// A uniform resource decoder able to receive URIs that encode a fountain part.
///
/// # Examples
///
/// See the [`crate::ur`] module documentation for an example.
#[derive(Default)]
pub struct Decoder {
    fountain: crate::fountain::Decoder,
}

impl Decoder {
    /// Receives a URI representing a CBOR and `bytewords`-encoded fountain part
    /// into the decoder.
    ///
    /// # Examples
    ///
    /// See the [`crate::ur`] module documentation for an example.
    ///
    /// # Errors
    ///
    /// This function may error along all the necessary decoding steps:
    ///  - The string may not be a well-formed URI according to the uniform resource scheme
    ///  - The URI payload may not be a well-formed `bytewords` string
    ///  - The decoded byte payload may not be valid CBOR
    ///  - The CBOR-encoded fountain part may be inconsistent with previously received ones
    ///
    /// In all these cases, an error will be returned.
    pub fn receive(&mut self, value: &str) -> Result<(), Error> {
        let (kind, decoded) = decode(value)?;
        if kind != Kind::MultiPart {
            return Err(Error::NotMultiPart);
        }

        self.fountain
            .receive(crate::fountain::Part::from_cbor(decoded.as_slice())?)?;
        Ok(())
    }

    /// Returns whether the decoder is complete and hence the message available.
    ///
    /// # Examples
    ///
    /// See the [`crate::ur`] module documentation for an example.
    #[must_use]
    pub fn complete(&self) -> bool {
        self.fountain.complete()
    }

    /// If [`complete`], returns the decoded message, `None` otherwise.
    ///
    /// # Errors
    ///
    /// If an inconsistent internal state detected, an error will be returned.
    ///
    /// # Examples
    ///
    /// See the [`crate::ur`] module documentation for an example.
    ///
    /// [`complete`]: Decoder::complete
    pub fn message(&self) -> Result<Option<Vec<u8>>, Error> {
        self.fountain.message().map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minicbor::{bytes::ByteVec, data::Tag};

    fn make_message_ur(length: usize, seed: &str) -> Vec<u8> {
        let message = crate::xoshiro::test_utils::make_message(seed, length);
        minicbor::to_vec(ByteVec::from(message)).unwrap()
    }

    #[test]
    fn test_single_part_ur() {
        let ur = make_message_ur(50, "Wolf");
        let encoded = encode(&ur, &Type::Bytes);
        let expected = "ur:bytes/hdeymejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtgwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsdwkbrkch";
        assert_eq!(encoded, expected);
        let decoded = decode(&encoded).unwrap();
        assert_eq!((Kind::SinglePart, ur), decoded);
    }

    #[test]
    fn test_ur_encoder() {
        let ur = make_message_ur(256, "Wolf");
        let mut encoder = Encoder::bytes(&ur, 30).unwrap();
        let expected = vec![
            "ur:bytes/1-9/lpadascfadaxcywenbpljkhdcahkadaemejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtdkgslpgh",
            "ur:bytes/2-9/lpaoascfadaxcywenbpljkhdcagwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsgmghhkhstlrdcxaefz",
            "ur:bytes/3-9/lpaxascfadaxcywenbpljkhdcahelbknlkuejnbadmssfhfrdpsbiegecpasvssovlgeykssjykklronvsjksopdzmol",
            "ur:bytes/4-9/lpaaascfadaxcywenbpljkhdcasotkhemthydawydtaxneurlkosgwcekonertkbrlwmplssjtammdplolsbrdzcrtas",
            "ur:bytes/5-9/lpahascfadaxcywenbpljkhdcatbbdfmssrkzmcwnezelennjpfzbgmuktrhtejscktelgfpdlrkfyfwdajldejokbwf",
            "ur:bytes/6-9/lpamascfadaxcywenbpljkhdcackjlhkhybssklbwefectpfnbbectrljectpavyrolkzczcpkmwidmwoxkilghdsowp",
            "ur:bytes/7-9/lpatascfadaxcywenbpljkhdcavszmwnjkwtclrtvaynhpahrtoxmwvwatmedibkaegdosftvandiodagdhthtrlnnhy",
            "ur:bytes/8-9/lpayascfadaxcywenbpljkhdcadmsponkkbbhgsoltjntegepmttmoonftnbuoiyrehfrtsabzsttorodklubbuyaetk",
            "ur:bytes/9-9/lpasascfadaxcywenbpljkhdcajskecpmdckihdyhphfotjojtfmlnwmadspaxrkytbztpbauotbgtgtaeaevtgavtny",
            "ur:bytes/10-9/lpbkascfadaxcywenbpljkhdcahkadaemejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtwdkiplzs",
            "ur:bytes/11-9/lpbdascfadaxcywenbpljkhdcahelbknlkuejnbadmssfhfrdpsbiegecpasvssovlgeykssjykklronvsjkvetiiapk",
            "ur:bytes/12-9/lpbnascfadaxcywenbpljkhdcarllaluzmdmgstospeyiefmwejlwtpedamktksrvlcygmzemovovllarodtmtbnptrs",
            "ur:bytes/13-9/lpbtascfadaxcywenbpljkhdcamtkgtpknghchchyketwsvwgwfdhpgmgtylctotzopdrpayoschcmhplffziachrfgd",
            "ur:bytes/14-9/lpbaascfadaxcywenbpljkhdcapazewnvonnvdnsbyleynwtnsjkjndeoldydkbkdslgjkbbkortbelomueekgvstegt",
            "ur:bytes/15-9/lpbsascfadaxcywenbpljkhdcaynmhpddpzmversbdqdfyrehnqzlugmjzmnmtwmrouohtstgsbsahpawkditkckynwt",
            "ur:bytes/16-9/lpbeascfadaxcywenbpljkhdcawygekobamwtlihsnpalnsghenskkiynthdzotsimtojetprsttmukirlrsbtamjtpd",
            "ur:bytes/17-9/lpbyascfadaxcywenbpljkhdcamklgftaxykpewyrtqzhydntpnytyisincxmhtbceaykolduortotiaiaiafhiaoyce",
            "ur:bytes/18-9/lpbgascfadaxcywenbpljkhdcahkadaemejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtntwkbkwy",
            "ur:bytes/19-9/lpbwascfadaxcywenbpljkhdcadekicpaajootjzpsdrbalpeywllbdsnbinaerkurspbncxgslgftvtsrjtksplcpeo",
            "ur:bytes/20-9/lpbbascfadaxcywenbpljkhdcayapmrleeleaxpasfrtrdkncffwjyjzgyetdmlewtkpktgllepfrltataztksmhkbot",
        ];
        assert_eq!(encoder.fragment_count(), 9);
        for (index, e) in expected.into_iter().enumerate() {
            assert_eq!(encoder.current_index(), index);
            assert_eq!(encoder.next_part().unwrap(), e);
        }
    }

    #[test]
    fn test_ur_encoder_decoder_bc_crypto_request() {
        // https://github.com/BlockchainCommons/crypto-commons/blob/67ea252f4a7f295bb347cb046796d5b445b3ad3c/Docs/ur-99-request-response.md#the-seed-request

        fn crypto_seed() -> Result<Vec<u8>, minicbor::encode::Error<std::convert::Infallible>> {
            let mut e = minicbor::Encoder::new(Vec::new());

            let uuid = hex::decode("020C223A86F7464693FC650EF3CAC047").unwrap();
            let seed_digest =
                hex::decode("E824467CAFFEAF3BBC3E0CA095E660A9BAD80DDB6A919433A37161908B9A3986")
                    .unwrap();

            #[rustfmt::skip]
            e.map(2)?
                // 2.1 UUID: tag 37 type bytes(16)
                .u8(1)?.tag(Tag::Unassigned(37))?.bytes(&uuid)?
                // 2.2 crypto-seed: tag 500 type map
                .u8(2)?.tag(Tag::Unassigned(500))?.map(1)?
                // 2.2.1 crypto-seed-digest: tag 600 type bytes(32)
                .u8(1)?.tag(Tag::Unassigned(600))?.bytes(&seed_digest)?;

            Ok(e.into_writer())
        }

        let data = crypto_seed().unwrap();

        let e = encode(&data, &Type::Custom("crypto-request".into()));
        let expected = "ur:crypto-request/oeadtpdagdaobncpftlnylfgfgmuztihbawfsgrtflaotaadwkoyadtaaohdhdcxvsdkfgkepezepefrrffmbnnbmdvahnptrdtpbtuyimmemweootjshsmhlunyeslnameyhsdi";
        assert_eq!(expected, e);

        // Decoding should yield the same data
        let decoded = decode(e.as_str()).unwrap();
        assert_eq!((Kind::SinglePart, data), decoded);
    }

    #[test]
    fn test_multipart_ur() {
        let ur = make_message_ur(32767, "Wolf");
        let mut encoder = Encoder::bytes(&ur, 1000).unwrap();
        let mut decoder = Decoder::default();
        while !decoder.complete() {
            assert_eq!(decoder.message().unwrap(), None);
            decoder.receive(&encoder.next_part().unwrap()).unwrap();
        }
        assert_eq!(decoder.message().unwrap(), Some(ur));
    }

    #[test]
    fn test_decoder() {
        assert!(matches!(
            decode("uhr:bytes/aeadaolazmjendeoti"),
            Err(Error::InvalidScheme)
        ));
        assert!(matches!(
            decode("ur:aeadaolazmjendeoti"),
            Err(Error::TypeUnspecified)
        ));
        assert!(matches!(
            decode("ur:bytes#4/aeadaolazmjendeoti"),
            Err(Error::InvalidCharacters)
        ));
        assert!(matches!(
            decode("ur:bytes/1-1a/aeadaolazmjendeoti"),
            Err(Error::InvalidIndices)
        ));
        assert!(matches!(
            decode("ur:bytes/1-1/toomuch/aeadaolazmjendeoti"),
            Err(Error::InvalidIndices)
        ));
        decode("ur:bytes/aeadaolazmjendeoti").unwrap();
        decode("ur:whatever-12/aeadaolazmjendeoti").unwrap();
    }

    #[test]
    fn test_custom_encoder() {
        let data = String::from("Ten chars!");
        let max_length = 5;
        let mut encoder = Encoder::new(data.as_bytes(), max_length, "my-scheme").unwrap();
        assert_eq!(
            encoder.next_part().unwrap(),
            "ur:my-scheme/1-2/lpadaobkcywkwmhfwnfeghihjtcxiansvomopr"
        );
    }
}
