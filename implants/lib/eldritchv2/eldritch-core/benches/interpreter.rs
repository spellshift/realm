use criterion::{criterion_group, criterion_main, Criterion};
use eldritch_core::Interpreter;

fn bench_arithmetic(c: &mut Criterion) {
    c.bench_function("interpreter_arithmetic", |b| {
        b.iter(|| {
            let mut interpreter = Interpreter::new();
            interpreter.interpret("1 + 1").unwrap();
        })
    });
}

fn bench_variable_assignment(c: &mut Criterion) {
    c.bench_function("interpreter_var_assign", |b| {
        b.iter(|| {
            let mut interpreter = Interpreter::new();
            interpreter.interpret("x = 10; y = x * 2").unwrap();
        })
    });
}

fn bench_loop(c: &mut Criterion) {
    c.bench_function("interpreter_loop", |b| {
        b.iter(|| {
            let mut interpreter = Interpreter::new();
            let code = "
sum = 0
for i in range(10):
    sum = sum + i
";
            interpreter.interpret(code).unwrap();
        })
    });
}

fn bench_function_call(c: &mut Criterion) {
    c.bench_function("interpreter_function", |b| {
        b.iter(|| {
            let mut interpreter = Interpreter::new();
            let code = "
fn add(a, b):
    return a + b

add(5, 10)
";
            interpreter.interpret(code).unwrap();
        })
    });
}

// Separate benchmarks to measure overhead without initialization if needed.
// However, since `Interpreter` holds state, creating a new one for each iteration
// is the safest way to ensure isolation, though it includes startup cost.
// To measure just execution, we would need to pre-initialize, but `interpret` parses every time.
// Given the current API, benchmarking `interpret` covers the full cycle users experience.

criterion_group!(
    benches,
    bench_arithmetic,
    bench_variable_assignment,
    bench_loop,
    bench_function_call
);
criterion_main!(benches);
