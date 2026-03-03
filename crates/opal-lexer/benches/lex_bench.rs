use criterion::{criterion_group, criterion_main, Criterion};

fn bench_lexer_placeholder(c: &mut Criterion) {
    c.bench_function("lex_placeholder", |b| {
        b.iter(|| {
            // Will benchmark actual lexer once implemented
            std::hint::black_box("placeholder")
        });
    });
}

criterion_group!(benches, bench_lexer_placeholder);
criterion_main!(benches);
