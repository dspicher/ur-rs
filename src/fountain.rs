//! Split up big payloads into constantly sized chunks which can be recombined by a decoder.
//!
//! The `fountain` module provides an implementation of a fountain encoder, which splits
//! up a byte payload into multiple segments and emits an unbounded stream of parts which
//! can be recombined at the receiving decoder site. The emitted parts are either original
//! payload segments, or constructed by xor-ing a certain set of payload segments.
//!
//! A seeded `Xoshiro` RNG ensures that the receiver can reconstruct which segments
//! were combined into the part.
//! ```
//! let xor =
//!     |a: &[u8], b: &[u8]| -> Vec<_> { a.iter().zip(b.iter()).map(|(x1, x2)| x1 ^ x2).collect() };
//!
//! let data = String::from("Ten chars!");
//! let max_length = 4;
//! // note the padding
//! let (p1, p2, p3) = (b"Ten ", b"char", "s!\u{0}\u{0}".as_bytes());
//!
//! let mut encoder = ur::fountain::Encoder::new(data.as_bytes(), max_length).unwrap();
//! let mut decoder = ur::fountain::Decoder::default();
//!
//! // the fountain encoder first emits all original segments in order
//! let part1 = encoder.next_part();
//! assert_eq!(part1.data(), p1);
//! // receive the first part into the decoder
//! decoder.receive(part1).unwrap();
//!
//! let part2 = encoder.next_part();
//! assert_eq!(part2.data(), p2);
//! // receive the second part into the decoder
//! decoder.receive(part2).unwrap();
//!
//! // miss the third part
//! assert_eq!(encoder.next_part().data(), p3);
//!
//! // the RNG then first selects the original third segment
//! assert_eq!(encoder.next_part().data(), p3);
//!
//! // the RNG then selects all three segments to be xored
//! let xored = encoder.next_part();
//! assert_eq!(xored.data(), xor(&xor(p1, p2), p3));
//! // receive the xored part into the decoder
//! // since it already has p1 and p2, p3 can be computed
//! // from p1 xor p2 xor p3
//! decoder.receive(xored).unwrap();
//! assert!(decoder.complete());
//! assert_eq!(decoder.message().unwrap().as_deref(), Some(data.as_bytes()));
//! ```
//!
//! The index selection is biased towards combining fewer segments.
//!
//! ```
//! let data = String::from("Fifty chars").repeat(5);
//! let max_length = 5;
//! let mut encoder = ur::fountain::Encoder::new(data.as_bytes(), max_length).unwrap();
//! // 40% of the emitted parts represent original message segments
//! assert_eq!(
//!     (0..100)
//!         .map(|_i| if encoder.next_part().is_simple() {
//!             1
//!         } else {
//!             0
//!         })
//!         .sum::<usize>(),
//!     39
//! );
//! let mut encoder = ur::fountain::Encoder::new(data.as_bytes(), max_length).unwrap();
//! // On average, 3.33 segments (out of ten total) are combined into a part
//! assert_eq!(
//!     (0..100)
//!         .map(|_i| encoder.next_part().indexes().len())
//!         .sum::<usize>(),
//!     333
//! );
//! ```

extern crate alloc;
use alloc::vec::Vec;
use core::convert::Infallible;

/// Errors that can happen during fountain encoding and decoding.
#[derive(Debug)]
pub enum Error {
    /// CBOR decoding  error.
    CborDecode(minicbor::decode::Error),
    /// CBOR encoding error.
    CborEncode(minicbor::encode::Error<Infallible>),
    /// Expected non-empty message.
    EmptyMessage,
    /// Expected non-empty part.
    EmptyPart,
    /// Fragment length should be a positive integer greater than 0.
    InvalidFragmentLen,
    /// Received part is inconsistent with previous ones.
    InconsistentPart,
    /// An item was expected.
    ExpectedItem,
    /// Invalid padding detected.
    InvalidPadding,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CborDecode(e) => write!(f, "minicbor decoding error: {e}"),
            Self::CborEncode(e) => write!(f, "minicbor encoding error: {e}"),
            Self::EmptyMessage => write!(f, "expected non-empty message"),
            Self::EmptyPart => write!(f, "expected non-empty part"),
            Self::InvalidFragmentLen => write!(f, "expected positive maximum fragment length"),
            Self::InconsistentPart => write!(f, "part is inconsistent with previous ones"),
            Self::ExpectedItem => write!(f, "expected item"),
            Self::InvalidPadding => write!(f, "invalid padding"),
        }
    }
}

impl From<minicbor::decode::Error> for Error {
    fn from(e: minicbor::decode::Error) -> Self {
        Self::CborDecode(e)
    }
}

impl From<minicbor::encode::Error<Infallible>> for Error {
    fn from(e: minicbor::encode::Error<Infallible>) -> Self {
        Self::CborEncode(e)
    }
}

/// An encoder capable of emitting fountain-encoded transmissions.
///
/// # Examples
///
/// See the [`crate::fountain`] module documentation for an example.
#[derive(Debug)]
pub struct Encoder {
    parts: Vec<Vec<u8>>,
    message_length: usize,
    checksum: u32,
    current_sequence: usize,
}

impl Encoder {
    /// Constructs a new [`Encoder`], given a message and a maximum fragment length.
    ///
    /// # Examples
    ///
    /// ```
    /// use ur::fountain::Encoder;
    /// let encoder = Encoder::new(b"binary data", 4).unwrap();
    /// ```
    ///
    /// Note that the effective fragment size will not always equal the maximum
    /// fragment size:
    ///
    /// ```
    /// use ur::fountain::Encoder;
    /// let mut encoder = Encoder::new(b"data", 3).unwrap();
    /// assert_eq!(encoder.next_part().data().len(), 2);
    /// ```
    ///
    /// # Errors
    ///
    /// If an empty message or a zero maximum fragment length is passed, an error
    /// will be returned.
    pub fn new(message: &[u8], max_fragment_length: usize) -> Result<Self, Error> {
        if message.is_empty() {
            return Err(Error::EmptyMessage);
        }
        if max_fragment_length == 0 {
            return Err(Error::InvalidFragmentLen);
        }
        let fragment_length = fragment_length(message.len(), max_fragment_length);
        let fragments = partition(message.to_vec(), fragment_length);
        Ok(Self {
            parts: fragments,
            message_length: message.len(),
            checksum: crate::crc32().checksum(message),
            current_sequence: 0,
        })
    }

    /// Returns the current count of how many parts have been emitted.
    ///
    /// # Examples
    ///
    /// ```
    /// use ur::fountain::Encoder;
    /// let mut encoder = Encoder::new(b"data", 3).unwrap();
    /// assert_eq!(encoder.current_sequence(), 0);
    /// encoder.next_part();
    /// assert_eq!(encoder.current_sequence(), 1);
    /// ```
    #[must_use]
    pub const fn current_sequence(&self) -> usize {
        self.current_sequence
    }

    /// Returns the next part to be emitted by the fountain encoder.
    /// After all parts of the original message have been emitted once,
    /// the fountain encoder will emit the result of xoring together the parts
    /// selected by the Xoshiro RNG (which could be a single part).
    ///
    /// # Examples
    ///
    /// See the [`crate::fountain`] module documentation for an example.
    pub fn next_part(&mut self) -> Part {
        self.current_sequence += 1;
        let indexes = choose_fragments(self.current_sequence, self.parts.len(), self.checksum);

        let mut mixed = alloc::vec![0; self.parts[0].len()];
        for item in indexes {
            xor(&mut mixed, &self.parts[item]);
        }

        Part {
            sequence: self.current_sequence,
            sequence_count: self.parts.len(),
            message_length: self.message_length,
            checksum: self.checksum,
            data: mixed,
        }
    }

    /// Returns the number of segments the original message has been split up into.
    ///
    /// # Examples
    ///
    /// ```
    /// use ur::fountain::Encoder;
    /// let mut encoder = Encoder::new(b"data", 3).unwrap();
    /// assert_eq!(encoder.fragment_count(), 2);
    /// ```
    #[must_use]
    pub fn fragment_count(&self) -> usize {
        self.parts.len()
    }

    /// Returns whether all original segments have been emitted at least once.
    /// The fountain encoding is defined as doing this before combining segments
    /// with each other. Thus, this is equivalent to checking whether
    /// [`current_sequence`] >= [`fragment_count`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ur::fountain::Encoder;
    /// let mut encoder = Encoder::new(&b"data".repeat(10), 3).unwrap();
    /// while !encoder.complete() {
    ///     assert!(encoder.current_sequence() < encoder.fragment_count());
    ///     encoder.next_part();
    /// }
    /// assert_eq!(encoder.current_sequence(), encoder.fragment_count());
    /// ```
    ///
    /// [`fragment_count`]: Encoder::fragment_count
    /// [`current_sequence`]: Encoder::current_sequence
    #[must_use]
    pub fn complete(&self) -> bool {
        self.current_sequence >= self.parts.len()
    }
}

/// A decoder capable of receiving and recombining fountain-encoded transmissions.
///
/// # Examples
///
/// See the [`crate::fountain`] module documentation for an example.
#[derive(Default)]
pub struct Decoder {
    decoded: alloc::collections::btree_map::BTreeMap<usize, Part>,
    received: alloc::collections::btree_set::BTreeSet<Vec<usize>>,
    buffer: alloc::collections::btree_map::BTreeMap<Vec<usize>, Part>,
    queue: Vec<(usize, Part)>,
    sequence_count: usize,
    message_length: usize,
    checksum: u32,
    fragment_length: usize,
}

impl Decoder {
    /// Receives a fountain-encoded part into the decoder.
    ///
    /// # Examples
    ///
    /// See the [`crate::fountain`] module documentation for an example.
    ///
    /// # Errors
    ///
    /// If the part would fail [`validate`] because it is inconsistent
    /// with previously received parts, an error will be returned.
    ///
    /// [`validate`]: Decoder::validate
    pub fn receive(&mut self, part: Part) -> Result<bool, Error> {
        if self.complete() {
            return Ok(false);
        }

        // Only receive parts that will yield data.
        if part.sequence_count == 0 || part.data.is_empty() || part.message_length == 0 {
            return Err(Error::EmptyPart);
        }

        if self.received.is_empty() {
            self.sequence_count = part.sequence_count;
            self.message_length = part.message_length;
            self.checksum = part.checksum;
            self.fragment_length = part.data.len();
        } else if !self.validate(&part) {
            return Err(Error::InconsistentPart);
        }
        let indexes = part.indexes();
        if self.received.contains(&indexes) {
            return Ok(false);
        }
        self.received.insert(indexes);
        if part.is_simple() {
            self.process_simple(part)?;
        } else {
            self.process_complex(part)?;
        }
        Ok(true)
    }

    fn process_simple(&mut self, part: Part) -> Result<(), Error> {
        let index = *part.indexes().first().ok_or(Error::ExpectedItem)?;
        self.decoded.insert(index, part.clone());
        self.queue.push((index, part));
        self.process_queue()?;
        Ok(())
    }

    fn process_queue(&mut self) -> Result<(), Error> {
        while !self.queue.is_empty() {
            let (index, simple) = self.queue.pop().ok_or(Error::ExpectedItem)?;
            let to_process: Vec<Vec<usize>> = self
                .buffer
                .keys()
                .filter(|&idxs| idxs.contains(&index))
                .cloned()
                .collect();
            for indexes in to_process {
                let mut part = self.buffer.remove(&indexes).ok_or(Error::ExpectedItem)?;
                let mut new_indexes = indexes.clone();
                let to_remove = indexes
                    .iter()
                    .position(|&x| x == index)
                    .ok_or(Error::ExpectedItem)?;
                new_indexes.remove(to_remove);
                xor(&mut part.data, &simple.data);
                if new_indexes.len() == 1 {
                    self.decoded
                        .insert(*new_indexes.first().unwrap(), part.clone());
                    self.queue.push((*new_indexes.first().unwrap(), part));
                } else {
                    self.buffer.insert(new_indexes, part);
                }
            }
        }
        Ok(())
    }

    fn process_complex(&mut self, mut part: Part) -> Result<(), Error> {
        let mut indexes = part.indexes();
        let to_remove: Vec<usize> = indexes
            .clone()
            .into_iter()
            .filter(|idx| self.decoded.keys().any(|k| k == idx))
            .collect();
        if indexes.len() == to_remove.len() {
            return Ok(());
        }
        for remove in to_remove {
            let idx_to_remove = indexes
                .iter()
                .position(|&x| x == remove)
                .ok_or(Error::ExpectedItem)?;
            indexes.remove(idx_to_remove);
            xor(
                &mut part.data,
                &self.decoded.get(&remove).ok_or(Error::ExpectedItem)?.data,
            );
        }
        if indexes.len() == 1 {
            self.decoded.insert(*indexes.first().unwrap(), part.clone());
            self.queue.push((*indexes.first().unwrap(), part));
        } else {
            self.buffer.insert(indexes, part);
        }
        Ok(())
    }

    /// Returns whether the decoder is complete and hence the message available.
    ///
    /// # Examples
    ///
    /// See the [`crate::fountain`] module documentation for an example.
    #[must_use]
    pub fn complete(&self) -> bool {
        self.message_length != 0 && self.decoded.len() == self.sequence_count
    }

    /// Checks whether a [`Part`] is receivable by the decoder.
    /// This can fail if other parts were previously received whose
    /// metadata (such as number of segments) is inconsistent with the
    /// present [`Part`]. Note that a fresh decoder will always return
    /// false here.
    ///
    /// # Examples
    ///
    /// ```
    /// use ur::fountain::{Decoder, Encoder};
    /// let mut decoder = Decoder::default();
    /// let mut encoder = Encoder::new(b"data", 3).unwrap();
    /// let part = encoder.next_part();
    ///
    /// // a fresh decoder always returns false
    /// assert!(!decoder.validate(&part));
    ///
    /// // parts with the same metadata validate successfully
    /// decoder.receive(part).unwrap();
    /// let part = encoder.next_part();
    /// assert!(decoder.validate(&part));
    ///
    /// // parts with the different metadata don't validate
    /// let mut encoder = Encoder::new(b"more data", 3).unwrap();
    /// let part = encoder.next_part();
    /// assert!(!decoder.validate(&part));
    /// ```
    #[must_use]
    pub fn validate(&self, part: &Part) -> bool {
        if self.received.is_empty() {
            return false;
        }

        if part.sequence_count != self.sequence_count
            || part.message_length != self.message_length
            || part.checksum != self.checksum
            || part.data.len() != self.fragment_length
        {
            return false;
        }

        true
    }

    /// If [`complete`], returns the decoded message, `None` otherwise.
    ///
    /// # Errors
    ///
    /// If an inconsistent internal state is detected, an error will be returned.
    ///
    /// # Examples
    ///
    /// See the [`crate::fountain`] module documentation for an example.
    ///
    /// [`complete`]: Decoder::complete
    pub fn message(&self) -> Result<Option<Vec<u8>>, Error> {
        if !self.complete() {
            return Ok(None);
        }
        let combined = (0..self.sequence_count)
            .map(|idx| self.decoded.get(&idx).ok_or(Error::ExpectedItem))
            .collect::<Result<Vec<&Part>, Error>>()?
            .iter()
            .fold(alloc::vec![], |a, b| [a, b.data.clone()].concat());
        if !combined
            .get(self.message_length..)
            .ok_or(Error::ExpectedItem)?
            .iter()
            .all(|&x| x == 0)
        {
            return Err(Error::InvalidPadding);
        }
        Ok(Some(
            combined
                .get(..self.message_length)
                .ok_or(Error::ExpectedItem)?
                .to_vec(),
        ))
    }
}

/// A part emitted by a fountain [`Encoder`].
///
/// Most commonly, this is obtained by calling [`next_part`] on the encoder.
///
/// [`next_part`]: Encoder::next_part
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    sequence: usize,
    sequence_count: usize,
    message_length: usize,
    checksum: u32,
    data: Vec<u8>,
}

impl<C> minicbor::Encode<C> for Part {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        #[allow(clippy::cast_possible_truncation)]
        e.array(5)?
            .u32(self.sequence as u32)?
            .u32(self.sequence_count as u32)?
            .u32(self.message_length as u32)?
            .u32(self.checksum)?
            .bytes(&self.data)?;

        Ok(())
    }
}

impl<'b, C> minicbor::Decode<'b, C> for Part {
    fn decode(
        d: &mut minicbor::Decoder<'b>,
        _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        if !matches!(d.array()?, Some(5)) {
            return Err(minicbor::decode::Error::message(
                "invalid CBOR array length",
            ));
        }

        Ok(Self {
            sequence: d.u32()? as usize,
            sequence_count: d.u32()? as usize,
            message_length: d.u32()? as usize,
            checksum: d.u32()?,
            data: d.bytes()?.to_vec(),
        })
    }
}

impl Part {
    pub(crate) fn from_cbor(cbor: &[u8]) -> Result<Self, Error> {
        minicbor::decode(cbor).map_err(Error::from)
    }

    /// Returns the indexes of the message segments that were combined into this part.
    ///
    /// # Examples
    ///
    /// ```
    /// use ur::fountain::Encoder;
    /// let mut encoder = Encoder::new(b"data", 3).unwrap();
    /// assert_eq!(encoder.next_part().indexes(), vec![0]);
    /// assert_eq!(encoder.next_part().indexes(), vec![1]);
    /// ```
    #[must_use]
    pub fn indexes(&self) -> Vec<usize> {
        choose_fragments(self.sequence, self.sequence_count, self.checksum)
    }

    /// Indicates whether this part is an original segment of the message, or was obtained by
    /// combining multiple segments via xor.
    ///
    /// # Examples
    ///
    /// The encoding setup follows the `fountain` module example.
    ///
    /// ```
    /// use ur::fountain::Encoder;
    /// let mut encoder = Encoder::new(b"Ten chars!", 4).unwrap();
    /// // The encoder always emits the simple parts covering the message first
    /// assert!(encoder.next_part().is_simple());
    /// assert!(encoder.next_part().is_simple());
    /// assert!(encoder.next_part().is_simple());
    /// // The encoder then emits segment 3 again
    /// assert!(encoder.next_part().is_simple());
    /// // The encoder then emits all 3 segments combined
    /// assert!(!encoder.next_part().is_simple());
    /// ```
    #[must_use]
    pub fn is_simple(&self) -> bool {
        self.indexes().len() == 1
    }

    pub(crate) fn cbor(&self) -> Result<Vec<u8>, Error> {
        minicbor::to_vec(self).map_err(Error::from)
    }

    #[must_use]
    pub(crate) fn sequence_id(&self) -> alloc::string::String {
        alloc::format!("{}-{}", self.sequence, self.sequence_count)
    }

    /// Returns a slice view onto the underlying data.
    ///
    /// Note that for non-simple parts this will be the result of
    /// xoring multiple message segments together.
    ///
    /// # Examples
    ///
    /// ```
    /// let data = String::from("Ten chars!");
    /// let mut encoder = ur::fountain::Encoder::new(data.as_bytes(), 4).unwrap();
    /// assert_eq!(encoder.next_part().data(), "Ten ".as_bytes());
    /// ```
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Calculates the quotient of `a` and `b`, rounding the results towards
/// positive infinity.
///
/// Note: there's an implementation on the `usize` type of this function,
/// but it's not stable yet.
#[must_use]
const fn div_ceil(a: usize, b: usize) -> usize {
    let d = a / b;
    let r = a % b;
    if r > 0 { d + 1 } else { d }
}

#[must_use]
pub(crate) const fn fragment_length(data_length: usize, max_fragment_length: usize) -> usize {
    let fragment_count = div_ceil(data_length, max_fragment_length);
    div_ceil(data_length, fragment_count)
}

#[must_use]
pub(crate) fn partition(mut data: Vec<u8>, fragment_length: usize) -> Vec<Vec<u8>> {
    let mut padding =
        alloc::vec![0; (fragment_length - (data.len() % fragment_length)) % fragment_length];
    data.append(&mut padding);
    data.chunks(fragment_length).map(<[u8]>::to_vec).collect()
}

#[must_use]
fn choose_fragments(sequence: usize, fragment_count: usize, checksum: u32) -> Vec<usize> {
    if sequence <= fragment_count {
        return alloc::vec![sequence - 1];
    }

    #[allow(clippy::cast_possible_truncation)]
    let sequence = sequence as u32;

    let mut seed = [0u8; 8];
    seed[0..4].copy_from_slice(&sequence.to_be_bytes());
    seed[4..8].copy_from_slice(&checksum.to_be_bytes());

    let mut xoshiro = crate::xoshiro::Xoshiro256::from(seed.as_slice());
    let degree = xoshiro.choose_degree(fragment_count);
    let indexes = (0..fragment_count).collect();
    let mut shuffled = xoshiro.shuffled(indexes);
    shuffled.truncate(degree as usize);
    shuffled
}

fn xor(v1: &mut [u8], v2: &[u8]) {
    debug_assert_eq!(v1.len(), v2.len());

    for (x1, &x2) in v1.iter_mut().zip(v2.iter()) {
        *x1 ^= x2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_length() {
        assert_eq!(fragment_length(12345, 1955), 1764);
        assert_eq!(fragment_length(12345, 30000), 12345);

        assert_eq!(fragment_length(10, 4), 4);
        assert_eq!(fragment_length(10, 5), 5);
        assert_eq!(fragment_length(10, 6), 5);
        assert_eq!(fragment_length(10, 10), 10);
    }

    #[test]
    fn test_partition_and_join() {
        let join = |data: Vec<Vec<u8>>, message_length: usize| {
            let mut flattened: Vec<u8> = data.into_iter().flatten().collect();
            flattened.truncate(message_length);
            flattened
        };

        let message = crate::xoshiro::test_utils::make_message("Wolf", 1024);
        let fragment_length = fragment_length(message.len(), 100);
        let fragments = partition(message.clone(), fragment_length);
        let expected_fragments = vec![
            "916ec65cf77cadf55cd7f9cda1a1030026ddd42e905b77adc36e4f2d3ccba44f7f04f2de44f42d84c374a0e149136f25b01852545961d55f7f7a8cde6d0e2ec43f3b2dcb644a2209e8c9e34af5c4747984a5e873c9cf5f965e25ee29039f",
            "df8ca74f1c769fc07eb7ebaec46e0695aea6cbd60b3ec4bbff1b9ffe8a9e7240129377b9d3711ed38d412fbb4442256f1e6f595e0fc57fed451fb0a0101fb76b1fb1e1b88cfdfdaa946294a47de8fff173f021c0e6f65b05c0a494e50791",
            "270a0050a73ae69b6725505a2ec8a5791457c9876dd34aadd192a53aa0dc66b556c0c215c7ceb8248b717c22951e65305b56a3706e3e86eb01c803bbf915d80edcd64d4d41977fa6f78dc07eecd072aae5bc8a852397e06034dba6a0b570",
            "797c3a89b16673c94838d884923b8186ee2db5c98407cab15e13678d072b43e406ad49477c2e45e85e52ca82a94f6df7bbbe7afbed3a3a830029f29090f25217e48d1f42993a640a67916aa7480177354cc7440215ae41e4d02eae9a1912",
            "33a6d4922a792c1b7244aa879fefdb4628dc8b0923568869a983b8c661ffab9b2ed2c149e38d41fba090b94155adbed32f8b18142ff0d7de4eeef2b04adf26f2456b46775c6c20b37602df7da179e2332feba8329bbb8d727a138b4ba7a5",
            "03215eda2ef1e953d89383a382c11d3f2cad37a4ee59a91236a3e56dcf89f6ac81dd4159989c317bd649d9cbc617f73fe10033bd288c60977481a09b343d3f676070e67da757b86de27bfca74392bac2996f7822a7d8f71a489ec6180390",
            "089ea80a8fcd6526413ec6c9a339115f111d78ef21d456660aa85f790910ffa2dc58d6a5b93705caef1091474938bd312427021ad1eeafbd19e0d916ddb111fabd8dcab5ad6a6ec3a9c6973809580cb2c164e26686b5b98cfb017a337968",
            "c7daaa14ae5152a067277b1b3902677d979f8e39cc2aafb3bc06fcf69160a853e6869dcc09a11b5009f91e6b89e5b927ab1527a735660faa6012b420dd926d940d742be6a64fb01cdc0cff9faa323f02ba41436871a0eab851e7f5782d10",
            "fbefde2a7e9ae9dc1e5c2c48f74f6c824ce9ef3c89f68800d44587bedc4ab417cfb3e7447d90e1e417e6e05d30e87239d3a5d1d45993d4461e60a0192831640aa32dedde185a371ded2ae15f8a93dba8809482ce49225daadfbb0fec629e",
            "23880789bdf9ed73be57fa84d555134630e8d0f7df48349f29869a477c13ccca9cd555ac42ad7f568416c3d61959d0ed568b2b81c7771e9088ad7fd55fd4386bafbf5a528c30f107139249357368ffa980de2c76ddd9ce4191376be0e6b5",
            "170010067e2e75ebe2d2904aeb1f89d5dc98cd4a6f2faaa8be6d03354c990fd895a97feb54668473e9d942bb99e196d897e8f1b01625cf48a7b78d249bb4985c065aa8cd1402ed2ba1b6f908f63dcd84b66425df00000000000000000000",
        ];
        assert_eq!(fragments.len(), expected_fragments.len());
        for (fragment, expected_fragment) in fragments.iter().zip(expected_fragments) {
            assert_eq!(hex::encode(fragment), expected_fragment);
        }
        let rejoined = join(fragments, message.len());
        assert_eq!(rejoined, message);
    }

    #[test]
    fn test_choose_fragments() {
        let message = crate::xoshiro::test_utils::make_message("Wolf", 1024);
        let checksum = crate::crc32().checksum(&message);
        let fragment_length = crate::fountain::fragment_length(message.len(), 100);
        let fragments = crate::fountain::partition(message, fragment_length);
        let expected_fragment_indexes = vec![
            vec![0],
            vec![1],
            vec![2],
            vec![3],
            vec![4],
            vec![5],
            vec![6],
            vec![7],
            vec![8],
            vec![9],
            vec![10],
            vec![9],
            vec![2, 5, 6, 8, 9, 10],
            vec![8],
            vec![1, 5],
            vec![1],
            vec![0, 2, 4, 5, 8, 10],
            vec![5],
            vec![2],
            vec![2],
            vec![0, 1, 3, 4, 5, 7, 9, 10],
            vec![0, 1, 2, 3, 5, 6, 8, 9, 10],
            vec![0, 2, 4, 5, 7, 8, 9, 10],
            vec![3, 5],
            vec![4],
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            vec![0, 1, 3, 4, 5, 6, 7, 9, 10],
            vec![6],
            vec![5, 6],
            vec![7],
        ];
        for seq_num in 1..=30 {
            let mut indexes = crate::fountain::choose_fragments(seq_num, fragments.len(), checksum);
            indexes.sort_unstable();
            assert_eq!(indexes, expected_fragment_indexes[seq_num - 1]);
        }
    }

    #[test]
    fn test_xor() {
        let mut rng = crate::xoshiro::Xoshiro256::from("Wolf");

        let data1 = rng.next_bytes(10);
        assert_eq!(hex::encode(&data1), "916ec65cf77cadf55cd7");

        let data2 = rng.next_bytes(10);
        assert_eq!(hex::encode(&data2), "f9cda1a1030026ddd42e");

        let mut data3 = data1.clone();
        xor(&mut data3, &data2);
        assert_eq!(hex::encode(&data3), "68a367fdf47c8b2888f9");

        xor(&mut data3, &data1);
        assert_eq!(hex::encode(data3), hex::encode(data2));
    }

    #[test]
    fn test_fountain_encoder() {
        let message = crate::xoshiro::test_utils::make_message("Wolf", 256);
        let mut encoder = Encoder::new(&message, 30).unwrap();
        let expected_parts = [
            "916ec65cf77cadf55cd7f9cda1a1030026ddd42e905b77adc36e4f2d3c",
            "cba44f7f04f2de44f42d84c374a0e149136f25b01852545961d55f7f7a",
            "8cde6d0e2ec43f3b2dcb644a2209e8c9e34af5c4747984a5e873c9cf5f",
            "965e25ee29039fdf8ca74f1c769fc07eb7ebaec46e0695aea6cbd60b3e",
            "c4bbff1b9ffe8a9e7240129377b9d3711ed38d412fbb4442256f1e6f59",
            "5e0fc57fed451fb0a0101fb76b1fb1e1b88cfdfdaa946294a47de8fff1",
            "73f021c0e6f65b05c0a494e50791270a0050a73ae69b6725505a2ec8a5",
            "791457c9876dd34aadd192a53aa0dc66b556c0c215c7ceb8248b717c22",
            "951e65305b56a3706e3e86eb01c803bbf915d80edcd64d4d0000000000",
            "330f0f33a05eead4f331df229871bee733b50de71afd2e5a79f196de09",
            "3b205ce5e52d8c24a52cffa34c564fa1af3fdffcd349dc4258ee4ee828",
            "dd7bf725ea6c16d531b5f03254783803048ca08b87148daacd1cd7a006",
            "760be7ad1c6187902bbc04f539b9ee5eb8ea6833222edea36031306c01",
            "5bf4031217d2c3254b088fa7553778b5003632f46e21db129416f65b55",
            "73f021c0e6f65b05c0a494e50791270a0050a73ae69b6725505a2ec8a5",
            "b8546ebfe2048541348910267331c643133f828afec9337c318f71b7df",
            "23dedeea74e3a0fb052befabefa13e2f80e4315c9dceed4c8630612e64",
            "d01a8daee769ce34b6b35d3ca0005302724abddae405bdb419c0a6b208",
            "3171c5dc365766eff25ae47c6f10e7de48cfb8474e050e5fe997a6dc24",
            "e055c2433562184fa71b4be94f262e200f01c6f74c284b0dc6fae6673f",
        ]
        .iter()
        .enumerate()
        .map(|(i, data)| super::Part {
            sequence: i + 1,
            sequence_count: 9,
            message_length: 256,
            checksum: 23_570_951,
            data: hex::decode(data).unwrap(),
        });
        for (sequence, e) in expected_parts.into_iter().enumerate() {
            assert_eq!(encoder.current_sequence(), sequence);
            assert_eq!(encoder.next_part(), e);
        }
    }

    #[test]
    fn test_fountain_encoder_cbor() {
        let max_fragment_length = 30;
        let size = 256;
        let message = crate::xoshiro::test_utils::make_message("Wolf", size);
        let mut encoder = Encoder::new(&message, max_fragment_length).unwrap();
        let expected_parts = vec![
            "8501091901001a0167aa07581d916ec65cf77cadf55cd7f9cda1a1030026ddd42e905b77adc36e4f2d3c",
            "8502091901001a0167aa07581dcba44f7f04f2de44f42d84c374a0e149136f25b01852545961d55f7f7a",
            "8503091901001a0167aa07581d8cde6d0e2ec43f3b2dcb644a2209e8c9e34af5c4747984a5e873c9cf5f",
            "8504091901001a0167aa07581d965e25ee29039fdf8ca74f1c769fc07eb7ebaec46e0695aea6cbd60b3e",
            "8505091901001a0167aa07581dc4bbff1b9ffe8a9e7240129377b9d3711ed38d412fbb4442256f1e6f59",
            "8506091901001a0167aa07581d5e0fc57fed451fb0a0101fb76b1fb1e1b88cfdfdaa946294a47de8fff1",
            "8507091901001a0167aa07581d73f021c0e6f65b05c0a494e50791270a0050a73ae69b6725505a2ec8a5",
            "8508091901001a0167aa07581d791457c9876dd34aadd192a53aa0dc66b556c0c215c7ceb8248b717c22",
            "8509091901001a0167aa07581d951e65305b56a3706e3e86eb01c803bbf915d80edcd64d4d0000000000",
            "850a091901001a0167aa07581d330f0f33a05eead4f331df229871bee733b50de71afd2e5a79f196de09",
            "850b091901001a0167aa07581d3b205ce5e52d8c24a52cffa34c564fa1af3fdffcd349dc4258ee4ee828",
            "850c091901001a0167aa07581ddd7bf725ea6c16d531b5f03254783803048ca08b87148daacd1cd7a006",
            "850d091901001a0167aa07581d760be7ad1c6187902bbc04f539b9ee5eb8ea6833222edea36031306c01",
            "850e091901001a0167aa07581d5bf4031217d2c3254b088fa7553778b5003632f46e21db129416f65b55",
            "850f091901001a0167aa07581d73f021c0e6f65b05c0a494e50791270a0050a73ae69b6725505a2ec8a5",
            "8510091901001a0167aa07581db8546ebfe2048541348910267331c643133f828afec9337c318f71b7df",
            "8511091901001a0167aa07581d23dedeea74e3a0fb052befabefa13e2f80e4315c9dceed4c8630612e64",
            "8512091901001a0167aa07581dd01a8daee769ce34b6b35d3ca0005302724abddae405bdb419c0a6b208",
            "8513091901001a0167aa07581d3171c5dc365766eff25ae47c6f10e7de48cfb8474e050e5fe997a6dc24",
            "8514091901001a0167aa07581de055c2433562184fa71b4be94f262e200f01c6f74c284b0dc6fae6673f",
        ];
        assert_eq!(encoder.fragment_count(), size / max_fragment_length + 1);
        for e in expected_parts {
            assert_eq!(hex::encode(encoder.next_part().cbor().unwrap()), e);
        }
    }

    #[test]
    fn test_fountain_encoder_zero_max_length() {
        assert!(matches!(
            Encoder::new(b"foo", 0),
            Err(Error::InvalidFragmentLen)
        ));
    }

    #[test]
    fn test_fountain_encoder_is_complete() {
        let message = crate::xoshiro::test_utils::make_message("Wolf", 256);
        let mut encoder = Encoder::new(&message, 30).unwrap();
        for _ in 0..encoder.parts.len() {
            encoder.next_part();
        }
        assert!(encoder.complete());
    }

    #[test]
    fn test_decoder() {
        let seed = "Wolf";
        let message_size = 32767;
        let max_fragment_length = 1000;

        let message = crate::xoshiro::test_utils::make_message(seed, message_size);
        let mut encoder = Encoder::new(&message, max_fragment_length).unwrap();
        let mut decoder = Decoder::default();
        while !decoder.complete() {
            assert_eq!(decoder.message().unwrap(), None);
            let part = encoder.next_part();
            let _next = decoder.receive(part);
        }
        assert_eq!(decoder.message().unwrap(), Some(message));
    }

    #[test]
    fn test_empty_encoder() {
        assert!(Encoder::new(&[], 1).is_err());
    }

    #[test]
    fn test_decoder_skip_some_simple_fragments() {
        let seed = "Wolf";
        let message_size = 32767;
        let max_fragment_length = 1000;

        let message = crate::xoshiro::test_utils::make_message(seed, message_size);
        let mut encoder = Encoder::new(&message, max_fragment_length).unwrap();
        let mut decoder = Decoder::default();
        let mut skip = false;
        while !decoder.complete() {
            let part = encoder.next_part();
            if !skip {
                let _next = decoder.receive(part);
            }
            skip = !skip;
        }
        assert_eq!(decoder.message().unwrap(), Some(message));
    }

    #[test]
    fn test_decoder_receive_return_value() {
        let seed = "Wolf";
        let message_size = 1000;
        let max_fragment_length = 10;

        let message = crate::xoshiro::test_utils::make_message(seed, message_size);
        let mut encoder = Encoder::new(&message, max_fragment_length).unwrap();
        let mut decoder = Decoder::default();
        let part = encoder.next_part();
        assert_eq!(
            part.data(),
            vec![0x91, 0x6e, 0xc6, 0x5c, 0xf7, 0x7c, 0xad, 0xf5, 0x5c, 0xd7]
        );
        assert!(decoder.receive(part.clone()).unwrap());
        // same indexes
        assert!(!decoder.receive(part).unwrap());
        // non-valid
        let mut part = encoder.next_part();
        part.checksum += 1;
        assert!(matches!(
            decoder.receive(part),
            Err(Error::InconsistentPart)
        ));
        // decoder complete
        while !decoder.complete() {
            let part = encoder.next_part();
            decoder.receive(part).unwrap();
        }
        let part = encoder.next_part();
        assert!(!decoder.receive(part).unwrap());
    }

    #[test]
    fn test_decoder_part_validation() {
        let mut encoder = Encoder::new(b"foo", 2).unwrap();
        let mut decoder = Decoder::default();
        let mut part = encoder.next_part();
        assert!(decoder.receive(part.clone()).unwrap());
        assert!(decoder.validate(&part));
        part.checksum += 1;
        assert!(!decoder.validate(&part));
        part.checksum -= 1;
        part.message_length += 1;
        assert!(!decoder.validate(&part));
        part.message_length -= 1;
        part.sequence_count += 1;
        assert!(!decoder.validate(&part));
        part.sequence_count -= 1;
        part.data.push(1);
        assert!(!decoder.validate(&part));
    }

    #[test]
    fn test_empty_decoder_empty_part() {
        let mut decoder = Decoder::default();
        let mut part = Part {
            sequence: 12,
            sequence_count: 8,
            message_length: 100,
            checksum: 0x1234_5678,
            data: vec![1, 5, 3, 3, 5],
        };

        // Check sequence_count.
        part.sequence_count = 0;
        assert!(matches!(
            decoder.receive(part.clone()),
            Err(Error::EmptyPart)
        ));
        part.sequence_count = 8;

        // Check message_length.
        part.message_length = 0;
        assert!(matches!(
            decoder.receive(part.clone()),
            Err(Error::EmptyPart)
        ));
        part.message_length = 100;

        // Check data.
        part.data = vec![];
        assert!(matches!(
            decoder.receive(part.clone()),
            Err(Error::EmptyPart)
        ));
        part.data = vec![1, 5, 3, 3, 5];

        // Should not validate as there aren't any previous parts received.
        assert!(!decoder.validate(&part));
    }

    #[test]
    fn test_fountain_cbor() {
        let part = Part {
            sequence: 12,
            sequence_count: 8,
            message_length: 100,
            checksum: 0x1234_5678,
            data: vec![1, 5, 3, 3, 5],
        };
        let cbor = part.cbor().unwrap();
        let part2 = Part::from_cbor(&cbor).unwrap();
        let cbor2 = part2.cbor().unwrap();
        assert_eq!(cbor, cbor2);
    }

    #[test]
    fn test_part_from_cbor_errors() {
        // 0x18 is the first byte value that doesn't directly encode a u8,
        // but implies a following value
        assert!(matches!(
            Part::from_cbor(&[0x18]),
            Err(Error::CborDecode(e)) if e.to_string() == "unexpected type u8 at position 0: expected array"
        ));
        // the top-level item must be an array
        assert!(
            matches!(Part::from_cbor(&[0x1]), Err(Error::CborDecode(e)) if e.to_string() == "unexpected type u8 at position 0: expected array")
        );
        // the array must be of length five
        assert!(matches!(
            Part::from_cbor(&[0x84, 0x1, 0x2, 0x3, 0x4]),
            Err(Error::CborDecode(e)) if e.to_string() == "decode error: invalid CBOR array length"
        ));
        assert!(matches!(
            Part::from_cbor(&[0x86, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6]),
            Err(Error::CborDecode(e)) if e.to_string() == "decode error: invalid CBOR array length"
        ));
        // items one through four must be an unsigned integer
        let mut cbor = [0x85, 0x1, 0x2, 0x3, 0x4, 0x41, 0x5];
        for idx in 1..=4 {
            Part::from_cbor(&cbor).unwrap();
            cbor[idx] = 0x41;
            assert!(matches!(
                Part::from_cbor(&cbor),
                Err(Error::CborDecode(e)) if e.to_string() == format!("unexpected type bytes at position {idx}: expected u32")
            ));
            cbor[idx] = u8::try_from(idx).unwrap();
        }
        // the fifth item must be byte string
        assert!(matches!(
            Part::from_cbor(&[0x85, 0x1, 0x2, 0x3, 0x4, 0x5]),
            Err(Error::CborDecode(e)) if e.to_string() == "unexpected type u8 at position 5: expected bytes (definite length)"
        ));
    }

    #[test]
    fn test_part_from_cbor_unsigned_types() {
        // u8
        Part::from_cbor(&[0x85, 0x1, 0x2, 0x3, 0x4, 0x41, 0x5]).unwrap();
        // u16
        Part::from_cbor(&[
            0x85, 0x19, 0x1, 0x2, 0x19, 0x3, 0x4, 0x19, 0x5, 0x6, 0x19, 0x7, 0x8, 0x41, 0x5,
        ])
        .unwrap();
        // u32
        Part::from_cbor(&[
            0x85, 0x1a, 0x1, 0x2, 0x3, 0x4, 0x1a, 0x5, 0x6, 0x7, 0x8, 0x1a, 0x9, 0x10, 0x11, 0x12,
            0x1a, 0x13, 0x14, 0x15, 0x16, 0x41, 0x5,
        ])
        .unwrap();
        // u64
        assert!(matches!(
            Part::from_cbor(&[
                0x85, 0x1b, 0x1, 0x2, 0x3, 0x4, 0xa, 0xb, 0xc, 0xd, 0x1a, 0x5, 0x6, 0x7, 0x8, 0x1a,
                0x9, 0x10, 0x11, 0x12, 0x1a, 0x13, 0x14, 0x15, 0x16, 0x41, 0x5,
            ]),
            Err(Error::CborDecode(e)) if e.to_string().contains("converting u64 to u32")
        ));
        assert!(matches!(
            Part::from_cbor(&[
                0x85, 0x1a, 0x1, 0x2, 0x3, 0x4, 0x1b, 0x5, 0x6, 0x7, 0x8, 0xa, 0xb, 0xc, 0xd, 0x1a,
                0x9, 0x10, 0x11, 0x12, 0x1a, 0x13, 0x14, 0x15, 0x16, 0x41, 0x5,
            ]),
            Err(Error::CborDecode(e)) if e.to_string().contains("converting u64 to u32")
        ));
        assert!(matches!(
            Part::from_cbor(&[
                0x85, 0x1a, 0x1, 0x2, 0x3, 0x4, 0x1a, 0x5, 0x6, 0x7, 0x8, 0x1b, 0x9, 0x10, 0x11,
                0x12, 0xa, 0xb, 0xc, 0xd, 0x1a, 0x13, 0x14, 0x15, 0x16, 0x41, 0x5,
            ]),
            Err(Error::CborDecode(e)) if e.to_string().contains("converting u64 to u32")
        ));
        assert!(matches!(
            Part::from_cbor(&[
                0x85, 0x1a, 0x1, 0x2, 0x3, 0x4, 0x1a, 0x5, 0x6, 0x7, 0x8, 0x1a, 0x9, 0x10, 0x11,
                0x12, 0x1b, 0x13, 0x14, 0x15, 0x16, 0xa, 0xb, 0xc, 0xd, 0x41, 0x5,
            ]),
            Err(Error::CborDecode(e)) if e.to_string().contains("converting u64 to u32")
        ));
    }

    #[test]
    fn test_error_formatting() {
        assert_eq!(
            super::Error::from(minicbor::decode::Error::end_of_input()).to_string(),
            "minicbor decoding error: end of input bytes"
        );
        assert_eq!(
            super::Error::from(minicbor::encode::Error::message("error")).to_string(),
            "minicbor encoding error: error"
        );
        assert_eq!(
            super::Error::EmptyMessage.to_string(),
            "expected non-empty message"
        );
        assert_eq!(
            super::Error::EmptyPart.to_string(),
            "expected non-empty part"
        );
        assert_eq!(
            super::Error::InvalidFragmentLen.to_string(),
            "expected positive maximum fragment length"
        );
        assert_eq!(
            super::Error::InconsistentPart.to_string(),
            "part is inconsistent with previous ones"
        );
        assert_eq!(super::Error::ExpectedItem.to_string(), "expected item");
        assert_eq!(super::Error::InvalidPadding.to_string(), "invalid padding");
    }

    #[test]
    fn test_invalid_padding() {
        let mut encoder = Encoder::new(b"Hello world", 20).unwrap();
        let mut part = encoder.next_part();
        part.message_length -= 1;
        let mut decoder = Decoder::default();
        decoder.receive(part).unwrap();
        assert_eq!(
            decoder.message().unwrap_err().to_string(),
            "invalid padding"
        );
    }
}
