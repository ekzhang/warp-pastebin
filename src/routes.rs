use maplit::hashmap;
use nanoid::nanoid;
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{http::StatusCode, Filter, Rejection, Reply};

pub fn routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let files = warp::fs::dir("static");

    let api = {
        let store = Store::new();
        let store_filter = warp::any().map(move || store.clone());

        let paste_filter = warp::path!("paste")
            .and(warp::post())
            .and(json_body(1024 * 256))
            .and(store_filter.clone())
            .and_then(paste);

        let paste_view_filter = warp::path!("paste" / String)
            .and(warp::get())
            .and(store_filter.clone())
            .and_then(paste_view);

        warp::path("api").and(paste_filter.or(paste_view_filter))
    };

    files.or(api)
}

fn json_body<T: DeserializeOwned + Send>(
    max_length: u64,
) -> impl Filter<Extract = (T,), Error = Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(max_length).and(warp::body::json())
}

#[derive(Deserialize)]
struct Paste {
    text: String,
}

type Pastes = HashMap<String, Paste>;

#[derive(Default, Clone)]
struct Store {
    pastes: Arc<RwLock<Pastes>>,
}

impl Store {
    fn new() -> Self {
        Default::default()
    }
}

async fn paste(body: Paste, store: Store) -> Result<impl Reply, Rejection> {
    let id = nanoid!();
    store.pastes.write().await.insert(id.to_string(), body);
    Ok(warp::reply::json(&hashmap! { "id" => id }))
}

async fn paste_view(id: String, store: Store) -> Result<impl Reply, Rejection> {
    let pastes = store.pastes.read().await;
    match pastes.get(&id) {
        Some(p) => Ok(warp::reply::with_status(
            String::from(&p.text),
            StatusCode::OK,
        )),
        None => Err(warp::reject()),
    }
}
