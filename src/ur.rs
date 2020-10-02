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

pub struct Decoder {}

impl Decoder {
    pub fn decode(value: &str) -> anyhow::Result<Vec<u8>> {
        match value.strip_prefix("ur:") {
            Some(val) => match val.strip_prefix("bytes/") {
                Some(v) => Ok(crate::bytewords::decode(
                    v,
                    &crate::bytewords::Style::Minimal,
                )?),
                None => Err(anyhow::anyhow!("Invalid type")),
            },
            None => Err(anyhow::anyhow!("Invalid Scheme")),
        }
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
            "ur:bytes/1-9/ltadascfadaxcywenbpljkhdcahkadaemejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtdkgsltgh",
            "ur:bytes/2-9/ltaoascfadaxcywenbpljkhdcagwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsgmghhkhstlrdcxaefz",
            "ur:bytes/3-9/ltaxascfadaxcywenbpljkhdcahelbknlkuejnbadmssfhfrdpsbiegecpasvssovlgeykssjykklronvsjksopdzool",
            "ur:bytes/4-9/ltaaascfadaxcywenbpljkhdcasotkhemthydawydtaxneurlkosgwcekonertkbrlwmplssjtammdplolsbrdzertas",
            "ur:bytes/5-9/ltahascfadaxcywenbpljkhdcatbbdfmssrkzocwnezmlennjpfzbgmuktrhtejscktelgfpdlrkfyfwdajldejokbwf",
            "ur:bytes/6-9/ltamascfadaxcywenbpljkhdcackjlhkhybssklbwefectpfnbbectrljectpavyrolkzezepkmwidmwoxkilghdsowp",
            "ur:bytes/7-9/ltatascfadaxcywenbpljkhdcavszownjkwtclrtvaynhpahrtoxmwvwatmedibkaegdosftvandiodagdhthtrlnnhy",
            "ur:bytes/8-9/ltayascfadaxcywenbpljkhdcadmsponkkbbhgsolnjntegepmttmoonftnbuoiyrehfrtsabzsttorodklubbuyaetk",
            "ur:bytes/9-9/ltasascfadaxcywenbpljkhdcajskecpmdckihdyhphfotjojtfmlpwmadspaxrkytbztpbauotbgtgtaeaevtgavtny",
            "ur:bytes/10-9/ltbkascfadaxcywenbpljkhdcahkadaemejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtwdkiplzs",
            "ur:bytes/11-9/ltbdascfadaxcywenbpljkhdcahelbknlkuejnbadmssfhfrdpsbiegecpasvssovlgeykssjykklronvsjkvetiiapk",
            "ur:bytes/12-9/ltbnascfadaxcywenbpljkhdcarllaluzodmgstospeyiefmwejlwtpedamktksrvlcygmzmmovovllarodtmtbnptrs",
            "ur:bytes/13-9/ltbtascfadaxcywenbpljkhdcamtkgtpknghchchyketwsvwgwfdhpgmgtylctotztpdrpayoschcmhplffziachrfgd",
            "ur:bytes/14-9/ltbaascfadaxcywenbpljkhdcapazmwnvonnvdnsbyleynwtnsjkjndeoldydkbkdslgjkbbkortbelomueekgvstegt",
            "ur:bytes/15-9/ltbsascfadaxcywenbpljkhdcaynmhpddpzoversbdqdfyrehnqzlugmjzmnmtwmrouohtstgsbsahpawkditkckynwt",
            "ur:bytes/16-9/ltbeascfadaxcywenbpljkhdcawygekobamwtlihsnpalpsghenskkiynthdzttsimtojetprsttmukirlrsbtamjtpd",
            "ur:bytes/17-9/ltbyascfadaxcywenbpljkhdcamklgftaxykpewyrtqzhydntpnytyisincxmhtbceaykolduortotiaiaiafhiaoyce",
            "ur:bytes/18-9/ltbgascfadaxcywenbpljkhdcahkadaemejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtntwkbkwy",
            "ur:bytes/19-9/ltbwascfadaxcywenbpljkhdcadekicpaajootjzpsdrbalteywllbdsnbinaerkurspbncxgslgftvtsrjtksplcpeo",
            "ur:bytes/20-9/ltbbascfadaxcywenbpljkhdcayapmrleeleaxpasfrtrdkncffwjyjzgyetdmlewtkpktgllepfrltatazcksmhkbot"
        ];
        for e in expected {
            dbg!(e);
            assert_eq!(encoder.next_part(), e);
        }
    }
}
