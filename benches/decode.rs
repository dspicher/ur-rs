use criterion::{criterion_group, criterion_main, Criterion};
use ur::decode;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("decode bytes", |b| b.iter(|| decode(std::hint::black_box("ur:bytes/hdeymejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtgwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsdwkbrkch"))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
