use qrcode::QrCode;

use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, long)]
    payload: String,
}

fn main() {
    let opt = Opt::from_args();
    let mut encoder = ur::Encoder::new(opt.payload.as_bytes(), 5, "bytes").unwrap();
    let mut stdout = std::io::stdout();
    loop {
        let ur = encoder.next_part().unwrap();
        let code = QrCode::new(&ur).unwrap();
        let string = code
            .render::<char>()
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build();
        stdout
            .write_all(format!("{}\n", string).as_bytes())
            .unwrap();
        stdout
            .write_all(format!("{}\n\n\n\n", ur).as_bytes())
            .unwrap();
        stdout.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
