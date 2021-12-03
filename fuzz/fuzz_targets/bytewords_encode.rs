use honggfuzz::fuzz;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            for style in [
                ur::bytewords::Style::Standard,
                ur::bytewords::Style::Uri,
                ur::bytewords::Style::Minimal,
            ] {
                let encoded = ur::bytewords::encode(data, &style).unwrap();
                let decoded = ur::bytewords::decode(&encoded, &style).unwrap();
                assert_eq!(data, decoded);
            }
        });
    }
}
