use jsonparser::*;
use std::io::*;

fn main() {
    std::io::stdin().lock().lines().for_each(|l| {
        let line = l.unwrap();
        let parsed = line.parse::<JSONValue>();
        match parsed {
            Ok(v) => println!("{:?}", v),
            Err(e) => println!("{:?}", e)
        }
    });
}
