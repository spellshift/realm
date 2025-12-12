use criterion::{Criterion, criterion_group, criterion_main};
use eldritch_repl::parser::InputParser;
use eldritch_repl::{Input, Repl};

fn benchmark_input_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("InputParser");

    // Benchmark simple character parsing
    let simple_input = b"print('hello world')";
    group.bench_function("simple_chars", |b| {
        b.iter(|| {
            let mut parser = InputParser::new();
            parser.parse(simple_input)
        })
    });

    // Benchmark ANSI CSI sequences (Arrow keys)
    let csi_input = b"\x1b[A\x1b[B\x1b[C\x1b[D";
    group.bench_function("csi_sequences", |b| {
        b.iter(|| {
            let mut parser = InputParser::new();
            parser.parse(csi_input)
        })
    });

    // Benchmark split packets simulation
    // We simulate parsing byte by byte to test state machine overhead
    let split_input = b"\x1b[A";
    group.bench_function("split_packets", |b| {
        b.iter(|| {
            let mut parser = InputParser::new();
            for byte in split_input {
                parser.parse(&[*byte]);
            }
        })
    });

    group.finish();
}

fn benchmark_repl_handle_input(c: &mut Criterion) {
    let mut group = c.benchmark_group("Repl::handle_input");

    // Benchmark typing speed
    group.bench_function("typing_speed", |b| {
        b.iter_batched(
            Repl::new,
            |mut repl| {
                for c in "print('hello')".chars() {
                    repl.handle_input(Input::Char(c));
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Benchmark history search (worst case: linear scan)
    group.bench_function("history_search", |b| {
        // Setup a repl with large history
        let mut history = Vec::new();
        for i in 0..1000 {
            history.push(format!("command_{} arg_{}", i, i));
        }

        b.iter_batched(
            || {
                let mut repl = Repl::new();
                repl.load_history(history.clone());
                repl.handle_input(Input::HistorySearch); // Start search
                repl
            },
            |mut repl| {
                // Search for "999" (at end of history)
                repl.handle_input(Input::Char('9'));
                repl.handle_input(Input::Char('9'));
                repl.handle_input(Input::Char('9'));
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Benchmark input that triggers macro expansion (handled in Repl::handle_enter -> expand_macros)
    // "!ls" triggers macro expansion.
    group.bench_function("macro_expansion", |b| {
        b.iter_batched(
            || {
                let mut repl = Repl::new();
                // Type "!ls"
                repl.handle_input(Input::Char('!'));
                repl.handle_input(Input::Char('l'));
                repl.handle_input(Input::Char('s'));
                repl
            },
            |mut repl| {
                // Press Enter to trigger expansion
                repl.handle_input(Input::Enter);
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, benchmark_input_parser, benchmark_repl_handle_input);
criterion_main!(benches);
