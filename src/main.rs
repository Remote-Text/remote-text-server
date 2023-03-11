use warp::Filter;
use serde::Serialize;


#[derive(Serialize, Copy, Clone)]
struct Hello {
    hello: &'static str
}

#[tokio::main]
async fn main() {
    let h = Hello { hello: "world" };
    let hello = warp::any()
        .map(move || warp::reply::json(&h))
        .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*"));

    warp::serve(hello)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
