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
//! assert_eq!(decoder.message().unwrap(), data.as_bytes());
//! ```

/// Encodes a data payload into a single URI
///
/// # Examples
///
/// ```
/// assert_eq!(
///     ur::ur::encode("data".as_bytes(), "bytes"),
///     "ur:bytes/iehsjyhspmwfwfia"
/// );
/// ```
pub fn encode<T: Into<String>>(data: &[u8], ur_type: T) -> String {
    let body = crate::bytewords::encode(data, &crate::bytewords::Style::Minimal);
    encode_ur(&[ur_type.into(), body])
}

#[must_use]
fn encode_ur(items: &[String]) -> String {
    format!("{}:{}", "ur", items.join("/"))
}

/// A uniform resource encoder with an underlying fountain encoding.
///
/// # Examples
///
/// See the [`crate::ur`] module documentation for an example.
pub struct Encoder {
    fountain: crate::fountain::Encoder,
    ur_type: String,
}

impl Encoder {
    /// Creates a new [`Encoder`] for given a message payload.
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
    pub fn new<T: Into<String>>(
        message: &[u8],
        max_fragment_length: usize,
        ur_type: T,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            fountain: crate::fountain::Encoder::new(message, max_fragment_length)?,
            ur_type: ur_type.into(),
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
    pub fn next_part(&mut self) -> anyhow::Result<String> {
        let part = self.fountain.next_part();
        let body = crate::bytewords::encode(&part.cbor()?, &crate::bytewords::Style::Minimal);
        Ok(encode_ur(&[self.ur_type.clone(), part.sequence_id(), body]))
    }

    /// Returns the current count of already emitted parts.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut encoder = ur::Encoder::new("data".as_bytes(), 5, "bytes").unwrap();
    /// assert_eq!(encoder.current_index(), 0);
    /// encoder.next_part().unwrap();
    /// assert_eq!(encoder.current_index(), 1);
    /// ```
    #[must_use]
    pub fn current_index(&self) -> usize {
        self.fountain.current_sequence()
    }

    /// Returns the number of segments the original message has been split up into.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut encoder = ur::Encoder::new("data".as_bytes(), 3, "bytes").unwrap();
    /// assert_eq!(encoder.fragment_count(), 2);
    /// ```
    #[must_use]
    pub fn fragment_count(&self) -> usize {
        self.fountain.fragment_count()
    }
}

fn decode(value: &str) -> anyhow::Result<Vec<u8>> {
    match value.strip_prefix("ur:") {
        Some(v) => {
            let mut parts = v.rsplit('/');
            // rsplit will always return at least one item
            let payload = parts.next().unwrap();
            match parts.count() {
                0 => Err(anyhow::anyhow!("No type specified")),
                1 | 2 => Ok(crate::bytewords::decode(
                    payload,
                    &crate::bytewords::Style::Minimal,
                )?),
                _ => Err(anyhow::anyhow!("Invalid encoding: too many separators '/'")),
            }
        }
        None => Err(anyhow::anyhow!("Invalid Scheme")),
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
    pub fn receive(&mut self, value: &str) -> anyhow::Result<()> {
        let decoded = decode(value)?;
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

    /// If [`complete`], returns the decoded message.
    ///
    /// # Errors
    ///
    /// If the message is not completely decoded yet or an inconsisten
    /// internal state detected, an error will be returned.
    ///
    /// # Examples
    ///
    /// See the [`crate::ur`] module documentation for an example.
    ///
    /// [`complete`]: Decoder::complete
    pub fn message(&self) -> anyhow::Result<Vec<u8>> {
        self.fountain.message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_cbor::Value;
    use std::collections::BTreeMap;

    fn make_message_ur(length: usize, seed: &str) -> Vec<u8> {
        let message = crate::xoshiro::test_utils::make_message(seed, length);
        serde_cbor::to_vec(&Value::Bytes(message)).unwrap()
    }

    #[test]
    fn test_single_part_ur() {
        let ur = make_message_ur(50, "Wolf");
        let encoded = encode(&ur, "bytes");
        let expected = "ur:bytes/hdeymejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtgwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsdwkbrkch";
        assert_eq!(encoded, expected);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(ur, decoded);
    }

    #[test]
    fn test_ur_encoder() {
        let ur = make_message_ur(256, "Wolf");
        let mut encoder = Encoder::new(&ur, 30, "bytes").unwrap();
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

        // 2.1 UUID: tag 37 type bytes(16)
        let uuid_value = Value::Bytes(hex::decode("020C223A86F7464693FC650EF3CAC047").unwrap());
        let uuid = Value::Tag(37, Box::new(uuid_value));

        // 2.2.1 crypto-seed-digest: tag 600 type bytes(32)
        let crypto_seed_digest_value = Value::Bytes(
            hex::decode("E824467CAFFEAF3BBC3E0CA095E660A9BAD80DDB6A919433A37161908B9A3986")
                .unwrap(),
        );
        let crypto_seed_digest = Value::Tag(600, Box::new(crypto_seed_digest_value));

        // 2.2 crypto-seed: tag 500 type map
        let mut crypto_seed_map = BTreeMap::new();
        crypto_seed_map.insert(Value::Integer(1), crypto_seed_digest);
        let crypto_seed_value = Value::Map(crypto_seed_map);
        let crypto_seed = Value::Tag(500, Box::new(crypto_seed_value));

        // 1. Top level is a map
        let mut top_level_map = BTreeMap::new();
        top_level_map.insert(Value::Integer(1), uuid);
        top_level_map.insert(Value::Integer(2), crypto_seed);
        let top_level = Value::Map(top_level_map);

        let data = serde_cbor::to_vec(&top_level).unwrap();

        let e = encode(&data, "crypto-request");
        let expected = "ur:crypto-request/oeadtpdagdaobncpftlnylfgfgmuztihbawfsgrtflaotaadwkoyadtaaohdhdcxvsdkfgkepezepefrrffmbnnbmdvahnptrdtpbtuyimmemweootjshsmhlunyeslnameyhsdi";
        assert_eq!(expected, e);

        // Decoding should yield the same data
        let d = decode(e.as_str()).unwrap();
        assert_eq!(data, d);
    }

    #[test]
    fn test_multipart_ur() {
        let ur = make_message_ur(32767, "Wolf");
        let mut encoder = Encoder::new(&ur, 1000, "bytes").unwrap();
        let mut decoder = Decoder::default();
        while !decoder.complete() {
            decoder.receive(&encoder.next_part().unwrap()).unwrap();
        }
        assert_eq!(decoder.message().unwrap(), ur);
    }

    #[test]
    fn test_decoder() {
        assert_eq!(
            decode("uhr:bytes/aeadaolazmjendeoti")
                .unwrap_err()
                .to_string(),
            "Invalid Scheme"
        );
        assert_eq!(
            decode("ur:aeadaolazmjendeoti").unwrap_err().to_string(),
            "No type specified"
        );
        assert_eq!(
            decode("ur:bytes/seq/toomuch/aeadaolazmjendeoti")
                .unwrap_err()
                .to_string(),
            "Invalid encoding: too many separators '/'"
        );
        decode("ur:bytes/aeadaolazmjendeoti").unwrap();
        decode("ur:whatever/aeadaolazmjendeoti").unwrap();
    }
}
