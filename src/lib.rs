use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::{Arc, Mutex};
use std::io::{Write, Read, BufReader, BufRead};
use std::{thread, time, error};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KlunkyResponse {
    pub result: Vec<String>,
    pub error: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
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
    NotPost,
}

impl KlunkyConnection {
    pub fn request(&mut self) -> Result<KlunkyRequest, Box<dyn error::Error>> {
        let mut reader = BufReader::new(self.connection.try_clone()?);
        let mut name = String::new();
        loop {
        let r = reader.read_line(&mut name)?;
            if r < 3 { //detect empty line
                break;
            }
        }
        let mut size = 0;
        let linesplit = name.split("\n");
        for l in linesplit {
            if l.starts_with("Content-Length") {
                    let sizeplit = l.split(":");
                    for s in sizeplit {
                        if !(s.starts_with("Content-Length")) {
                            size = s.trim().parse::<usize>()?; //Get Content-Length
                    }
                }
            }
        }
        let mut buffer = vec![0; size]; //New Vector with size of Content   
        reader.read_exact(&mut buffer)?; //Get the Body Content.        
        let data = std::str::from_utf8(&buffer)?;

        let deserialized: KlunkyRequest = serde_json::from_str(&data)?;
        Ok(deserialized)
    }

    pub fn respond(&mut self, response: KlunkyResponse) -> Result<usize, std::io::Error> {
        let header = "HTTP/1.1 200 OK";
        let message = format!("{}", serde_json::to_string(&response).unwrap());

        let res = self.connection.write(
            format!("{header}\r\nContent-Length:{}\r\n\r\n{message}", message.len()).as_bytes());

        self.connection.flush()?;

        res
    }
}

impl Drop for KlunkyConnection {
    fn drop(&mut self) {
        self.connection.shutdown(Shutdown::Both).ok();
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
                let make_request = c.request();
                if let Ok(req) = make_request {
                    println!("action: {}, params:{:?}", req.action, req.params);
                    c.respond(KlunkyResponse{result:vec![], error: vec![]}).unwrap();

                } else {
                    c.respond(KlunkyResponse{result:vec![], error: vec![format!("{:?}", make_request)]}).unwrap();
                }            
            }
        }
    }
}