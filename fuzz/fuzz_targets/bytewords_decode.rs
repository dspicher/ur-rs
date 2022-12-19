use honggfuzz::fuzz;

use ur::bytewords::{decode, Style};

fn main() {
    loop {
        fuzz!(|data: &str| {
            decode(data, Style::Minimal).ok();
            decode(data, Style::Standard).ok();
            decode(data, Style::Uri).ok();
        });
    }
}
