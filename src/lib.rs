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

impl From<KlunkyResponse> for String {
    fn from(resp: KlunkyResponse) -> Self {
        serde_json::to_string(&resp).unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use reqwest;

    pub struct App {
        words: Vec<String>,
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_basic_app() {
        let app = App {words: vec![] };
        let proxy_truth = Arc::new(Mutex::new(app));
        let proxy: Prox<App> = proxy_truth.clone();

        klunky_spawn(proxy.clone(), |req, p| {
            let mut pp = p.lock().unwrap();
            pp.words.push(req.action.clone());

            KlunkyResponse { response: vec![format!("added word: {}", req.action)], error: vec![]}.into()
        });

        for i in 0..5 {
            reqwest::Client::new().post("http://127.0.0.1:55865")
                .body(format!("{{\"action\": \"test{}\", \"params\": []}}", i))
                .send()
                .await.unwrap();
        }

        let proxy = proxy_truth.clone();
        let pp = proxy.lock().unwrap();
        assert_eq!(pp.words, vec!["test0","test1","test2","test3","test4"])
    }
}