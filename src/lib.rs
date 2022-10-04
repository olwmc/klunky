use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;

struct KlunkyRequest;
struct KlunkyConnection;

/* Need to abstract and wrap stuff in a KlunkyRequest(inside tcp stream) and a KlunkyConnection(tcpstream itself) */
struct KlunkyServer {
    // need this to work between threads
    conns: Arc<Mutex<Vec<TcpStream>>>,
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
                // can this deadlock? I don't think so because they're in different scopes and
                // one would just wait for the other as either is guaranteed to get unlocked
                if let Ok((socket, _)) = listener.accept() {
                    copy.clone().lock().unwrap().push( socket )
                }
            }
        });
    }
    // pub fn connections(&mut self) -> impl Iterator<Item=&TcpStream> {
    //     self.conns.clone().clone().lock().unwrap().iter().map(|(s, _)|)
    // }
    //fn clear_connections(&mut self)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time, net::Shutdown};

    #[test]
    fn test_1() {
        let mut kc = KlunkyServer::new(6666);
        kc.spawn();

        for _ in 0..5 {
            thread::sleep(time::Duration::from_millis(3000));

            let connclone = kc.conns.clone();
            let clone = connclone.lock().unwrap();
            for mut conn in clone.iter() {
                let buf = "abcde".as_bytes();
                conn.write(&buf);
                conn.shutdown(Shutdown::Both);
            }

            let connclone = kc.conns.clone();
            *connclone.lock().unwrap() = vec![];
        }
    }
}
