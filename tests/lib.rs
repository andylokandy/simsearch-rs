use std::fs::File;
use std::io::Read;

use json;
use simsearch::{SearchOptions, SimSearch};

#[macro_use(quickcheck)]
extern crate quickcheck_macros;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref ENGINE: SimSearch<String> = populate_engine();
}

fn populate_engine() -> SimSearch<String> {
    let mut engine = SimSearch::new_with(SearchOptions::new().stop_whitespace(true));

    let mut file = File::open("./books.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let j = json::parse(&content).unwrap();

    for title in j.members() {
        engine.insert(title.as_str().unwrap().to_string(), title.as_str().unwrap());
    }

    engine
}

#[quickcheck]
fn test_quickcheck(tokens: Vec<String>) {
    ENGINE.search(&tokens.join(" "));
}
