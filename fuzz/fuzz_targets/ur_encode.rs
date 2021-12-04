use honggfuzz::fuzz;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            let max_length = 1 + *data.get(0).unwrap() as usize;
            let mut encoder = ur::Encoder::new(data, max_length, "bytes").unwrap();
            let mut decoder = ur::Decoder::default();
            for _ in 0..encoder.fragment_count() {
                let fragment = encoder.next_fragment().unwrap();
                decoder.receive(&fragment).unwrap();
            }
            assert_eq!(decoder.message().unwrap(), data);
        });
    }
}
