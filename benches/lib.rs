use std::fs::File;
use std::io::Read;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use json;
use simsearch::{SimSearch, SearchOptions};

/// Loads content of a 'books.json' file into a JsonValue.
fn load_content() -> json::JsonValue {
    let mut file = File::open("./books.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    json::parse(&content).unwrap()
}

fn bench_engine(c: &mut Criterion) {
    c.bench_function("add books", |bencher| {
        let j = load_content();

        let mut books: Vec<(&str, &str)> = Vec::new();

        for title in j.members() {
            books.push((title.as_str().unwrap(), title.as_str().unwrap()));
        }

        bencher.iter_batched_ref(
            || SimSearch::new(),
            |engine| {
                for (title, terms) in &books {
                    engine.insert(*title, *terms);
                }
            },
            BatchSize::SmallInput,
        )
    });
    c.bench_function("search_jaro_winkler", |bencher| {
        let mut engine = SimSearch::new();
        let j = load_content();

        for title in j.members() {
            engine.insert(title.as_str().unwrap(), title.as_str().unwrap());
        }

        bencher.iter(|| engine.search("odl sea"));
    });
    c.bench_function("search_levenshtein", |bencher| {
        let options = SearchOptions::new().levenshtein(true);
        let mut engine = SimSearch::new_with(options);
        let j = load_content();

        for title in j.members() {
            engine.insert(title.as_str().unwrap(), title.as_str().unwrap());
        }

        bencher.iter(|| engine.search("odl sea"));
    });
}

criterion_group!(benches, bench_engine);
criterion_main!(benches);
