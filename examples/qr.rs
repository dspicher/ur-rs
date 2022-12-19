use qrcode::QrCode;

use std::io::Write;

fn main() {
    let mut encoder =
        ur::Encoder::new(std::env::args().last().unwrap().as_bytes(), 5, "bytes").unwrap();
    let mut stdout = std::io::stdout();
    loop {
        let ur = encoder.next_part().unwrap();
        let code = QrCode::new(&ur).unwrap();
        let string = code
            .render::<char>()
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build();
        stdout.write_all(format!("{string}\n").as_bytes()).unwrap();
        stdout
            .write_all(format!("{ur}\n\n\n\n").as_bytes())
            .unwrap();
        stdout.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
