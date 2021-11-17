pub struct Encoder {
    fountain: crate::fountain::Encoder,
    ur_type: String,
}

impl Encoder {
    pub fn encode<T: Into<String>>(data: &[u8], ur_type: T) -> anyhow::Result<String> {
        let body = crate::bytewords::encode(data, &crate::bytewords::Style::Minimal)?;
        Ok(Self::encode_ur(&[ur_type.into(), body]))
    }

    #[must_use]
    fn encode_ur(items: &[String]) -> String {
        Self::encode_uri("ur", items)
    }

    fn encode_uri(scheme: &str, items: &[String]) -> String {
        format!("{}:{}", scheme, items.join("/"))
    }

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

    pub fn next_part(&mut self) -> anyhow::Result<String> {
        let part = self.fountain.next_part()?;
        let body = crate::bytewords::encode(&part.cbor()?, &crate::bytewords::Style::Minimal)?;
        Ok(Self::encode_ur(&[
            self.ur_type.clone(),
            part.sequence_id(),
            body,
        ]))
    }

    #[must_use]
    pub fn current_index(&self) -> usize {
        self.fountain.current_sequence()
    }
}

#[derive(std::default::Default)]
pub struct Decoder {
    fountain: crate::fountain::Decoder,
}

impl Decoder {
    pub fn decode(value: &str) -> anyhow::Result<Vec<u8>> {
        match value.strip_prefix("ur:") {
            Some(val) => match val.strip_prefix("bytes/") {
                Some(v) => Ok(crate::bytewords::decode(
                    match v.find('/') {
                        None => v,
                        Some(idx) => v
                            .get(idx + 1..)
                            .ok_or_else(|| anyhow::anyhow!("expected items"))?,
                    },
                    &crate::bytewords::Style::Minimal,
                )?),
                None => Err(anyhow::anyhow!("Invalid type")),
            },
            None => Err(anyhow::anyhow!("Invalid Scheme")),
        }
    }

    pub fn receive(&mut self, value: &str) -> anyhow::Result<()> {
        let decoded = Self::decode(value)?;
        self.fountain
            .receive(crate::fountain::Part::from_cbor(decoded.as_slice())?)?;
        Ok(())
    }

    #[must_use]
    pub fn complete(&self) -> bool {
        self.fountain.complete()
    }

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
        let encoded = Encoder::encode(&ur, "bytes").unwrap();
        let expected = "ur:bytes/hdeymejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtgwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsdwkbrkch";
        assert_eq!(encoded, expected);
        let decoded = Decoder::decode(&encoded).unwrap();
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
        for (index, e) in expected.into_iter().enumerate() {
            assert_eq!(encoder.current_index(), index);
            assert_eq!(encoder.next_part().unwrap(), e);
        }
    }

    #[test]
    fn test_ur_encoder_bc_crypto_request() {
        // https://github.com/BlockchainCommons/crypto-commons/blob/67ea252f4a7f295bb347cb046796d5b445b3ad3c/Docs/ur-99-request-response.md

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

        let e = Encoder::encode(&data, "crypto-request").unwrap();
        let expected = "ur:crypto-request/oeadtpdagdaobncpftlnylfgfgmuztihbawfsgrtflaotaadwkoyadtaaohdhdcxvsdkfgkepezepefrrffmbnnbmdvahnptrdtpbtuyimmemweootjshsmhlunyeslnameyhsdi";
        assert_eq!(expected, e);
    }

    #[test]
    fn test_multipart_ur() {
        let ur = make_message_ur(32767, "Wolf");
        let mut encoder = Encoder::new(&ur, 1000, "bytes").unwrap();
        let mut decoder = Decoder::default();
        while !decoder.complete() {
            let part = encoder.next_part().unwrap();
            decoder.receive(&part).unwrap();
        }
        assert_eq!(decoder.message().unwrap(), ur);
    }

    #[test]
    fn test_decoder() {
        assert_eq!(
            Decoder::decode("uhr:bytes/aeadaolazmjendeoti")
                .unwrap_err()
                .to_string(),
            "Invalid Scheme"
        );
        assert_eq!(
            Decoder::decode("ur:byts/aeadaolazmjendeoti")
                .unwrap_err()
                .to_string(),
            "Invalid type"
        );
        Decoder::decode("ur:bytes/aeadaolazmjendeoti").unwrap();
    }
}
