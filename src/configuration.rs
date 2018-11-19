use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};

const DEFAULT: &str = "1";

pub struct Configuration{
    pub data: HashMap<String, String>,
}

impl Configuration {
    pub fn new() -> Configuration{
        Configuration{
            data: HashMap::new()
        }
    }

    pub fn load(&mut self) {
        // Copiar el archivo config al directorio del ejecutable
        let f = File::open("config");
        let _f = match f {
            Ok(file) => {
                for line in BufReader::new(file).lines() {
                    let new_line = line.unwrap();
                    let vec: Vec<&str> = new_line.split("=").collect();
                    self.data.insert(vec[0].to_string(), vec[1].to_string());
                }
            },
            Err(_err) => {
                println!("Error al cargar archivo");
            },
        };
    }
    pub fn get(&self, key: &str) -> &str {
        if !self.data.contains_key(key) {
            DEFAULT
        } else {
            self.data.get(key).unwrap()
        }
    }
}