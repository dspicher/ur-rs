//! Encode and decode byte payloads according to the [`bytewords`](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-012-bytewords.md) scheme.
//!
//! The [`bytewords`](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-012-bytewords.md) encoding
//! scheme defines three styles how byte payloads can be encoded.
//!
//! # Standard style
//! ```
//! use ur::bytewords::{decode, encode, Style};
//! let data = "Some bytes".as_bytes();
//! let encoded = encode(data, Style::Standard);
//! assert_eq!(
//!     encoded,
//!     "guru jowl join inch crux iced kick jury inch junk taxi aqua kite limp"
//! );
//! assert_eq!(data, decode(&encoded, Style::Standard).unwrap());
//! ```
//!
//! # URI style
//! ```
//! use ur::bytewords::{decode, encode, Style};
//! let data = "Some bytes".as_bytes();
//! let encoded = encode(data, Style::Uri);
//! assert_eq!(
//!     encoded,
//!     "guru-jowl-join-inch-crux-iced-kick-jury-inch-junk-taxi-aqua-kite-limp"
//! );
//! assert_eq!(data, decode(&encoded, Style::Uri).unwrap());
//! ```
//!
//! # Minimal style
//! ```
//! use ur::bytewords::{decode, encode, Style};
//! let data = "Some binary data".as_bytes();
//! let encoded = encode(data, Style::Minimal);
//! assert_eq!(encoded, "gujljnihcxidinjthsjpkkcxiehsjyhsnsgdmkht");
//! assert_eq!(data, decode(&encoded, Style::Minimal).unwrap());
//! ```

/// The three different `bytewords` encoding styles. See the [`encode`] documentation for examples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Four-letter words, separated by spaces
    Standard,
    /// Four-letter words, separated by dashes
    Uri,
    /// Two-letter words, concatenated without separators
    Minimal,
}

/// The two different errors that can be returned when decoding.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Usually indicates a wrong encoding [`Style`] was passed.
    InvalidWord,
    /// The CRC32 checksum doesn't validate.
    InvalidChecksum,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidWord => write!(f, "invalid word"),
            Error::InvalidChecksum => write!(f, "invalid checksum"),
        }
    }
}

impl std::error::Error for Error {}

/// Decodes a `bytewords`-encoded String back into a byte payload. The encoding
/// must contain a four-byte checksum.
///
/// # Examples
///
/// ```
/// use ur::bytewords::{decode, Style};
/// assert_eq!(
///     decode("able tied also webs lung", Style::Standard).unwrap(),
///     vec![0]
/// );
/// assert_eq!(
///     decode("able-tied-also-webs-lung", Style::Uri).unwrap(),
///     vec![0]
/// );
/// // Notice how the minimal encoding consists of the start and end letters of the bytewords
/// assert_eq!(decode("aetdaowslg", Style::Minimal).unwrap(), vec![0]);
/// ```
///
/// # Errors
///
/// If the encoded string contains unrecognized words, is inconsistent with
/// the provided `style`, or contains an invalid checksum, an error will be
/// returned.
pub fn decode(encoded: &str, style: Style) -> Result<Vec<u8>, Error> {
    let separator = match style {
        Style::Standard => " ",
        Style::Uri => "-",
        Style::Minimal => return decode_minimal(encoded),
    };
    decode_from_index(&mut encoded.split(separator), &crate::constants::WORD_IDXS)
}

fn decode_minimal(encoded: &str) -> Result<Vec<u8>, Error> {
    decode_from_index(
        &mut (0..encoded.len())
            .step_by(2)
            .map(|idx| encoded.get(idx..idx + 2).unwrap()),
        &crate::constants::MINIMAL_IDXS,
    )
}

fn decode_from_index(
    keys: &mut dyn Iterator<Item = &str>,
    indexes: &phf::Map<&'static str, u8>,
) -> Result<Vec<u8>, Error> {
    strip_checksum(
        &keys
            .map(|k| indexes.get(k).copied())
            .collect::<Option<Vec<_>>>()
            .ok_or(Error::InvalidWord)?,
    )
}

fn strip_checksum(data: &[u8]) -> Result<Vec<u8>, Error> {
    if data.len() < 4 {
        return Err(Error::InvalidChecksum);
    }
    let (payload, checksum) = data.split_at(data.len() - 4);
    if crate::crc32().checksum(payload).to_be_bytes() == checksum {
        Ok(payload.to_vec())
    } else {
        Err(Error::InvalidChecksum)
    }
}

/// Encodes a byte payload into a `bytewords` encoded String.
///
/// # Examples
///
/// ```
/// use ur::bytewords::{encode, Style};
/// assert_eq!(encode(&[0], Style::Standard), "able tied also webs lung");
/// assert_eq!(encode(&[0], Style::Uri), "able-tied-also-webs-lung");
/// // Notice how the minimal encoding consists of the start and end letters of the bytewords
/// assert_eq!(encode(&[0], Style::Minimal), "aetdaowslg");
/// ```
#[must_use]
pub fn encode(data: &[u8], style: Style) -> String {
    let checksum = crate::crc32().checksum(data).to_be_bytes();
    let data = data.iter().chain(checksum.iter());
    let words: Vec<&str> = match style {
        Style::Standard | Style::Uri => data
            .map(|&b| crate::constants::WORDS.get(b as usize).copied().unwrap())
            .collect(),
        Style::Minimal => data
            .map(|&b| crate::constants::MINIMALS.get(b as usize).copied().unwrap())
            .collect(),
    };
    let separator = match style {
        Style::Standard => " ",
        Style::Uri => "-",
        Style::Minimal => "",
    };
    words.join(separator)
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
            encode(&input, Style::Standard),
            "able acid also lava zoom jade need echo taxi"
        );
        assert_eq!(
            encode(&input, Style::Uri),
            "able-acid-also-lava-zoom-jade-need-echo-taxi"
        );
        assert_eq!(encode(&input, Style::Minimal), "aeadaolazmjendeoti");

        assert_eq!(
            decode(
                "able acid also lava zoom jade need echo taxi",
                Style::Standard
            )
            .unwrap(),
            input
        );
        assert_eq!(
            decode("able-acid-also-lava-zoom-jade-need-echo-taxi", Style::Uri).unwrap(),
            input
        );
        assert_eq!(decode("aeadaolazmjendeoti", Style::Minimal).unwrap(), input);

        // empty payload is allowed
        decode(&encode(&[], Style::Minimal), Style::Minimal).unwrap();

        // bad checksum
        assert_eq!(
            decode(
                "able acid also lava zero jade need echo wolf",
                Style::Standard
            )
            .unwrap_err(),
            Error::InvalidChecksum
        );
        assert_eq!(
            decode("able-acid-also-lava-zero-jade-need-echo-wolf", Style::Uri).unwrap_err(),
            Error::InvalidChecksum
        );
        assert_eq!(
            decode("aeadaolazojendeowf", Style::Minimal).unwrap_err(),
            Error::InvalidChecksum
        );

        // too short
        assert_eq!(
            decode("wolf", Style::Standard).unwrap_err(),
            Error::InvalidChecksum
        );
        assert_eq!(decode("", Style::Standard).unwrap_err(), Error::InvalidWord);
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

        assert_eq!(decode(encoded, Style::Standard).unwrap(), input.to_vec());
        assert_eq!(
            decode(encoded_minimal, Style::Minimal).unwrap(),
            input.to_vec()
        );
        assert_eq!(encode(&input, Style::Standard), encoded);
        assert_eq!(encode(&input, Style::Minimal), encoded_minimal);
    }
}
