use jsonparser::*;
use std::io::*;

fn main() {
    std::io::stdin().lock().lines().for_each(|l| {
        let line = l.unwrap();
        let parsed: JSONValue = line.parse().unwrap();
        println!("{:?}", parsed);
    });
}
