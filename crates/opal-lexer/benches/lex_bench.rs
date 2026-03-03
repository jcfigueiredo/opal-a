use criterion::{Criterion, criterion_group, criterion_main};
use opal_lexer::lex;

fn bench_lex_hello_world(c: &mut Criterion) {
    let source = r#"
name = "Opal"
print(f"Hello, {name}!")
"#;
    c.bench_function("lex_hello_world", |b| {
        b.iter(|| lex(std::hint::black_box(source)))
    });
}

fn bench_lex_medium(c: &mut Criterion) {
    let source = r#"
def factorial(n: Int) -> Int
  if n <= 1 then 1 else n * factorial(n - 1) end
end

result = factorial(10)
print(f"Result: {result}")
"#;
    c.bench_function("lex_medium_program", |b| {
        b.iter(|| lex(std::hint::black_box(source)))
    });
}

fn bench_lex_function(c: &mut Criterion) {
    let source = r#"
def factorial(n: Int) -> Int
  if n <= 1 then 1 else n * factorial(n - 1) end
end

def fib(n)
  if n <= 1 then n else fib(n - 1) + fib(n - 2) end
end

print(factorial(10))
print(fib(10))
"#;
    c.bench_function("lex_function_program", |b| {
        b.iter(|| lex(std::hint::black_box(source)))
    });
}

criterion_group!(
    benches,
    bench_lex_hello_world,
    bench_lex_medium,
    bench_lex_function
);
criterion_main!(benches);
