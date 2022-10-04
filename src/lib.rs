use std::io::Write;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};

struct KlunkyRequest;
struct KlunkyConnection;

/* Need to abstract away this shit and wrap stuff in a KlunkyRequest(inside tcp stream) and a KlunkyConnection(tcpstream itself) */
struct KlunkyServer {
    // need this to work between threads
    conns: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>,
    listener: TcpListener
}

impl KlunkyServer {
    // Spawn => Add incoming connections to queue, lock
    // connections: returns an iterator

    fn new(port: u32) -> Self {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
        listener.set_nonblocking(true).expect("Cannot set non-blocking");

        Self { conns: Default::default(), listener }
    }

    fn spawn(&mut self) {
        let copy = self.conns.clone();
        thread::spawn(move || {
            
        });
    }
    //fn connections(&mut self) <== this also clears out the connections
}
use std::{thread, time};

fn main() {
    let mut kc = KlunkyServer::new(6666);
    kc.spawn();
    
    // loop {
    //     // Work we're doing in the middle here
    //     thread::sleep(time::Duration::from_millis(1000));

    //     // ---
    //     // ---
    //     // ---
    //     // ---
        
    //     // Accept connections
    //     match kc.listener.accept() {
    //         Ok( (_socket, _addr) ) => kc.conns.push( (_socket, _addr) ),
    //         Err(_) => {}
    //     }

    //     // Process connections
    //     for conn in &mut kc.conns {
    //         let buf = "abcde".as_bytes();
    //         conn.0.write(&buf);
    //     }

    //     kc.conns.clear();
    // }
}
