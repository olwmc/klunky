use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;

struct KlunkyRequest;
struct KlunkyConnection;

/* Need to abstract away this shit and wrap stuff in a KlunkyRequest(inside tcp stream) and a KlunkyConnection(tcpstream itself) */
struct KlunkyServer {
    // need this to work between threads
    conns: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>,
    listener: TcpListener,
}

impl KlunkyServer {
    // Spawn => Add incoming connections to queue, lock
    // connections: returns an iterator

    pub fn new(port: u32) -> Self {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
        listener.set_nonblocking(true).expect("Cannot set non-blocking");

        Self { conns: Default::default(), listener }
    }

    pub fn spawn(&mut self) {
        let copy = self.conns.clone();
        let listener = self.listener.try_clone().unwrap();

        thread::spawn(move || {
            // Accept connections
            loop {
                match listener.accept() {
                    Ok( (_socket, _addr) ) => copy.clone().lock().unwrap().push( (_socket, _addr) ),
                    Err(_) => {}
                }
            }
        });
    }
    // pub fn connections(&mut self) -> impl Iterator<Item=&TcpStream> {
    // }
    //fn clear_connections(&mut self)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[test]
    fn test_1() {
        let mut kc = KlunkyServer::new(6666);
        kc.spawn();

        thread::sleep(time::Duration::from_millis(3000));

        for conn in kc.connections() {
            let buf = "abcde".as_bytes();
            conn.write(&buf);
        }
    }
}
