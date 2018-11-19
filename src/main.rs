mod configuration;
use std::collections::HashMap;

fn main() {
    println!("Hello, world!");
    let mut a = configuration::Configuration::new();
    a.load();
    println!("{}", a.get("pepe"));
}
