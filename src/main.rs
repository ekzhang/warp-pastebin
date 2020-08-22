use std::convert::Infallible;

use routes::routes;
use server::make_server;

mod routes;
mod server;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let port = std::env::var("PORT")
        .unwrap_or("3535".to_string())
        .parse()
        .expect("PORT must be an integer");

    let svc = warp::service(routes());
    let make_svc = hyper::service::make_service_fn(|_: _| {
        let svc = svc.clone();
        async move { Ok::<_, Infallible>(svc) }
    });

    println!("Starting server at http://localhost:{}", port);
    make_server(port)?.serve(make_svc).await
}
