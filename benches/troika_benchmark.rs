#[macro_use]
extern crate criterion;

use criterion::Criterion;
use troika_rust::troika::Troika;
use troika_rust::ftroika::Ftroika;
use troika_rust::stroika::{Stroika, Strit};

fn basic_troika() {
    let mut troika = Troika::default();
    let input = [0u8; 242];
    let mut output = [0u8; 243];

    troika.absorb(&input);
    troika.squeeze(&mut output);
}

fn basic_ftroika() {
    let mut ftroika = Ftroika::default();
    let input = [0u8; 8019];
    let mut output = [0u8; 243];

    ftroika.absorb(&input);
    ftroika.finalize();
    ftroika.squeeze(&mut output);
}

fn basic_stroika() {
    let mut troika = Stroika::default();
    let input = [Strit::ZERO(); 8019];
    let mut output = [Strit::ZERO(); 243];

    troika.absorb(&input);
    troika.finalize();
    troika.squeeze(&mut output);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("troika with input of 8019 zeros", |b| {
        b.iter(|| basic_troika())
    });
}

fn criterion_benchmark_f(c: &mut Criterion) {
    c.bench_function("Ftroika with input of 8019 zeros", |b| {
        b.iter(|| basic_ftroika())
    });
}
fn criterion_benchmark_s(c: &mut Criterion) {
    c.bench_function("Stroika with input of 8019 zeros", |b| {
        b.iter(|| basic_stroika())
    });
}

criterion_group!(benches, criterion_benchmark, criterion_benchmark_f, criterion_benchmark_s);
criterion_main!(benches);
