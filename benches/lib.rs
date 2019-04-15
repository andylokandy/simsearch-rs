#![feature(test)]

extern crate test;

use std::fs::File;
use std::io::Read;
use test::Bencher;

use json;
use simsearch::SimSearch;

#[bench]
fn bench_add_100(b: &mut Bencher) {
    let mut engine = SimSearch::new();

    let mut file = File::open("./books.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let j = json::parse(&content).unwrap();

    let mut books: Vec<(String, &str)> = Vec::new();

    for book in j.members() {
        let title = book["title"].as_str().unwrap();
        books.push((title.to_owned(), title));
    }

    b.iter(|| {
        for (title, terms) in books.clone() {
            engine.insert(title, terms);
        }
    });
}

#[bench]
fn bench_search(b: &mut Bencher) {
    let mut engine = SimSearch::new();

    let mut file = File::open("./books.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let j = json::parse(&content).unwrap();

    for book in j.members() {
        let title = book["title"].as_str().unwrap();
        engine.insert(title.to_owned(), title);
    }

    b.iter(|| engine.search("odl sea"));
}
