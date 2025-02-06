use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ur::decode;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("decode bytes", |b| b.iter(|| decode(black_box("ur:bytes/hdeymejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtgwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsdwkbrkch"))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
