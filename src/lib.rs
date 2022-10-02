use std::sync::{Arc, Mutex};
use serde_derive::{Deserialize, Serialize};
use warp::{Filter};

#[derive(Deserialize, Serialize, Debug)]
pub struct KlunkyRequest {
    action: String,
    params: Vec<String>,
    key: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KlunkyResponse {
    response: Vec<String>,
    error: Vec<String>,
}

pub type Prox<T> = Arc<Mutex<T>>;

pub fn with_prox<T: Send>(prox:Prox<T>) -> impl Filter<Extract = (Prox<T>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || prox.clone())
}

pub fn klunky_spawn<T: Send + 'static>(proxy: Prox<T>, f: fn(KlunkyRequest, Prox<T>) -> String) {
     let make_request = warp::post()
            .and(warp::body::json())
            .and(with_prox(Arc::clone(&proxy)))
            .map(f);

    tokio::spawn(async move {
        warp::serve(make_request).run(([127, 0, 0, 1], 55865)).await
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};
    
    pub struct App {
        magic_number: i32,
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_basic_app() {
        let app = App {magic_number: 10 };
        let proxy_original = Arc::new(Mutex::new(app));

        klunky_spawn(Arc::clone(&proxy_original), |req, proxy| {
            let mut pp = proxy.lock().unwrap();
            pp.magic_number += 1;
            serde_json::to_string(&KlunkyResponse { response: vec![format!("action = {}, magic_number = {}", req.action, pp.magic_number)], error: vec![]}).unwrap()
        });

        let ten_millis = time::Duration::from_secs(10);
        thread::sleep(ten_millis);

        println!("mn = {}", proxy_original.clone().lock().unwrap().magic_number);
    }
}