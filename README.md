# Warp Pastebin

[Live website](https://warp-pastebin.herokuapp.com/)

Warp is a web framework written in Rust. I've enjoyed playing with its `Filter` system so far.

```rust
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
```

It's pretty neat how simple & composable these combinators are!
