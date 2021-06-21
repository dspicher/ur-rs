#[derive(PartialEq)]
pub enum Style {
    Standard,
    Uri,
    Minimal,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidWord,
    InvalidChecksum,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::InvalidWord => "invalid word",
                Error::InvalidChecksum => "invalid checksum",
            }
        )
    }
}

impl std::error::Error for Error {}

pub fn decode(encoded: &str, style: &Style) -> Result<Vec<u8>, Error> {
    let separator = match style {
        Style::Standard => " ",
        Style::Uri => "-",
        Style::Minimal => return decode_minimal(encoded),
    };
    let mut data = vec![];
    for word in encoded.split(separator) {
        match crate::constants::WORD_IDXS.get(word) {
            Some(idx) => data.push(*idx),
            None => return Err(Error::InvalidWord),
        }
    }
    strip_checksum(&data)
}

fn decode_minimal(encoded: &str) -> Result<Vec<u8>, Error> {
    let mut data = vec![];
    for idx in (0..encoded.len()).step_by(2) {
        let substr = encoded.get(idx..idx + 2).unwrap();
        match crate::constants::MINIMAL_IDXS.get(substr) {
            Some(idx) => data.push(*idx),
            None => return Err(Error::InvalidWord),
        }
    }
    strip_checksum(&data)
}

fn strip_checksum(data: &[u8]) -> Result<Vec<u8>, Error> {
    if data.len() < 4 {
        return Err(Error::InvalidChecksum);
    }
    match (
        data.get(..data.len() - 4),
        data.get(data.len() - 4..data.len()),
    ) {
        (Some(payload), Some(checksum)) => {
            if crate::crc32().checksum(payload).to_be_bytes() == checksum {
                Ok(payload.to_vec())
            } else {
                Err(Error::InvalidChecksum)
            }
        }
        _ => Err(Error::InvalidChecksum),
    }
}

pub fn encode(data: &[u8], style: &Style) -> anyhow::Result<String> {
    let checksum = crate::crc32().checksum(data).to_be_bytes();
    let data = data.iter().chain(checksum.iter());
    let words: Vec<&str> = match style {
        Style::Standard | Style::Uri => data
            .map(|b| {
                crate::constants::WORDS
                    .get(*b as usize)
                    .copied()
                    .ok_or_else(|| anyhow::anyhow!("expected item"))
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        Style::Minimal => data
            .map(|b| {
                crate::constants::MINIMALS
                    .get(*b as usize)
                    .copied()
                    .ok_or_else(|| anyhow::anyhow!("expected item"))
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
    };
    let separator = match style {
        Style::Standard => " ",
        Style::Uri => "-",
        Style::Minimal => "",
    };
    Ok(words.join(separator))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc() {
        assert_eq!(crate::crc32().checksum(b"Hello, world!"), 0xebe6_c6e6);
        assert_eq!(crate::crc32().checksum(b"Wolf"), 0x598c_84dc);
    }

    #[test]
    fn test_bytewords() {
        let input = vec![0, 1, 2, 128, 255];
        assert_eq!(
            encode(&input, &Style::Standard).unwrap(),
            "able acid also lava zoom jade need echo taxi"
        );
        assert_eq!(
            encode(&input, &Style::Uri).unwrap(),
            "able-acid-also-lava-zoom-jade-need-echo-taxi"
        );
        assert_eq!(
            encode(&input, &Style::Minimal).unwrap(),
            "aeadaolazmjendeoti"
        );

        assert_eq!(
            decode(
                "able acid also lava zoom jade need echo taxi",
                &Style::Standard
            )
            .unwrap(),
            input
        );
        assert_eq!(
            decode("able-acid-also-lava-zoom-jade-need-echo-taxi", &Style::Uri).unwrap(),
            input
        );
        assert_eq!(
            decode("aeadaolazmjendeoti", &Style::Minimal).unwrap(),
            input
        );

        // bad checksum
        assert_eq!(
            decode(
                "able acid also lava zero jade need echo wolf",
                &Style::Standard
            )
            .unwrap_err(),
            Error::InvalidChecksum
        );
        assert_eq!(
            decode("able-acid-also-lava-zero-jade-need-echo-wolf", &Style::Uri).unwrap_err(),
            Error::InvalidChecksum
        );
        assert_eq!(
            decode("aeadaolazojendeowf", &Style::Minimal).unwrap_err(),
            Error::InvalidChecksum
        );

        // too short
        assert_eq!(
            decode("wolf", &Style::Standard).unwrap_err(),
            Error::InvalidChecksum
        );
        assert_eq!(
            decode("", &Style::Standard).unwrap_err(),
            Error::InvalidWord
        );
    }

    #[test]
    fn test_encoding() {
        let input: [u8; 100] = [
            245, 215, 20, 198, 241, 235, 69, 59, 209, 205, 165, 18, 150, 158, 116, 135, 229, 212,
            19, 159, 17, 37, 239, 240, 253, 11, 109, 191, 37, 242, 38, 120, 223, 41, 156, 189, 242,
            254, 147, 204, 66, 163, 216, 175, 191, 72, 169, 54, 32, 60, 144, 230, 210, 137, 184,
            197, 33, 113, 88, 14, 157, 31, 177, 46, 1, 115, 205, 69, 225, 150, 65, 235, 58, 144,
            65, 240, 133, 69, 113, 247, 63, 53, 242, 165, 160, 144, 26, 13, 79, 237, 133, 71, 82,
            69, 254, 165, 138, 41, 85, 24,
        ];

        let encoded = "yank toys bulb skew when warm free fair tent swan \
                       open brag mint noon jury list view tiny brew note \
                       body data webs what zinc bald join runs data whiz \
                       days keys user diet news ruby whiz zone menu surf \
                       flew omit trip pose runs fund part even crux fern \
                       math visa tied loud redo silk curl jugs hard beta \
                       next cost puma drum acid junk swan free very mint \
                       flap warm fact math flap what limp free jugs yell \
                       fish epic whiz open numb math city belt glow wave \
                       limp fuel grim free zone open love diet gyro cats \
                       fizz holy city puff";

        let encoded_minimal = "yktsbbswwnwmfefrttsnonbgmtnnjyltvwtybwne\
                               bydawswtzcbdjnrsdawzdsksurdtnsrywzzemusf\
                               fwottppersfdptencxfnmhvatdldroskcljshdba\
                               ntctpadmadjksnfevymtfpwmftmhfpwtlpfejsyl\
                               fhecwzonnbmhcybtgwwelpflgmfezeonledtgocs\
                               fzhycypf";

        assert_eq!(decode(encoded, &Style::Standard).unwrap(), input.to_vec());
        assert_eq!(
            decode(encoded_minimal, &Style::Minimal).unwrap(),
            input.to_vec()
        );
        assert_eq!(encode(&input, &Style::Standard).unwrap(), encoded);
        assert_eq!(encode(&input, &Style::Minimal).unwrap(), encoded_minimal);
    }
}
