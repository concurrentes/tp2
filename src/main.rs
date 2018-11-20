mod configuration;
use configuration::ServerData;
use configuration::ClientData;

use std::sync::mpsc;
use std::thread;
use std::time;

/*==============================================================================
 * Packet
 *------------------------------------------------------------------------------
 *
 */

struct Packet {
  from: VirtualLink,
  workload: u64
}

impl Packet {

  fn from_iface(iface: &NetworkInterface, workload: u64) -> Packet {
    Packet {
      from: (*iface).get_virtual_link(),
      workload: workload
    }
  }

  fn answer_me_at(tx: &mpsc::Sender<Packet>, workload: u64) -> Packet {
    Packet {
      from: VirtualLink::linked_to(tx),
      workload: workload
    }
  }

}

/*==============================================================================
 * VirtualLink
 *------------------------------------------------------------------------------
 *
 */

struct VirtualLink {
  s: mpsc::Sender<Packet>
}

impl VirtualLink {

  fn to_iface(interface: &NetworkInterface) -> VirtualLink {
    VirtualLink {
      s: (*interface).s.clone()
    }
  }

  fn linked_to(tx: &mpsc::Sender<Packet>) -> VirtualLink {
    VirtualLink {
      s: (*tx).clone()
    }
  }

  fn send_through(&self, packet: Packet) {
    self.s.send(packet).unwrap()
  }

}

/*==============================================================================
 * Network Interface
 *------------------------------------------------------------------------------
 *
 */

struct NetworkInterface {
  s: mpsc::Sender<Packet>,
  r: mpsc::Receiver<Packet>
}

impl NetworkInterface {

  fn new() -> NetworkInterface {
    let (tx, rx) = mpsc::channel();

    NetworkInterface {
      s: tx,
      r: rx
    }
  }

  fn read(&self) -> Packet {
    self.r.recv().unwrap()
  }

  fn get_virtual_link(&self) -> VirtualLink {
    VirtualLink::to_iface(self)
  }

}

/*==============================================================================
 * Host
 *
 */

 struct Host {
   nic: NetworkInterface,
 }

 impl Host {

   fn new() -> Host {
     Host {
       nic: NetworkInterface::new(),
     }
   }

   fn get_virtual_link(&self) -> VirtualLink {
     self.nic.get_virtual_link()
   }

 }

/*==============================================================================
 * Server
 *
 */

struct Server {
  host: Host,
  processing_power: u64
}

impl Server {

  fn new(server_data: ServerData) -> Server {
    Server {
      host: Host::new(),
      processing_power: server_data.get_processing_power()
    }
  }

  fn get_virtual_link(&self) -> VirtualLink {
    self.host.get_virtual_link()
  }

  fn run(self) {
    let rx = self.host.nic.r;
    let tx = self.host.nic.s;

    for message in rx {

       // Obtenemos la cantidad de cuadrantes a procesar.
       let workload = message.workload;

       /*
        * Procesamos los cuadrantes.
        *
        * El workload tiene unidades de trabajo. El poder de procesamiento
        * tiene unidades de trabajo por segundo. El sleep time tiene unidades
        * de milisegundos.
        *
        * Por ejemplo, un servidor recibe 5 unidades de trabajo desde el
        * cliente. El servidor puede procesar dos unidades de trabajo por
        * segundo. El hilo dormirá entonces 2500 milisegundos simulando
        * el procesamiento de la carga. Para acelerar o relentizar
        * la simulación, podemos ajustar el factor global de velocidad;
        * por ejemplo, si el factor global es 2.0, en vez de dormir los 2500
        * milisegundos dormiría 1250.
        *
        */
       let sleep_time = (1000*workload)/self.processing_power;
       let sleep_time_scaled = ((sleep_time as f64)/GLOBAL_SPEED) as u64;
       thread::sleep(time::Duration::from_millis(sleep_time_scaled));

       // Devolvemos el ACK.
       let response = Packet::answer_me_at(&tx, 0);
       message.from.send_through(response);
    }
  }

}

/*==============================================================================
 * Client
 *
 */

struct Target {
  virtual_link: VirtualLink,
  weight: f64
}

struct Client {
  host: Host,
  distribution_scheme: Vec<Target>,
  work_generation_rate: u64
}

impl Client {

  fn new(servers: &Vec<Server>, client_data: ClientData) -> Client {
    let workshare: &Vec<f64> = client_data.get_workshare();
    let mut distribution = Vec::new();

    for i in 0..servers.len() {
      distribution.push(Target {
        virtual_link: servers[i].get_virtual_link(),
        weight: workshare[i]
      });
    }

    Client {
      host: Host::new(),
      distribution_scheme: distribution,
      work_generation_rate: client_data.get_work_generation_rate()
    }
  }

  fn run(self) {
    /*
     * Cada cierta cantidad de tiempo, el observatorio genera x cuadrantes.
     * A partir de ahí itera por la lista de servidores distribuyendo los
     * cuadrantes según los factores de distribución (e.g., si debe enviar
     * una fracción p_k de los cuadrantes al servidor k, enviará p_k*x
     * cuadrantes al servidor k).
     *
     * Habiendo enviado los mensajes, simplemente espera las respuestas.
     * Suponiendo alternativamente que hay que seguir generando cuadrantes
     * mientras se toman fotos, se pueden tener internamente dos threads,
     * uno acumulando cuadrantes y otro tomando cuadrantes y distribuyendolos.
     *
     * Para medir el tiempo de respuesta del observatorio se puede ir
     * calculando una media móvil, tomando el tiempo que tarda en responder
     * cada servidor.
     */

     let targets = &self.distribution_scheme;

     loop {
       // TODO: Generar x cuadrantes.
       let x = self.work_generation_rate;

       // Distribuimos los x cuadrantes generados.
       for target in targets {
         let workload = ((x as f64)*(target.weight)) as u64;
         let packet = Packet::from_iface(&self.host.nic, workload);
         target.virtual_link.send_through(packet);
       }

       // Esperamos la respuesta de cada servidor.
       for _d in targets {
         let _response = self.host.nic.read();
       }
     }
  }

}

/*==============================================================================
 * Main
 *
 */

const GLOBAL_SPEED: f64 = 1.0;

fn main() {

  /*
   * Cargamos la configuración. La configuración es un archivo de texto con
   * pares clave-valor. El objeto de configuración puede usarse como
   *
   *     configuration.get("clave") // retorna el valor asociado a "clave".
   */
  let mut configuration = configuration::Configuration::new();
  configuration.load();

  let mut threads = Vec::new();
  let mut servers: Vec<Server> = Vec::new();
  let mut clients: Vec<Client> = Vec::new();

  // Inicializamos los servidores.
  let server_data: Vec<ServerData> = configuration.get_server_dataset();

  for d in server_data {
    servers.push(Server::new(d));
  }

  // Inicializamos los clientes.
  let client_data: Vec<ClientData> = configuration.get_client_dataset();

  for c in client_data {
    clients.push(Client::new(&servers, c));
  }

  for server in servers {
    let th = thread::spawn(move || {
      server.run();
    });
    threads.push(th);
  }

  for client in clients {
    let th = thread::spawn(move || {
      client.run();
    });
    threads.push(th);
  }

  for th in threads {
    th.join().unwrap();
  }

}
