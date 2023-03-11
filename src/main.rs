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

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
