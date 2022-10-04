use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::Write;

struct KlunkyRequest;
struct KlunkyConnection;

/* Need to abstract and wrap stuff in a KlunkyRequest(inside tcp stream) and a KlunkyConnection(tcpstream itself) */
/* I may need to impl iterator for this to preserve scope and stuff */
pub struct KlunkyServer {
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

        // Might want to use a tokio task here as they're much lighter weight
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

    /* 
        does this even work? i feel like we would want more dynamic behavior here. This is just as limiting
        by putting it into a closure here
    */
    pub fn handle_connections(&mut self, f: fn(&TcpStream)) {
        // Definately need to wrap everything in a KlunkyConnection that has a request() and send(...) method
        let connclone = self.conns.clone();
        let mut clone = connclone.lock().unwrap();
        
        // Proces the connections
        for conn in clone.iter() {
            f(conn);
            conn.shutdown(Shutdown::Both).unwrap();
        }
        
        (*clone).clear();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[test]
    fn test_1() {
        let mut kc = KlunkyServer::new(6666);
        kc.spawn();

        loop {
            // Do some work
            thread::sleep(time::Duration::from_millis(500));

            // Might have to do some funky stuff to make this easy
            kc.handle_connections(|mut stream| {
                let buf = "HTTP/1.1 200 OK\r\n".as_bytes();
                stream.write(&buf).unwrap();
            })
        }
    }
}
