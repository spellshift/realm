use criterion::{criterion_group, criterion_main, Criterion};
use eldritch_core::Interpreter;

// Helper to benchmark a specific builtin call
fn bench_builtin(c: &mut Criterion, name: &str, code: &str) {
    c.bench_function(&format!("builtin_{name}"), |b| {
        b.iter(|| {
            let mut interpreter = Interpreter::new();
            interpreter.interpret(code).unwrap();
        })
    });
}

fn bench_builtins(c: &mut Criterion) {
    // Basic types and conversions
    bench_builtin(c, "int", "int('123')");
    bench_builtin(c, "float", "float('123.456')");
    bench_builtin(c, "str", "str(123)");
    bench_builtin(c, "bool", "bool(1)");
    bench_builtin(c, "bytes", "bytes([65, 66, 67])");
    bench_builtin(c, "type", "type(123)");

    // Collections constructors
    bench_builtin(c, "list", "list((1, 2, 3))");
    bench_builtin(c, "tuple", "tuple([1, 2, 3])");
    bench_builtin(c, "set", "set([1, 2, 3])");
    bench_builtin(c, "dict", "dict(a=1, b=2)");

    // Math
    bench_builtin(c, "abs", "abs(-100)");
    bench_builtin(c, "max", "max([1, 2, 3, 10, 5])");
    bench_builtin(c, "min", "min([1, 2, 3, 10, 5])");

    // Logic
    bench_builtin(c, "all", "all([True, True, True])");
    bench_builtin(c, "any", "any([False, True, False])");

    // Iteration & Sequence operations
    bench_builtin(c, "len", "len([1, 2, 3, 4, 5])");
    bench_builtin(c, "range", "range(100)"); // Returns a list/iterator, doesn't iterate
    bench_builtin(c, "enumerate", "enumerate([1, 2, 3])");
    bench_builtin(c, "reversed", "reversed([1, 2, 3])");
    bench_builtin(c, "sorted", "sorted([3, 1, 2])");
    bench_builtin(c, "zip", "zip([1, 2], [3, 4])");

    // Inspection / Debugging
    bench_builtin(c, "dir", "dir()");
    bench_builtin(c, "repr", "repr([1, 2, 3])");

    // Output (Use with no_std to suppress actual stdout writing if implementation supports it)
    bench_builtin(c, "print", "print('hello')");
    bench_builtin(c, "pprint", "pprint([1, 2, 3])");

    // System / Meta
    bench_builtin(c, "libs", "libs()");
    bench_builtin(c, "builtins", "builtins()");

    // Assertions
    bench_builtin(c, "assert", "assert(True)");
    bench_builtin(c, "assert_eq", "assert_eq(1, 1)");

    // Fail (We expect an error, but benchmarking the call overhead)
    // Note: unwrap() would panic, so we handle the result
    c.bench_function("builtin_fail", |b| {
        b.iter(|| {
            let mut interpreter = Interpreter::new();
            let _ = interpreter.interpret("fail('error')");
        })
    });
}

criterion_group!(benches, bench_builtins);
criterion_main!(benches);
