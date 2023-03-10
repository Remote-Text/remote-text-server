use warp::Filter;



#[tokio::main]
async fn main() {
    let hello = warp::any().map(|| "Hello, World!");

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
