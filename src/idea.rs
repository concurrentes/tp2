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

  fn new(processing_power: u64) -> Server {
    Server {
      host: Host::new(),
      processing_power: processing_power
    }
  }

  fn get_virtual_link(&self) -> VirtualLink {
    self.host.get_virtual_link()
  }

  fn run(self) {
    let rx = self.host.nic.r;
    let tx = self.host.nic.s;

    for message in rx {
      /*
       * TODO
       *
       * Procesar mensaje; en el mensaje debería llegar la cantidad de
       * cuadrantes, y el enlace para responder al observatorio que envió
       * el pedido.
       *
       * El servidor debería tener un parámetro de configuración que indica
       * que puede procesar x cuadrantes por unidad de tiempo; entonces,
       * habría que ejecutar un sleep adecuado a la cantidad de cuadrantes
       * recibida.
       *
       * Para hacer la simulación más eficiente, se puede tener también un
       * factor global de speed up que permite ajustar los sleep globalmente,
       * para que las unidades tengan sentido pero que la simulación pueda
       * ejecutarse rápidamente.
       *
       */

       // Obtenemos la cantidad de cuadrantes a procesar.
       let workload = message.workload;

       /*
        * Procesamos los cuadrantes.
        *
        * TODO: Cargar factor global desde un archivo. El factor global es para
        * que la configuración pueda hacerse en unidades coherentes, pero que
        * la simulación no deba ser necesariamente en tiempo real.
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
  distribution_scheme: Vec<Target>
}

impl Client {

  fn new(servers: &Vec<Server>, workshare: &Vec<f64>) -> Client {
    let mut distribution = Vec::new();

    for i in 0..servers.len() {
      distribution.push(Target {
        virtual_link: servers[i].get_virtual_link(),
        weight: workshare[i]
      });
    }

    Client {
      host: Host::new(),
      distribution_scheme: distribution
    }
  }

  fn run(self) {
    /*
     * TODO
     *
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
       let x = 0;

       /*
        * Distribuimos los x cuadrantes generados.
        *
        * TODO: Medir tiempo de respuesta, calcular media móvil para cada srv.
        */
       for target in targets {
         let workload = ((x as f64)*(target.weight)) as u64;
         let packet = Packet::from_iface(&self.host.nic, workload);
         target.virtual_link.send_through(packet);
       }

       /*
        * Esperamos la respuesta de cada servidor.
        */
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

// TODO: Cargar configuración desde archivo.
const SERVER_COUNT: usize = 5;
const CLIENT_COUNT: usize = 5;

const GLOBAL_SPEED: f64 = 1.0;

fn main() {
  let mut threads = Vec::new();
  let mut servers: Vec<Server> = Vec::new();
  let mut clients: Vec<Client> = Vec::new();

  /* Inicializar servidores.
   *
   * TODO: Cargar capacidad de procesamiento de cada servidor desde archivo.
   * Solo habría que completar el siguiente vector, con un entero por server.
   */
  let power: Vec<u64> = Vec::new();

  for i in 0..SERVER_COUNT {
    servers.push(Server::new(power[i]));
  }

  /*
   * Inicializar clientes.
   *
   * TODO: Cargar la distribución de trabajo para cada servidor desde archivo.
   * Por cada iteracion se puede cargar un nuevo conjunto de valores.
   */
  let mut workshare: Vec<f64> = Vec::new();

  for _i in 0..CLIENT_COUNT {
    clients.push(Client::new(&servers, &workshare));
    workshare.clear();
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
