use maplit::hashmap;
use nanoid::nanoid;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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

        let raw_view_filter = warp::path!("raw" / String)
            .and(warp::get())
            .and(store_filter.clone())
            .and_then(raw_view);

        warp::path("api").and(paste_filter.or(paste_view_filter).or(raw_view_filter))
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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
struct Paste {
    text: String,
    lang: String,
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
    let id = nanoid!(8);
    store.pastes.write().await.insert(id.to_string(), body);
    Ok(warp::reply::with_status(
        warp::reply::json(&hashmap! { "id" => id }),
        StatusCode::CREATED,
    ))
}

async fn paste_view(id: String, store: Store) -> Result<impl Reply, Rejection> {
    let pastes = store.pastes.read().await;
    match pastes.get(&id) {
        Some(p) => Ok(warp::reply::json(p)),
        None => Err(warp::reject()),
    }
}

async fn raw_view(id: String, store: Store) -> Result<impl Reply, Rejection> {
    let pastes = store.pastes.read().await;
    match pastes.get(&id) {
        Some(p) => Ok(warp::reply::with_status(
            String::from(&p.text),
            StatusCode::OK,
        )),
        None => Err(warp::reject()),
    }
}

#[cfg(test)]
mod tests {
    use maplit::hashmap;
    use std::collections::HashMap;
    use warp::http::StatusCode;
    use warp::test::request;

    use super::{routes, Paste};

    #[tokio::test]
    async fn test_post() {
        let api = routes();

        let resp = request()
            .method("POST")
            .path("/api/paste")
            .json(&Paste {
                text: "Testing paste api".into(),
                lang: "plaintext".into(),
            })
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_1mb_too_large() {
        let api = routes();

        let long_text = "a".repeat(1024 * 1024);
        let resp = request()
            .method("POST")
            .path("/api/paste")
            .json(&Paste {
                text: long_text,
                lang: "plaintext".into(),
            })
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_not_exists() {
        let api = routes();

        let resp = request()
            .method("GET")
            .path("/api/paste/bad_id_123")
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_e2e() {
        let api = routes();

        let p = Paste {
            text: "My favorite pastebin".into(),
            lang: "plaintext".into(),
        };
        let resp_post = request()
            .method("POST")
            .path("/api/paste")
            .json(&p)
            .reply(&api)
            .await;

        assert_eq!(resp_post.status(), StatusCode::CREATED);
        let json: HashMap<&str, &str> = serde_json::from_slice(resp_post.body())
            .expect("Could not deserialize POST response to JSON object");
        let id = json
            .get("id")
            .expect("POST response does not have an `id` property");
        assert_eq!(json, hashmap! {"id" => *id});

        let resp_get = request()
            .method("GET")
            .path(&format!("/api/paste/{}", id))
            .reply(&api)
            .await;

        assert_eq!(resp_get.status(), StatusCode::OK);
        assert_eq!(serde_json::from_slice::<Paste>(resp_get.body()).unwrap(), p);

        let resp_get_raw = request()
            .method("GET")
            .path(&format!("/api/raw/{}", id))
            .reply(&api)
            .await;

        assert_eq!(resp_get_raw.status(), StatusCode::OK);
        assert_eq!(resp_get_raw.body(), &p.text);
    }
}
