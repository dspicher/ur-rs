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

extern crate alloc;
use alloc::vec::Vec;

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
    /// Invalid bytewords string length.
    InvalidLength,
    /// The bytewords string contains non-ASCII characters.
    NonAscii,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidWord => write!(f, "invalid word"),
            Self::InvalidChecksum => write!(f, "invalid checksum"),
            Self::InvalidLength => write!(f, "invalid length"),
            Self::NonAscii => write!(f, "bytewords string contains non-ASCII characters"),
        }
    }
}

#[cfg(feature = "std")]
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
    if !encoded.is_ascii() {
        return Err(Error::NonAscii);
    }

    let separator = match style {
        Style::Standard => ' ',
        Style::Uri => '-',
        Style::Minimal => return decode_minimal(encoded),
    };
    decode_from_index(&mut encoded.split(separator), false)
}

fn decode_minimal(encoded: &str) -> Result<Vec<u8>, Error> {
    if encoded.len() % 2 != 0 {
        return Err(Error::InvalidLength);
    }

    decode_from_index(
        &mut (0..encoded.len())
            .step_by(2)
            .map(|idx| encoded.get(idx..idx + 2).unwrap()),
        true,
    )
}

#[allow(clippy::too_many_lines)]
fn decode_from_index(
    keys: &mut dyn Iterator<Item = &str>,
    minimal: bool,
) -> Result<Vec<u8>, Error> {
    strip_checksum(
        keys.map(|k| {
            if minimal {
                match k {
                    "ae" => Some(0),
                    "ad" => Some(1),
                    "ao" => Some(2),
                    "ax" => Some(3),
                    "aa" => Some(4),
                    "ah" => Some(5),
                    "am" => Some(6),
                    "at" => Some(7),
                    "ay" => Some(8),
                    "as" => Some(9),
                    "bk" => Some(10),
                    "bd" => Some(11),
                    "bn" => Some(12),
                    "bt" => Some(13),
                    "ba" => Some(14),
                    "bs" => Some(15),
                    "be" => Some(16),
                    "by" => Some(17),
                    "bg" => Some(18),
                    "bw" => Some(19),
                    "bb" => Some(20),
                    "bz" => Some(21),
                    "cm" => Some(22),
                    "ch" => Some(23),
                    "cs" => Some(24),
                    "cf" => Some(25),
                    "cy" => Some(26),
                    "cw" => Some(27),
                    "ce" => Some(28),
                    "ca" => Some(29),
                    "ck" => Some(30),
                    "ct" => Some(31),
                    "cx" => Some(32),
                    "cl" => Some(33),
                    "cp" => Some(34),
                    "cn" => Some(35),
                    "dk" => Some(36),
                    "da" => Some(37),
                    "ds" => Some(38),
                    "di" => Some(39),
                    "de" => Some(40),
                    "dt" => Some(41),
                    "dr" => Some(42),
                    "dn" => Some(43),
                    "dw" => Some(44),
                    "dp" => Some(45),
                    "dm" => Some(46),
                    "dl" => Some(47),
                    "dy" => Some(48),
                    "eh" => Some(49),
                    "ey" => Some(50),
                    "eo" => Some(51),
                    "ee" => Some(52),
                    "ec" => Some(53),
                    "en" => Some(54),
                    "em" => Some(55),
                    "et" => Some(56),
                    "es" => Some(57),
                    "ft" => Some(58),
                    "fr" => Some(59),
                    "fn" => Some(60),
                    "fs" => Some(61),
                    "fm" => Some(62),
                    "fh" => Some(63),
                    "fz" => Some(64),
                    "fp" => Some(65),
                    "fw" => Some(66),
                    "fx" => Some(67),
                    "fy" => Some(68),
                    "fe" => Some(69),
                    "fg" => Some(70),
                    "fl" => Some(71),
                    "fd" => Some(72),
                    "ga" => Some(73),
                    "ge" => Some(74),
                    "gr" => Some(75),
                    "gs" => Some(76),
                    "gt" => Some(77),
                    "gl" => Some(78),
                    "gw" => Some(79),
                    "gd" => Some(80),
                    "gy" => Some(81),
                    "gm" => Some(82),
                    "gu" => Some(83),
                    "gh" => Some(84),
                    "go" => Some(85),
                    "hf" => Some(86),
                    "hg" => Some(87),
                    "hd" => Some(88),
                    "hk" => Some(89),
                    "ht" => Some(90),
                    "hp" => Some(91),
                    "hh" => Some(92),
                    "hl" => Some(93),
                    "hy" => Some(94),
                    "he" => Some(95),
                    "hn" => Some(96),
                    "hs" => Some(97),
                    "id" => Some(98),
                    "ia" => Some(99),
                    "ie" => Some(100),
                    "ih" => Some(101),
                    "iy" => Some(102),
                    "io" => Some(103),
                    "is" => Some(104),
                    "in" => Some(105),
                    "im" => Some(106),
                    "je" => Some(107),
                    "jz" => Some(108),
                    "jn" => Some(109),
                    "jt" => Some(110),
                    "jl" => Some(111),
                    "jo" => Some(112),
                    "js" => Some(113),
                    "jp" => Some(114),
                    "jk" => Some(115),
                    "jy" => Some(116),
                    "kp" => Some(117),
                    "ko" => Some(118),
                    "kt" => Some(119),
                    "ks" => Some(120),
                    "kk" => Some(121),
                    "kn" => Some(122),
                    "kg" => Some(123),
                    "ke" => Some(124),
                    "ki" => Some(125),
                    "kb" => Some(126),
                    "lb" => Some(127),
                    "la" => Some(128),
                    "ly" => Some(129),
                    "lf" => Some(130),
                    "ls" => Some(131),
                    "lr" => Some(132),
                    "lp" => Some(133),
                    "ln" => Some(134),
                    "lt" => Some(135),
                    "lo" => Some(136),
                    "ld" => Some(137),
                    "le" => Some(138),
                    "lu" => Some(139),
                    "lk" => Some(140),
                    "lg" => Some(141),
                    "mn" => Some(142),
                    "my" => Some(143),
                    "mh" => Some(144),
                    "me" => Some(145),
                    "mo" => Some(146),
                    "mu" => Some(147),
                    "mw" => Some(148),
                    "md" => Some(149),
                    "mt" => Some(150),
                    "ms" => Some(151),
                    "mk" => Some(152),
                    "nl" => Some(153),
                    "ny" => Some(154),
                    "nd" => Some(155),
                    "ns" => Some(156),
                    "nt" => Some(157),
                    "nn" => Some(158),
                    "ne" => Some(159),
                    "nb" => Some(160),
                    "oy" => Some(161),
                    "oe" => Some(162),
                    "ot" => Some(163),
                    "ox" => Some(164),
                    "on" => Some(165),
                    "ol" => Some(166),
                    "os" => Some(167),
                    "pd" => Some(168),
                    "pt" => Some(169),
                    "pk" => Some(170),
                    "py" => Some(171),
                    "ps" => Some(172),
                    "pm" => Some(173),
                    "pl" => Some(174),
                    "pe" => Some(175),
                    "pf" => Some(176),
                    "pa" => Some(177),
                    "pr" => Some(178),
                    "qd" => Some(179),
                    "qz" => Some(180),
                    "re" => Some(181),
                    "rp" => Some(182),
                    "rl" => Some(183),
                    "ro" => Some(184),
                    "rh" => Some(185),
                    "rd" => Some(186),
                    "rk" => Some(187),
                    "rf" => Some(188),
                    "ry" => Some(189),
                    "rn" => Some(190),
                    "rs" => Some(191),
                    "rt" => Some(192),
                    "se" => Some(193),
                    "sa" => Some(194),
                    "sr" => Some(195),
                    "ss" => Some(196),
                    "sk" => Some(197),
                    "sw" => Some(198),
                    "st" => Some(199),
                    "sp" => Some(200),
                    "so" => Some(201),
                    "sg" => Some(202),
                    "sb" => Some(203),
                    "sf" => Some(204),
                    "sn" => Some(205),
                    "to" => Some(206),
                    "tk" => Some(207),
                    "ti" => Some(208),
                    "tt" => Some(209),
                    "td" => Some(210),
                    "te" => Some(211),
                    "ty" => Some(212),
                    "tl" => Some(213),
                    "tb" => Some(214),
                    "ts" => Some(215),
                    "tp" => Some(216),
                    "ta" => Some(217),
                    "tn" => Some(218),
                    "uy" => Some(219),
                    "uo" => Some(220),
                    "ut" => Some(221),
                    "ue" => Some(222),
                    "ur" => Some(223),
                    "vt" => Some(224),
                    "vy" => Some(225),
                    "vo" => Some(226),
                    "vl" => Some(227),
                    "ve" => Some(228),
                    "vw" => Some(229),
                    "va" => Some(230),
                    "vd" => Some(231),
                    "vs" => Some(232),
                    "wl" => Some(233),
                    "wd" => Some(234),
                    "wm" => Some(235),
                    "wp" => Some(236),
                    "we" => Some(237),
                    "wy" => Some(238),
                    "ws" => Some(239),
                    "wt" => Some(240),
                    "wn" => Some(241),
                    "wz" => Some(242),
                    "wf" => Some(243),
                    "wk" => Some(244),
                    "yk" => Some(245),
                    "yn" => Some(246),
                    "yl" => Some(247),
                    "ya" => Some(248),
                    "yt" => Some(249),
                    "zs" => Some(250),
                    "zo" => Some(251),
                    "zt" => Some(252),
                    "zc" => Some(253),
                    "ze" => Some(254),
                    "zm" => Some(255),
                    _ => None,
                }
            } else {
                match k {
                    "able" => Some(0),
                    "acid" => Some(1),
                    "also" => Some(2),
                    "apex" => Some(3),
                    "aqua" => Some(4),
                    "arch" => Some(5),
                    "atom" => Some(6),
                    "aunt" => Some(7),
                    "away" => Some(8),
                    "axis" => Some(9),
                    "back" => Some(10),
                    "bald" => Some(11),
                    "barn" => Some(12),
                    "belt" => Some(13),
                    "beta" => Some(14),
                    "bias" => Some(15),
                    "blue" => Some(16),
                    "body" => Some(17),
                    "brag" => Some(18),
                    "brew" => Some(19),
                    "bulb" => Some(20),
                    "buzz" => Some(21),
                    "calm" => Some(22),
                    "cash" => Some(23),
                    "cats" => Some(24),
                    "chef" => Some(25),
                    "city" => Some(26),
                    "claw" => Some(27),
                    "code" => Some(28),
                    "cola" => Some(29),
                    "cook" => Some(30),
                    "cost" => Some(31),
                    "crux" => Some(32),
                    "curl" => Some(33),
                    "cusp" => Some(34),
                    "cyan" => Some(35),
                    "dark" => Some(36),
                    "data" => Some(37),
                    "days" => Some(38),
                    "deli" => Some(39),
                    "dice" => Some(40),
                    "diet" => Some(41),
                    "door" => Some(42),
                    "down" => Some(43),
                    "draw" => Some(44),
                    "drop" => Some(45),
                    "drum" => Some(46),
                    "dull" => Some(47),
                    "duty" => Some(48),
                    "each" => Some(49),
                    "easy" => Some(50),
                    "echo" => Some(51),
                    "edge" => Some(52),
                    "epic" => Some(53),
                    "even" => Some(54),
                    "exam" => Some(55),
                    "exit" => Some(56),
                    "eyes" => Some(57),
                    "fact" => Some(58),
                    "fair" => Some(59),
                    "fern" => Some(60),
                    "figs" => Some(61),
                    "film" => Some(62),
                    "fish" => Some(63),
                    "fizz" => Some(64),
                    "flap" => Some(65),
                    "flew" => Some(66),
                    "flux" => Some(67),
                    "foxy" => Some(68),
                    "free" => Some(69),
                    "frog" => Some(70),
                    "fuel" => Some(71),
                    "fund" => Some(72),
                    "gala" => Some(73),
                    "game" => Some(74),
                    "gear" => Some(75),
                    "gems" => Some(76),
                    "gift" => Some(77),
                    "girl" => Some(78),
                    "glow" => Some(79),
                    "good" => Some(80),
                    "gray" => Some(81),
                    "grim" => Some(82),
                    "guru" => Some(83),
                    "gush" => Some(84),
                    "gyro" => Some(85),
                    "half" => Some(86),
                    "hang" => Some(87),
                    "hard" => Some(88),
                    "hawk" => Some(89),
                    "heat" => Some(90),
                    "help" => Some(91),
                    "high" => Some(92),
                    "hill" => Some(93),
                    "holy" => Some(94),
                    "hope" => Some(95),
                    "horn" => Some(96),
                    "huts" => Some(97),
                    "iced" => Some(98),
                    "idea" => Some(99),
                    "idle" => Some(100),
                    "inch" => Some(101),
                    "inky" => Some(102),
                    "into" => Some(103),
                    "iris" => Some(104),
                    "iron" => Some(105),
                    "item" => Some(106),
                    "jade" => Some(107),
                    "jazz" => Some(108),
                    "join" => Some(109),
                    "jolt" => Some(110),
                    "jowl" => Some(111),
                    "judo" => Some(112),
                    "jugs" => Some(113),
                    "jump" => Some(114),
                    "junk" => Some(115),
                    "jury" => Some(116),
                    "keep" => Some(117),
                    "keno" => Some(118),
                    "kept" => Some(119),
                    "keys" => Some(120),
                    "kick" => Some(121),
                    "kiln" => Some(122),
                    "king" => Some(123),
                    "kite" => Some(124),
                    "kiwi" => Some(125),
                    "knob" => Some(126),
                    "lamb" => Some(127),
                    "lava" => Some(128),
                    "lazy" => Some(129),
                    "leaf" => Some(130),
                    "legs" => Some(131),
                    "liar" => Some(132),
                    "limp" => Some(133),
                    "lion" => Some(134),
                    "list" => Some(135),
                    "logo" => Some(136),
                    "loud" => Some(137),
                    "love" => Some(138),
                    "luau" => Some(139),
                    "luck" => Some(140),
                    "lung" => Some(141),
                    "main" => Some(142),
                    "many" => Some(143),
                    "math" => Some(144),
                    "maze" => Some(145),
                    "memo" => Some(146),
                    "menu" => Some(147),
                    "meow" => Some(148),
                    "mild" => Some(149),
                    "mint" => Some(150),
                    "miss" => Some(151),
                    "monk" => Some(152),
                    "nail" => Some(153),
                    "navy" => Some(154),
                    "need" => Some(155),
                    "news" => Some(156),
                    "next" => Some(157),
                    "noon" => Some(158),
                    "note" => Some(159),
                    "numb" => Some(160),
                    "obey" => Some(161),
                    "oboe" => Some(162),
                    "omit" => Some(163),
                    "onyx" => Some(164),
                    "open" => Some(165),
                    "oval" => Some(166),
                    "owls" => Some(167),
                    "paid" => Some(168),
                    "part" => Some(169),
                    "peck" => Some(170),
                    "play" => Some(171),
                    "plus" => Some(172),
                    "poem" => Some(173),
                    "pool" => Some(174),
                    "pose" => Some(175),
                    "puff" => Some(176),
                    "puma" => Some(177),
                    "purr" => Some(178),
                    "quad" => Some(179),
                    "quiz" => Some(180),
                    "race" => Some(181),
                    "ramp" => Some(182),
                    "real" => Some(183),
                    "redo" => Some(184),
                    "rich" => Some(185),
                    "road" => Some(186),
                    "rock" => Some(187),
                    "roof" => Some(188),
                    "ruby" => Some(189),
                    "ruin" => Some(190),
                    "runs" => Some(191),
                    "rust" => Some(192),
                    "safe" => Some(193),
                    "saga" => Some(194),
                    "scar" => Some(195),
                    "sets" => Some(196),
                    "silk" => Some(197),
                    "skew" => Some(198),
                    "slot" => Some(199),
                    "soap" => Some(200),
                    "solo" => Some(201),
                    "song" => Some(202),
                    "stub" => Some(203),
                    "surf" => Some(204),
                    "swan" => Some(205),
                    "taco" => Some(206),
                    "task" => Some(207),
                    "taxi" => Some(208),
                    "tent" => Some(209),
                    "tied" => Some(210),
                    "time" => Some(211),
                    "tiny" => Some(212),
                    "toil" => Some(213),
                    "tomb" => Some(214),
                    "toys" => Some(215),
                    "trip" => Some(216),
                    "tuna" => Some(217),
                    "twin" => Some(218),
                    "ugly" => Some(219),
                    "undo" => Some(220),
                    "unit" => Some(221),
                    "urge" => Some(222),
                    "user" => Some(223),
                    "vast" => Some(224),
                    "very" => Some(225),
                    "veto" => Some(226),
                    "vial" => Some(227),
                    "vibe" => Some(228),
                    "view" => Some(229),
                    "visa" => Some(230),
                    "void" => Some(231),
                    "vows" => Some(232),
                    "wall" => Some(233),
                    "wand" => Some(234),
                    "warm" => Some(235),
                    "wasp" => Some(236),
                    "wave" => Some(237),
                    "waxy" => Some(238),
                    "webs" => Some(239),
                    "what" => Some(240),
                    "when" => Some(241),
                    "whiz" => Some(242),
                    "wolf" => Some(243),
                    "work" => Some(244),
                    "yank" => Some(245),
                    "yawn" => Some(246),
                    "yell" => Some(247),
                    "yoga" => Some(248),
                    "yurt" => Some(249),
                    "zaps" => Some(250),
                    "zero" => Some(251),
                    "zest" => Some(252),
                    "zinc" => Some(253),
                    "zone" => Some(254),
                    "zoom" => Some(255),
                    _ => None,
                }
            }
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(Error::InvalidWord)?,
    )
}

fn strip_checksum(mut data: Vec<u8>) -> Result<Vec<u8>, Error> {
    if data.len() < 4 {
        return Err(Error::InvalidChecksum);
    }
    let (payload, checksum) = data.split_at(data.len() - 4);
    if crate::crc32().checksum(payload).to_be_bytes() == checksum {
        data.truncate(data.len() - 4);
        Ok(data)
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
pub fn encode(data: &[u8], style: Style) -> alloc::string::String {
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

        // invalid length
        assert_eq!(
            decode("aea", Style::Minimal).unwrap_err(),
            Error::InvalidLength
        );

        // non ASCII
        assert_eq!(decode("₿", Style::Standard).unwrap_err(), Error::NonAscii);
        assert_eq!(decode("₿", Style::Uri).unwrap_err(), Error::NonAscii);
        assert_eq!(decode("₿", Style::Minimal).unwrap_err(), Error::NonAscii);
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
