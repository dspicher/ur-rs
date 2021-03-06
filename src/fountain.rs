pub struct Encoder {
    parts: Vec<Vec<u8>>,
    message_length: usize,
    checksum: u32,
    current_sequence: usize,
}

impl Encoder {
    #[must_use]
    pub fn new(message: &[u8], max_fragment_length: usize) -> Self {
        let fragment_length = fragment_length(message.len(), max_fragment_length);
        let fragments = partition(message.to_vec(), fragment_length);
        Self {
            parts: fragments,
            message_length: message.len(),
            checksum: crate::crc32().checksum(&message),
            current_sequence: 0,
        }
    }

    pub fn next_part(&mut self) -> anyhow::Result<Part> {
        self.current_sequence += 1;
        let indexes = choose_fragments(self.current_sequence, self.parts.len(), self.checksum)?;
        let mut mixed = vec![
            0;
            self.parts
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("expected item"))?
                .len()
        ];
        for i in indexes {
            mixed = xor(
                &mixed,
                self.parts
                    .get(i)
                    .ok_or_else(|| anyhow::anyhow!("expected item"))?,
            );
        }
        Ok(Part {
            sequence: self.current_sequence,
            sequence_count: self.parts.len(),
            message_length: self.message_length,
            checksum: self.checksum,
            data: mixed,
        })
    }

    #[must_use]
    pub fn complete(&self) -> bool {
        self.current_sequence >= self.parts.len()
    }
}

#[derive(Default)]
pub struct Decoder {
    decoded: std::collections::HashMap<usize, Part>,
    received: std::collections::HashSet<Vec<usize>>,
    buffer: std::collections::HashMap<Vec<usize>, Part>,
    queue: std::collections::VecDeque<(usize, Part)>,
    sequence_count: usize,
    message_length: usize,
    checksum: u32,
    fragment_length: usize,
}

impl Decoder {
    pub fn receive(&mut self, part: Part) -> anyhow::Result<bool> {
        if self.complete() {
            return Ok(false);
        }
        if !self.validate(&part) {
            return Ok(false);
        }
        let indexes = part.indexes()?;
        if self.received.contains(&indexes) {
            return Ok(false);
        }
        self.received.insert(indexes);
        if part.is_simple()? {
            self.process_simple(part)?;
        } else {
            self.process_complex(part)?;
        }
        Ok(true)
    }

    pub fn process_simple(&mut self, part: Part) -> anyhow::Result<()> {
        let index = *part
            .indexes()?
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("expected item"))?;
        self.decoded.insert(index, part.clone());
        self.queue.push_back((index, part));
        self.process_queue()?;
        Ok(())
    }

    pub fn process_queue(&mut self) -> anyhow::Result<()> {
        while !self.queue.is_empty() {
            let (index, simple) = self
                .queue
                .pop_front()
                .ok_or_else(|| anyhow::anyhow!("expected item"))?;
            let mut to_process = vec![];
            for indexes in self.buffer.keys() {
                if indexes.iter().any(|idx| idx == &index) {
                    to_process.push(indexes.clone());
                }
            }
            for indexes in to_process {
                let mut part = self
                    .buffer
                    .remove(&indexes)
                    .ok_or_else(|| anyhow::anyhow!("expected item"))?;
                let mut new_indexes = indexes.clone();
                let to_remove = indexes
                    .iter()
                    .position(|x| *x == index)
                    .ok_or_else(|| anyhow::anyhow!("expected item"))?;
                new_indexes.remove(to_remove);
                part.data = xor(&part.data, &simple.data);
                if new_indexes.len() == 1 {
                    self.decoded.insert(
                        *new_indexes
                            .get(0)
                            .ok_or_else(|| anyhow::anyhow!("expected item"))?,
                        part.clone(),
                    );
                    self.queue.push_back((
                        *new_indexes
                            .get(0)
                            .ok_or_else(|| anyhow::anyhow!("expected item"))?,
                        part,
                    ));
                } else {
                    self.buffer.insert(new_indexes, part);
                }
            }
        }
        Ok(())
    }

    pub fn process_complex(&mut self, mut part: Part) -> anyhow::Result<()> {
        let mut indexes = part.indexes()?;
        let mut to_remove = vec![];
        for index in indexes.clone() {
            if self.decoded.keys().any(|k| *k == index) {
                to_remove.push(index);
            }
        }
        if indexes.len() == to_remove.len() {
            return Ok(());
        }
        for remove in to_remove {
            let idx_to_remove = indexes
                .iter()
                .position(|x| *x == remove)
                .ok_or_else(|| anyhow::anyhow!("expected item"))?;
            indexes.remove(idx_to_remove);
            part.data = xor(
                &part.data,
                &self
                    .decoded
                    .get(&remove)
                    .ok_or_else(|| anyhow::anyhow!("expected item"))?
                    .data,
            );
        }
        if indexes.len() == 1 {
            self.decoded.insert(
                *indexes
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("expected item"))?,
                part.clone(),
            );
            self.queue.push_back((
                *indexes
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("expected item"))?,
                part,
            ));
        } else {
            self.buffer.insert(indexes, part);
        }
        Ok(())
    }

    #[must_use]
    pub fn complete(&self) -> bool {
        self.message_length != 0 && self.decoded.len() == self.sequence_count
    }

    pub fn validate(&mut self, part: &Part) -> bool {
        if self.received.is_empty() {
            self.sequence_count = part.sequence_count;
            self.message_length = part.message_length;
            self.checksum = part.checksum;
            self.fragment_length = part.data.len();
        } else {
            if part.sequence_count != self.sequence_count {
                return false;
            }
            if part.message_length != self.message_length {
                return false;
            }
            if part.checksum != self.checksum {
                return false;
            }
            if part.data.len() != self.fragment_length {
                return false;
            }
        }
        true
    }

    pub fn message(&self) -> anyhow::Result<Vec<u8>> {
        if !self.complete() {
            return Err(anyhow::anyhow!("not yet complete"));
        }
        let combined = (0..self.sequence_count)
            .map(|idx| {
                self.decoded
                    .get(&idx)
                    .ok_or_else(|| anyhow::anyhow!("expected item"))
            })
            .collect::<Result<Vec<&Part>, anyhow::Error>>()?
            .iter()
            .map(|p| p.data.clone())
            .fold(vec![], |a, b| [a, b].concat());
        if !combined
            .get(self.message_length..)
            .ok_or_else(|| anyhow::anyhow!("expected item"))?
            .to_vec()
            .iter()
            .all(|x| *x == 0)
        {
            return Err(anyhow::anyhow!("invalid padding detected"));
        }
        Ok(combined
            .get(..self.message_length)
            .ok_or_else(|| anyhow::anyhow!("expected item"))?
            .to_vec())
    }
}

#[derive(Clone, Debug)]
pub struct Part {
    sequence: usize,
    sequence_count: usize,
    message_length: usize,
    checksum: u32,
    data: Vec<u8>,
}

impl std::fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "seqNum:{}, seqLen:{}, messageLen:{}, checksum:{}, data:{}",
            self.sequence,
            self.sequence_count,
            self.message_length,
            self.checksum,
            hex::encode(&self.data)
        )
    }
}

impl Part {
    pub fn from_cbor(cbor: Vec<u8>) -> anyhow::Result<Self> {
        let mut decoder = cbor::Decoder::from_bytes(cbor);
        let items: Vec<cbor::Cbor> = match decoder.items().collect::<Result<_, _>>() {
            Ok(i) => i,
            Err(_) => return Err(anyhow::anyhow!("invalid cbor serialization for Part")),
        };
        if items.len() != 1 {
            return Err(anyhow::anyhow!("invalid cbor length for Part"));
        }
        let items = match items
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("expected item"))?
        {
            cbor::Cbor::Array(a) => a,
            _ => return Err(anyhow::anyhow!("invalid top-level item")),
        };
        if items.len() != 5 {
            return Err(anyhow::anyhow!("invalid cbor array length"));
        }
        let sequence: usize = match items
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("expected item"))?
        {
            cbor::Cbor::Unsigned(t) => match t {
                cbor::CborUnsigned::UInt8(u) => *u as usize,
                cbor::CborUnsigned::UInt16(u) => *u as usize,
                cbor::CborUnsigned::UInt32(u) => *u as usize,
                cbor::CborUnsigned::UInt64(_) => {
                    return Err(anyhow::anyhow!("unexpected item at position 0"))
                }
            },
            _ => return Err(anyhow::anyhow!("unexpected item at position 0")),
        };
        let sequence_count: usize = match items
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("expected item"))?
        {
            cbor::Cbor::Unsigned(t) => match t {
                cbor::CborUnsigned::UInt8(u) => *u as usize,
                cbor::CborUnsigned::UInt16(u) => *u as usize,
                cbor::CborUnsigned::UInt32(u) => *u as usize,
                cbor::CborUnsigned::UInt64(_) => {
                    return Err(anyhow::anyhow!("unexpected item at position 1"))
                }
            },
            _ => return Err(anyhow::anyhow!("unexpected item at position 1")),
        };
        let message_length: usize = match items
            .get(2)
            .ok_or_else(|| anyhow::anyhow!("expected item"))?
        {
            cbor::Cbor::Unsigned(t) => match t {
                cbor::CborUnsigned::UInt8(u) => *u as usize,
                cbor::CborUnsigned::UInt16(u) => *u as usize,
                cbor::CborUnsigned::UInt32(u) => *u as usize,
                cbor::CborUnsigned::UInt64(_) => {
                    return Err(anyhow::anyhow!("unexpected item at position 2"))
                }
            },
            _ => return Err(anyhow::anyhow!("unexpected item at position 2")),
        };
        let checksum: u32 = match items
            .get(3)
            .ok_or_else(|| anyhow::anyhow!("expected item"))?
        {
            cbor::Cbor::Unsigned(t) => match t {
                cbor::CborUnsigned::UInt8(u) => u32::from(*u),
                cbor::CborUnsigned::UInt16(u) => u32::from(*u),
                cbor::CborUnsigned::UInt32(u) => *u,
                cbor::CborUnsigned::UInt64(_) => {
                    return Err(anyhow::anyhow!("unexpected item at position 3"))
                }
            },
            _ => return Err(anyhow::anyhow!("unexpected item at position 3")),
        };
        let data: Vec<u8> = match &items
            .get(4)
            .ok_or_else(|| anyhow::anyhow!("expected item"))?
        {
            cbor::Cbor::Bytes(b) => b.to_vec(),
            _ => return Err(anyhow::anyhow!("unexpected item at position 4")),
        };
        Ok(Self {
            sequence,
            sequence_count,
            message_length,
            checksum,
            data,
        })
    }

    pub fn indexes(&self) -> anyhow::Result<Vec<usize>> {
        choose_fragments(self.sequence, self.sequence_count, self.checksum)
    }

    pub fn is_simple(&self) -> anyhow::Result<bool> {
        Ok(self.indexes()?.len() == 1)
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn cbor(&self) -> anyhow::Result<Vec<u8>> {
        let mut e = cbor::Encoder::from_memory();
        e.encode(vec![cbor::Cbor::Array(vec![
            cbor::Cbor::Unsigned(cbor::CborUnsigned::UInt32(self.sequence as u32)),
            cbor::Cbor::Unsigned(cbor::CborUnsigned::UInt32(self.sequence_count as u32)),
            cbor::Cbor::Unsigned(cbor::CborUnsigned::UInt32(self.message_length as u32)),
            cbor::Cbor::Unsigned(cbor::CborUnsigned::UInt32(self.checksum)),
            cbor::Cbor::Bytes(cbor::CborBytes(self.data.clone())),
        ])])?;
        Ok(e.as_bytes().to_vec())
    }

    #[must_use]
    pub fn sequence_id(&self) -> String {
        format!("{}-{}", self.sequence, self.sequence_count)
    }
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_sign_loss)]
pub fn fragment_length(data_length: usize, max_fragment_length: usize) -> usize {
    let fragment_count = data_length / max_fragment_length + 1;
    (data_length as f64 / fragment_count as f64).ceil() as usize
}

#[must_use]
pub fn partition(mut data: Vec<u8>, fragment_length: usize) -> Vec<Vec<u8>> {
    let mut padding = vec![0; (fragment_length - (data.len() % fragment_length)) % fragment_length];
    data.append(&mut padding);
    data.chunks(fragment_length).map(|c| c.to_vec()).collect()
}

pub fn join(data: Vec<Vec<u8>>, message_length: usize) -> Result<Vec<u8>, &'static str> {
    if data.iter().map(Vec::len).sum::<usize>() < message_length {
        return Err("insufficient data");
    }
    let mut flattened: Vec<u8> = data.into_iter().flatten().collect();
    flattened.truncate(message_length);
    Ok(flattened)
}

pub fn choose_fragments(
    sequence: usize,
    fragment_count: usize,
    checksum: u32,
) -> anyhow::Result<Vec<usize>> {
    if sequence <= fragment_count {
        return Ok(vec![sequence - 1]);
    }
    #[allow(clippy::cast_possible_truncation)]
    let mut seed: Vec<u8> = (sequence as u32).to_be_bytes().to_vec();
    seed.extend((checksum as u32).to_be_bytes().to_vec());
    let mut xoshiro = crate::xoshiro::Xoshiro256::from(seed.as_slice());
    let degree = xoshiro.choose_degree(fragment_count)?;
    let indexes = (0..fragment_count).collect();
    let mut shuffled = xoshiro.shuffled(indexes);
    shuffled.truncate(degree as usize);
    Ok(shuffled)
}

#[must_use]
pub fn xor(v1: &[u8], v2: &[u8]) -> Vec<u8> {
    v1.iter().zip(v2.iter()).map(|(&x1, &x2)| x1 ^ x2).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_length() {
        assert_eq!(fragment_length(12345, 1955), 1764);
        assert_eq!(fragment_length(12345, 30000), 12345);
    }

    #[test]
    fn test_partition_and_join() {
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
            "170010067e2e75ebe2d2904aeb1f89d5dc98cd4a6f2faaa8be6d03354c990fd895a97feb54668473e9d942bb99e196d897e8f1b01625cf48a7b78d249bb4985c065aa8cd1402ed2ba1b6f908f63dcd84b66425df00000000000000000000"
        ];
        assert_eq!(fragments.len(), expected_fragments.len());
        for i in 0..fragments.len() {
            assert_eq!(
                hex::encode(fragments.get(i).unwrap()),
                *expected_fragments.get(i).unwrap()
            );
        }
        let rejoined = join(fragments, message.len()).unwrap();
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
            let mut indexes =
                crate::fountain::choose_fragments(seq_num, fragments.len(), checksum).unwrap();
            indexes.sort_unstable();
            assert_eq!(
                indexes,
                *expected_fragment_indexes.get(seq_num - 1).unwrap()
            );
        }
    }

    #[test]
    fn test_xor() {
        let mut rng = crate::xoshiro::Xoshiro256::from("Wolf");
        let data1 = rng.next_bytes(10);
        assert_eq!(hex::encode(&data1), "916ec65cf77cadf55cd7");
        let data2 = rng.next_bytes(10);
        assert_eq!(hex::encode(&data2), "f9cda1a1030026ddd42e");
        let data3 = xor(&data1, &data2);
        assert_eq!(hex::encode(&data3), "68a367fdf47c8b2888f9");
        assert_eq!(hex::encode(xor(&data3, &data1)), hex::encode(data2));
    }

    #[test]
    fn test_fountain_encoder() {
        let message = crate::xoshiro::test_utils::make_message("Wolf", 256);
        let mut encoder = Encoder::new(&message, 30);
        let expected_parts = vec![
            "seqNum:1, seqLen:9, messageLen:256, checksum:23570951, data:916ec65cf77cadf55cd7f9cda1a1030026ddd42e905b77adc36e4f2d3c",
            "seqNum:2, seqLen:9, messageLen:256, checksum:23570951, data:cba44f7f04f2de44f42d84c374a0e149136f25b01852545961d55f7f7a",
            "seqNum:3, seqLen:9, messageLen:256, checksum:23570951, data:8cde6d0e2ec43f3b2dcb644a2209e8c9e34af5c4747984a5e873c9cf5f",
            "seqNum:4, seqLen:9, messageLen:256, checksum:23570951, data:965e25ee29039fdf8ca74f1c769fc07eb7ebaec46e0695aea6cbd60b3e",
            "seqNum:5, seqLen:9, messageLen:256, checksum:23570951, data:c4bbff1b9ffe8a9e7240129377b9d3711ed38d412fbb4442256f1e6f59",
            "seqNum:6, seqLen:9, messageLen:256, checksum:23570951, data:5e0fc57fed451fb0a0101fb76b1fb1e1b88cfdfdaa946294a47de8fff1",
            "seqNum:7, seqLen:9, messageLen:256, checksum:23570951, data:73f021c0e6f65b05c0a494e50791270a0050a73ae69b6725505a2ec8a5",
            "seqNum:8, seqLen:9, messageLen:256, checksum:23570951, data:791457c9876dd34aadd192a53aa0dc66b556c0c215c7ceb8248b717c22",
            "seqNum:9, seqLen:9, messageLen:256, checksum:23570951, data:951e65305b56a3706e3e86eb01c803bbf915d80edcd64d4d0000000000",
            "seqNum:10, seqLen:9, messageLen:256, checksum:23570951, data:330f0f33a05eead4f331df229871bee733b50de71afd2e5a79f196de09",
            "seqNum:11, seqLen:9, messageLen:256, checksum:23570951, data:3b205ce5e52d8c24a52cffa34c564fa1af3fdffcd349dc4258ee4ee828",
            "seqNum:12, seqLen:9, messageLen:256, checksum:23570951, data:dd7bf725ea6c16d531b5f03254783803048ca08b87148daacd1cd7a006",
            "seqNum:13, seqLen:9, messageLen:256, checksum:23570951, data:760be7ad1c6187902bbc04f539b9ee5eb8ea6833222edea36031306c01",
            "seqNum:14, seqLen:9, messageLen:256, checksum:23570951, data:5bf4031217d2c3254b088fa7553778b5003632f46e21db129416f65b55",
            "seqNum:15, seqLen:9, messageLen:256, checksum:23570951, data:73f021c0e6f65b05c0a494e50791270a0050a73ae69b6725505a2ec8a5",
            "seqNum:16, seqLen:9, messageLen:256, checksum:23570951, data:b8546ebfe2048541348910267331c643133f828afec9337c318f71b7df",
            "seqNum:17, seqLen:9, messageLen:256, checksum:23570951, data:23dedeea74e3a0fb052befabefa13e2f80e4315c9dceed4c8630612e64",
            "seqNum:18, seqLen:9, messageLen:256, checksum:23570951, data:d01a8daee769ce34b6b35d3ca0005302724abddae405bdb419c0a6b208",
            "seqNum:19, seqLen:9, messageLen:256, checksum:23570951, data:3171c5dc365766eff25ae47c6f10e7de48cfb8474e050e5fe997a6dc24",
            "seqNum:20, seqLen:9, messageLen:256, checksum:23570951, data:e055c2433562184fa71b4be94f262e200f01c6f74c284b0dc6fae6673f"
        ];
        for e in expected_parts {
            assert_eq!(encoder.next_part().unwrap().to_string(), e);
        }
    }

    #[test]
    fn test_fountain_encoder_cbor() {
        let message = crate::xoshiro::test_utils::make_message("Wolf", 256);
        let mut encoder = Encoder::new(&message, 30);
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
        for e in expected_parts {
            assert_eq!(hex::encode(encoder.next_part().unwrap().cbor().unwrap()), e);
        }
    }

    #[test]
    fn test_fountain_encoder_is_complete() {
        let message = crate::xoshiro::test_utils::make_message("Wolf", 256);
        let mut encoder = Encoder::new(&message, 30);
        for _ in 0..encoder.parts.len() {
            encoder.next_part().unwrap();
        }
        assert!(encoder.complete());
    }

    #[test]
    fn test_decoder() {
        let seed = "Wolf";
        let message_size = 32767;
        let max_fragment_length = 1000;

        let message = crate::xoshiro::test_utils::make_message(seed, message_size);
        let mut encoder = Encoder::new(&message, max_fragment_length);
        let mut decoder = Decoder::default();
        while !decoder.complete() {
            let part = encoder.next_part().unwrap();
            let _next = decoder.receive(part);
        }
        assert_eq!(decoder.message().unwrap(), message);
    }

    #[test]
    fn test_decoder_skip_some_simple_fragments() {
        let seed = "Wolf";
        let message_size = 32767;
        let max_fragment_length = 1000;

        let message = crate::xoshiro::test_utils::make_message(seed, message_size);
        let mut encoder = Encoder::new(&message, max_fragment_length);
        let mut decoder = Decoder::default();
        let mut skip = false;
        while !decoder.complete() {
            let part = encoder.next_part().unwrap();
            if !skip {
                let _next = decoder.receive(part);
            }
            skip = !skip;
        }
        assert_eq!(decoder.message().unwrap(), message);
    }

    #[test]
    fn test_decoder_receive_return_value() {
        let seed = "Wolf";
        let message_size = 1000;
        let max_fragment_length = 10;

        let message = crate::xoshiro::test_utils::make_message(seed, message_size);
        let mut encoder = Encoder::new(&message, max_fragment_length);
        let mut decoder = Decoder::default();
        let part = encoder.next_part().unwrap();
        assert!(decoder.receive(part.clone()).unwrap());
        // same indexes
        assert!(!decoder.receive(part).unwrap());
        // non-valid
        let mut part = encoder.next_part().unwrap();
        part.checksum += 1;
        assert!(!decoder.receive(part).unwrap());
        // decoder complete
        while !decoder.complete() {
            let part = encoder.next_part().unwrap();
            decoder.receive(part).unwrap();
        }
        let part = encoder.next_part().unwrap();
        assert!(!decoder.receive(part).unwrap());
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
        let part2 = Part::from_cbor(cbor.clone()).unwrap();
        let cbor2 = part2.cbor().unwrap();
        assert_eq!(cbor, cbor2);
    }

    #[test]
    fn test_part_from_cbor_errors() {
        // 0x18 is the first byte value that doesn't directly encode a u8,
        // but implies a following value
        assert_eq!(
            Part::from_cbor(vec![0x18]).unwrap_err().to_string(),
            "invalid cbor serialization for Part"
        );
        // a single cbor item is expected
        assert_eq!(
            Part::from_cbor(vec![0x1, 0x1]).unwrap_err().to_string(),
            "invalid cbor length for Part"
        );
        // the top-level item must be an array
        assert_eq!(
            Part::from_cbor(vec![0x1]).unwrap_err().to_string(),
            "invalid top-level item"
        );
        // the array must be of length five
        assert_eq!(
            Part::from_cbor(vec![0x84, 0x1, 0x2, 0x3, 0x4])
                .unwrap_err()
                .to_string(),
            "invalid cbor array length"
        );
        assert_eq!(
            Part::from_cbor(vec![0x86, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6])
                .unwrap_err()
                .to_string(),
            "invalid cbor array length"
        );
        // the first item must be an unsigned integer
        assert_eq!(
            Part::from_cbor(vec![0x85, 0x41, 0x1, 0x2, 0x3, 0x4, 0x41, 0x1])
                .unwrap_err()
                .to_string(),
            "unexpected item at position 0"
        );
        // the second item must be an unsigned integer
        assert_eq!(
            Part::from_cbor(vec![0x85, 0x1, 0x41, 0x2, 0x3, 0x4, 0x41, 0x1])
                .unwrap_err()
                .to_string(),
            "unexpected item at position 1"
        );
        // the third item must be an unsigned integer
        assert_eq!(
            Part::from_cbor(vec![0x85, 0x1, 0x2, 0x41, 0x3, 0x4, 0x41, 0x1])
                .unwrap_err()
                .to_string(),
            "unexpected item at position 2"
        );
        // the fourth item must be an unsigned integer
        assert_eq!(
            Part::from_cbor(vec![0x85, 0x1, 0x2, 0x3, 0x41, 0x4, 0x41, 0x1])
                .unwrap_err()
                .to_string(),
            "unexpected item at position 3"
        );
        // the fifth item must be byte string
        assert_eq!(
            Part::from_cbor(vec![0x85, 0x1, 0x2, 0x3, 0x4, 0x5])
                .unwrap_err()
                .to_string(),
            "unexpected item at position 4"
        );
        Part::from_cbor(vec![0x85, 0x1, 0x2, 0x3, 0x4, 0x41, 0x5]).unwrap();
    }

    #[test]
    fn test_part_from_cbor_unsigned_types() {
        // u8
        Part::from_cbor(vec![0x85, 0x1, 0x2, 0x3, 0x4, 0x41, 0x5]).unwrap();
        // u16
        Part::from_cbor(vec![
            0x85, 0x19, 0x1, 0x2, 0x19, 0x3, 0x4, 0x19, 0x5, 0x6, 0x19, 0x7, 0x8, 0x41, 0x5,
        ])
        .unwrap();
        // u32
        Part::from_cbor(vec![
            0x85, 0x1a, 0x1, 0x2, 0x3, 0x4, 0x1a, 0x5, 0x6, 0x7, 0x8, 0x1a, 0x9, 0x10, 0x11, 0x12,
            0x1a, 0x13, 0x14, 0x15, 0x16, 0x41, 0x5,
        ])
        .unwrap();
        // u64
        assert_eq!(
            Part::from_cbor(vec![
                0x85, 0x1b, 0x1, 0x2, 0x3, 0x4, 0xa, 0xb, 0xc, 0xd, 0x1a, 0x5, 0x6, 0x7, 0x8, 0x1a,
                0x9, 0x10, 0x11, 0x12, 0x1a, 0x13, 0x14, 0x15, 0x16, 0x41, 0x5,
            ])
            .unwrap_err()
            .to_string(),
            "unexpected item at position 0"
        );
        assert_eq!(
            Part::from_cbor(vec![
                0x85, 0x1a, 0x1, 0x2, 0x3, 0x4, 0x1b, 0x5, 0x6, 0x7, 0x8, 0xa, 0xb, 0xc, 0xd, 0x1a,
                0x9, 0x10, 0x11, 0x12, 0x1a, 0x13, 0x14, 0x15, 0x16, 0x41, 0x5,
            ])
            .unwrap_err()
            .to_string(),
            "unexpected item at position 1"
        );
        assert_eq!(
            Part::from_cbor(vec![
                0x85, 0x1a, 0x1, 0x2, 0x3, 0x4, 0x1a, 0x5, 0x6, 0x7, 0x8, 0x1b, 0x9, 0x10, 0x11,
                0x12, 0xa, 0xb, 0xc, 0xd, 0x1a, 0x13, 0x14, 0x15, 0x16, 0x41, 0x5,
            ])
            .unwrap_err()
            .to_string(),
            "unexpected item at position 2"
        );
        assert_eq!(
            Part::from_cbor(vec![
                0x85, 0x1a, 0x1, 0x2, 0x3, 0x4, 0x1a, 0x5, 0x6, 0x7, 0x8, 0x1a, 0x9, 0x10, 0x11,
                0x12, 0x1b, 0x13, 0x14, 0x15, 0x16, 0xa, 0xb, 0xc, 0xd, 0x41, 0x5,
            ])
            .unwrap_err()
            .to_string(),
            "unexpected item at position 3"
        );
    }
}
