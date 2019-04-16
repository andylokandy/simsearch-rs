use std::fs::File;
use std::io::Read;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use json;
use simsearch::SimSearch;

/// Loads content of a 'books.json' file into a JsonValue.
fn load_content() -> json::JsonValue {
    let mut file = File::open("./books.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    json::parse(&content).unwrap()
}

fn bench_engine(c: &mut Criterion) {
    c.bench_function("add 100", |bencher| {
        let j = load_content();

        let mut books: Vec<(&str, &str)> = Vec::new();

        for book in j.members() {
            let title = book["title"].as_str().unwrap();
            books.push((title, title));
        }

        bencher.iter_batched_ref(
            || SimSearch::new(),
            |engine| {
                for (title, terms) in &books {
                    engine.insert(title, terms);
                }
            },
            BatchSize::SmallInput,
        )
    });
    c.bench_function("search", |bencher| {
        let mut engine = SimSearch::new();
        let j = load_content();

        for book in j.members() {
            let title = book["title"].as_str().unwrap();
            engine.insert(title, title);
        }

        bencher.iter(|| engine.search("odl sea"));
    });
}

criterion_group!(benches, bench_engine);
criterion_main!(benches);
