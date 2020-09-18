#[derive(Debug, PartialEq)]
pub struct UR {
    r#type: String,
    cbor: Vec<u8>,
}

impl UR {
    #[must_use]
    pub fn cbor(&self) -> &[u8] {
        &self.cbor
    }
}

pub struct Encoder {}

impl Encoder {
    #[must_use]
    pub fn encode(ur: &UR) -> String {
        let body = crate::bytewords::encode(ur.cbor(), &crate::bytewords::Style::Minimal);
        Self::encode_ur(&[ur.r#type.clone(), body])
    }

    #[must_use]
    fn encode_ur(items: &[String]) -> String {
        Self::encode_uri("ur", &items)
    }

    fn encode_uri(scheme: &str, items: &[String]) -> String {
        format!("{}:{}", scheme, items.join("/"))
    }
}

pub struct Decoder {}

impl Decoder {
    pub fn decode(value: &str) -> anyhow::Result<UR> {
        match value.strip_prefix("ur:") {
            Some(val) => match val.strip_prefix("bytes/") {
                Some(v) => Ok(UR {
                    r#type: "bytes".into(),
                    cbor: crate::bytewords::decode(v, &crate::bytewords::Style::Minimal)?,
                }),
                None => Err(anyhow::anyhow!("Invalid type")),
            },
            None => Err(anyhow::anyhow!("Invalid Scheme")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_message_ur(length: usize, seed: &str) -> UR {
        let message = crate::xoshiro::test_utils::make_message(seed, length);
        let mut encoder = cbor::Encoder::from_memory();
        encoder
            .encode(vec![cbor::Cbor::Bytes(cbor::CborBytes(message))])
            .unwrap();
        UR {
            r#type: "bytes".into(),
            cbor: encoder.as_bytes().to_vec(),
        }
    }

    #[test]
    fn test_single_part_ur() {
        let ur = make_message_ur(50, "Wolf");
        let encoded = Encoder::encode(&ur);
        let expected = "ur:bytes/hdeymejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtgwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsdwkbrkch";
        assert_eq!(encoded, expected);
        let decoded = Decoder::decode(&encoded).unwrap();
        assert_eq!(ur, decoded);
    }
}
