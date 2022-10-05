use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::{Arc, Mutex};
use std::io::{Write, Read};
use std::{thread, time};

#[derive(Debug)]
pub struct KlunkyResponse {
    pub result: Vec<String>,
    pub error: Vec<String>
}

#[derive(Debug)]
pub struct KlunkyRequest {
    pub action: String,
    pub params: Vec<String>
}

pub struct KlunkyConnection {
    pub connection: TcpStream,
}

#[derive(Debug)]
pub enum KlunkyError {
    MalformedInput,
}

impl KlunkyConnection {
    pub fn request(&mut self) -> Result<KlunkyRequest, KlunkyError> {
        //let mut content_length = -1;
        //println!("buf = {buf:?}");

        Ok(KlunkyRequest { action: "Abcde".to_string(), params: vec![] })
    }

    pub fn respond(&mut self, response: KlunkyResponse) -> Result<usize, std::io::Error> {
        let header = "HTTP/1.1 200 OK";
        let message = format!("{:?}", response);
        self.connection.write(format!("{header}\r\nContent-Length:{}\r\n\r\n{message}", message.len()).as_bytes())
    }
}

impl Drop for KlunkyConnection {
    fn drop(&mut self) {
        self.connection.shutdown(Shutdown::Both).unwrap();
    }
}

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

    pub fn spawn(&mut self, delay: u64) {
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
                thread::sleep(time::Duration::from_millis(delay));
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

    #[test]
    fn test_1() {
        let mut kc = KlunkyServer::new(6666);
        kc.spawn(100);

        loop {
            // Do some program work
            thread::sleep(time::Duration::from_millis(500));
            let connections = kc.consume_connections().into_iter();

            for mut c in connections {
                println!("Request body = {:?}", c.request().unwrap());
                c.respond(KlunkyResponse{result:vec!["Ok".to_string()], error: vec![]}).ok();
            }
        }
    }
}