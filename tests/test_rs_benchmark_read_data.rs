//! Run `cargo test --release --test test_rs_benchmark_read_data -- --ignored --nocapture`.
//! Includes decoding, parsing, DataFrame conversion, and temporary Parquet writes.
#![cfg(not(feature = "datamap-only"))]

use std::{hint::black_box, io::Write, time::Instant};
use triple_s::{Datamap, QuestionOptions, QuestionType, read_data};

const ROWS: usize = 50_000;
const COLUMNS: usize = 3_000;
const ROUNDS: usize = 3;


fn benchmark_read_data(extension: &str, rows: usize, rounds: usize) {
    let mut datamap = Datamap::empty();
    for i in 0..COLUMNS as u32 {
        let (start, end) = if extension == "asc" {
            (i * 8 + 1, i * 8 + 8)
        } else {
            (i + 1, i + 1)
        };
        datamap.record_mut().add_question(
            QuestionOptions::new(&format!("Q{i}"))
                .with_question_type(QuestionType::Quantity)
                .with_location(start, Some(end))
                .finish()
                .unwrap(),
        );
    }
    let mut file = tempfile::Builder::new()
        .suffix(&format!(".{extension}"))
        .tempfile()
        .unwrap();
    {
        let mut writer = std::io::BufWriter::new(file.as_file_mut());
        let record = if extension == "asc" {
            "123.456 ".repeat(COLUMNS)
        } else {
            vec!["123.456"; COLUMNS].join(",")
        };
        for _ in 0..rows {
            writeln!(writer, "{record}").unwrap();
        }
        writer.flush().unwrap();
    }
    for threads in [1, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        // Warm metadata, OS file caches, and worker pools before measuring.
        pool.install(|| {
            black_box(read_data(&datamap, file.path()).unwrap());
        });
        let mut samples = Vec::with_capacity(rounds);
        for _ in 0..rounds {
            let start = Instant::now();
            pool.install(|| {
                black_box(read_data(&datamap, file.path()).unwrap());
            });
            samples.push(start.elapsed());
        }
        samples.sort();
        let median = samples[samples.len() / 2].as_secs_f64();
        println!(
            "{extension}: {rows} rows x {COLUMNS} fields, {threads} workers: median \
             {median:.4}s ({:.0} rows/s), {rounds} rounds",
            rows as f64 / median
        );
    }
}

#[test]
#[ignore = "performance benchmark; run explicitly in release mode"]
fn benchmark_fixed_width_read_data() {
    benchmark_read_data("asc", ROWS, ROUNDS);
}

#[test]
#[ignore = "performance benchmark; run explicitly in release mode"]
fn benchmark_csv_read_data() {
    benchmark_read_data("csv", ROWS, ROUNDS);
}
