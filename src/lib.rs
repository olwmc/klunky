use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::Write;

struct KlunkyResponse {
    result: Vec<String>,
    error: Vec<String>

}
struct KlunkyRequest {
    action: String,
    params: Vec<String>
}
pub struct KlunkyConnection {
    connection: TcpStream
}

impl Drop for KlunkyConnection {
    fn drop(&mut self) {
        self.connection.shutdown(Shutdown::Both).unwrap();
    }
}

/* Need to abstract and wrap stuff in a KlunkyRequest(inside tcp stream) and a KlunkyConnection(tcpstream itself) */
/* I may need to impl iterator for this to preserve scope and stuff */
pub struct KlunkyServer {    
    conns: Arc<Mutex<Vec<TcpStream>>>,
    listener: TcpListener,
}

impl KlunkyServer {
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

    pub fn consume_connections(&mut self) -> Vec<KlunkyConnection>{
        let mut v = vec![];
        let connclone = self.conns.clone();
        let mut clone = connclone.lock().unwrap();
        let content = &*clone;

        for c in content {
            v.push( KlunkyConnection{ connection: c.try_clone().unwrap()} )
        }

        (*clone).clear();
        
        return v
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
            thread::sleep(time::Duration::from_millis(500));

            for mut c in kc.consume_connections().into_iter() {
                let buf = "HTTP/1.1 200 OK\r\n".as_bytes();
                c.connection.write(&buf).unwrap();
            }

            println!("#connections = {}", kc.consume_connections().len());

        }
    }
}