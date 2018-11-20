use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

/*==============================================================================
 * Server data
 *
 */

pub struct ServerData {
  processing_power: u64
}

impl ServerData {

  pub fn get_processing_power(&self) -> u64 {
    self.processing_power
  }

}

/*==============================================================================
 * Client data
 *
 */

pub struct ClientData {
  work_generation_rate: u64,
  workshare: Vec<f64>
}

impl ClientData {

  pub fn get_workshare(&self) -> &Vec<f64> {
    &self.workshare
  }

  pub fn get_work_generation_rate(&self) -> u64 {
    self.work_generation_rate
  }

}

/*==============================================================================
 * Configuration
 *
 */

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
    // Copiamos el archivo config al directorio del ejecutable
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

  pub fn get_server_dataset(&self) -> Vec<ServerData> {
    let mut dataset = Vec::new();

    /*
     * Para configurar n servidores definimos la variable S:
     *
     *   S = p1;p2;p3;...;pn
     *
     * ... donde pk es el poder de procesamiento (u64), en cuadrantes por
     * unidad de tiempo, del k-ésimo servidor.
     *
     */

    let raw_data = self.get("S");
    let raw_dataset = raw_data.split(';');

    for p in raw_dataset {
      dataset.push(ServerData {
        processing_power: p.parse::<u64>().unwrap()
      });
    }

    return dataset;
  }

  pub fn get_client_dataset(&self) -> Vec<ClientData> {
    let mut dataset = Vec::new();

    /*
     * Para configurar m clientes definimos la variable C:
     *
     *   C=x1,p11,p12,...,p1n;x2,p21,p22,...,p2n;...;xm,pm1,pm2,...,pmn
     *
     * donde xj es la cantidad de cuadrantes a procesar generados por unidad
     * de tiempo por el j-ésimo cliente, y pjk es la fracción de trabajo que
     * el j-ésimo cliente envía al k-ésimo servidor.
     *
     */

    let raw_data = self.get("C");
    let raw_dataset = raw_data.split(';');

    for raw_client_data in raw_dataset {
      let mut workshare = Vec::new();

      // Tomamos los elementos de la tupla.
      let raw = raw_client_data.split(',').collect::<Vec<&str>>();

      // Parseamos la tasa de generación de trabajo.
      let work_generation_rate: u64 = raw[0].parse::<u64>().unwrap();

      // Parseamos la distribución de trabajo.
      for i in 1..raw.len() {
        workshare.push(raw[i].parse::<f64>().unwrap());
      }

      dataset.push(ClientData {
        work_generation_rate: work_generation_rate,
        workshare: workshare
      });
    }

    return dataset;
  }

}
