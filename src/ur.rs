pub struct Encoder {
    fountain: crate::fountain::Encoder,
}

impl Encoder {
    #[must_use]
    pub fn encode(ur: &[u8]) -> String {
        let body = crate::bytewords::encode(ur, &crate::bytewords::Style::Minimal);
        Self::encode_ur(&["bytes".into(), body])
    }

    #[must_use]
    fn encode_ur(items: &[String]) -> String {
        Self::encode_uri("ur", &items)
    }

    fn encode_uri(scheme: &str, items: &[String]) -> String {
        format!("{}:{}", scheme, items.join("/"))
    }

    #[must_use]
    pub fn new(message: &[u8], max_fragment_length: usize) -> Self {
        Self {
            fountain: crate::fountain::Encoder::new(message, max_fragment_length),
        }
    }

    pub fn next_part(&mut self) -> String {
        let part = self.fountain.next_part();
        let body = crate::bytewords::encode(&part.cbor(), &crate::bytewords::Style::Minimal);
        Self::encode_ur(&["bytes".into(), part.sequence_id(), body])
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
                        Some(idx) => &v[idx + 1..],
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
            .receive(crate::fountain::Part::from_cbor(decoded)?);
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

    fn make_message_ur(length: usize, seed: &str) -> Vec<u8> {
        let message = crate::xoshiro::test_utils::make_message(seed, length);
        let mut encoder = cbor::Encoder::from_memory();
        encoder
            .encode(vec![cbor::Cbor::Bytes(cbor::CborBytes(message))])
            .unwrap();
        encoder.as_bytes().to_vec()
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

    #[test]
    fn test_ur_encoder() {
        let ur = make_message_ur(256, "Wolf");
        let mut encoder = Encoder::new(&ur, 30);
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
        for e in expected {
            assert_eq!(encoder.next_part(), e);
        }
    }

    #[test]
    fn test_multipart_ur() {
        let ur = make_message_ur(32767, "Wolf");
        let mut encoder = Encoder::new(&ur, 1000);
        let mut decoder = Decoder::default();
        while !decoder.complete() {
            let part = encoder.next_part();
            decoder.receive(&part).unwrap();
        }
        assert_eq!(decoder.message().unwrap(), ur);
    }
}
