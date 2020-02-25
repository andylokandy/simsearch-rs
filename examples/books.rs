use std::fs::File;
use std::io::{self, Read as _};
use std::time::Instant;

use json;
use simsearch::SimSearch;

fn main() -> io::Result<()> {
    let mut engine = SimSearch::new();

    let mut file = File::open("./books.json")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let j = json::parse(&content).unwrap();

    for title in j.members() {
        engine.insert(title.as_str().unwrap(), title.as_str().unwrap());
    }

    println!("Please input a query string and hit enter (e.g 'old man'):",);

    loop {
        let mut pattern = String::new();
        io::stdin()
            .read_line(&mut pattern)
            .expect("failed to read from stdin");

        let start = Instant::now();
        let res = engine.search(&pattern);
        let end = Instant::now();

        println!("pattern: {:?}", pattern.trim());
        println!("results: {:?}", res);
        println!("time: {:?}", end - start);
    }
}
