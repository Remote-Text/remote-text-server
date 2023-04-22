#[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::path::{Path, PathBuf};

use warp::Filter;

/* // EXTERNAL CRATE USAGE //

pretty_env_logger:
    When initialized, this crate will use the standard error log to create colorful
    and organized log outputs, making it easier to read

serde:
    A common crate that serializes rust data structures to formats such as JSON,
    and can also deserialize data into rust data structures

uuid:
    This crate provides the generation and parsing of Unique Universal Identifiers

warp:
    Based off of the popular networking crate hyper, warp provides a simple and fast
    web server framework

*/


mod routes;
mod handlers;
mod api;
mod files;

#[allow(non_snake_case)]
fn FILES_DIR() -> PathBuf {
    Path::new(".").join("files")
}
#[allow(non_snake_case)]
fn PREVIEWS_DIR() -> PathBuf {
    Path::new(".").join("previews")
}

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        std::env::set_var("RUST_LOG", "remote_text_server=warn");
    }

    // Initialize pretty_env_logger so we can get organized/colorful logs
    pretty_env_logger::init();

    #[allow(non_snake_case)]
    if std::env::args().collect::<Vec<String>>().get(1) == Some(&"-vv".to_string()) {
        // log::trace!(target: "remote_text_server::main", "ENV_VARS");
        let VERGEN_BUILD_DATE = env!("VERGEN_BUILD_DATE");
        let VERGEN_BUILD_TIMESTAMP = env!("VERGEN_BUILD_TIMESTAMP");
        let VERGEN_CARGO_DEBUG = env!("VERGEN_CARGO_DEBUG");
        let VERGEN_CARGO_FEATURES = env!("VERGEN_CARGO_FEATURES");
        let VERGEN_CARGO_OPT_LEVEL = env!("VERGEN_CARGO_OPT_LEVEL");
        let VERGEN_CARGO_TARGET_TRIPLE = env!("VERGEN_CARGO_TARGET_TRIPLE");
        let VERGEN_GIT_BRANCH = env!("VERGEN_GIT_BRANCH");
        let VERGEN_GIT_COMMIT_AUTHOR_EMAIL = env!("VERGEN_GIT_COMMIT_AUTHOR_EMAIL");
        let VERGEN_GIT_COMMIT_AUTHOR_NAME = env!("VERGEN_GIT_COMMIT_AUTHOR_NAME");
        let VERGEN_GIT_COMMIT_COUNT = env!("VERGEN_GIT_COMMIT_COUNT");
        let VERGEN_GIT_COMMIT_DATE = env!("VERGEN_GIT_COMMIT_DATE");
        let VERGEN_GIT_COMMIT_MESSAGE = env!("VERGEN_GIT_COMMIT_MESSAGE");
        let VERGEN_GIT_COMMIT_TIMESTAMP = env!("VERGEN_GIT_COMMIT_TIMESTAMP");
        let VERGEN_GIT_DESCRIBE = env!("VERGEN_GIT_DESCRIBE");
        let VERGEN_GIT_SHA = env!("VERGEN_GIT_SHA");
        let VERGEN_RUSTC_CHANNEL = env!("VERGEN_RUSTC_CHANNEL");
        let VERGEN_RUSTC_COMMIT_DATE = env!("VERGEN_RUSTC_COMMIT_DATE");
        let VERGEN_RUSTC_COMMIT_HASH = env!("VERGEN_RUSTC_COMMIT_HASH");
        let VERGEN_RUSTC_HOST_TRIPLE = env!("VERGEN_RUSTC_HOST_TRIPLE");
        let VERGEN_RUSTC_LLVM_VERSION = env!("VERGEN_RUSTC_LLVM_VERSION");
        let VERGEN_RUSTC_SEMVER = env!("VERGEN_RUSTC_SEMVER");
        let VERGEN_SYSINFO_CPU_BRAND = env!("VERGEN_SYSINFO_CPU_BRAND");
        let VERGEN_SYSINFO_CPU_CORE_COUNT = env!("VERGEN_SYSINFO_CPU_CORE_COUNT");
        let VERGEN_SYSINFO_CPU_FREQUENCY = env!("VERGEN_SYSINFO_CPU_FREQUENCY");
        let VERGEN_SYSINFO_CPU_NAME = env!("VERGEN_SYSINFO_CPU_NAME");
        let VERGEN_SYSINFO_CPU_VENDOR = env!("VERGEN_SYSINFO_CPU_VENDOR");
        let VERGEN_SYSINFO_NAME = env!("VERGEN_SYSINFO_NAME");
        let VERGEN_SYSINFO_OS_VERSION = env!("VERGEN_SYSINFO_OS_VERSION");
        let VERGEN_SYSINFO_TOTAL_MEMORY = env!("VERGEN_SYSINFO_TOTAL_MEMORY");
        let VERGEN_SYSINFO_USER = env!("VERGEN_SYSINFO_USER");

        println!("\
VERGEN_BUILD_DATE: {VERGEN_BUILD_DATE}
VERGEN_BUILD_TIMESTAMP: {VERGEN_BUILD_TIMESTAMP}
VERGEN_CARGO_DEBUG: {VERGEN_CARGO_DEBUG}
VERGEN_CARGO_FEATURES: {VERGEN_CARGO_FEATURES}
VERGEN_CARGO_OPT_LEVEL: {VERGEN_CARGO_OPT_LEVEL}
VERGEN_CARGO_TARGET_TRIPLE: {VERGEN_CARGO_TARGET_TRIPLE}
VERGEN_GIT_BRANCH: {VERGEN_GIT_BRANCH}
VERGEN_GIT_COMMIT_AUTHOR_EMAIL: {VERGEN_GIT_COMMIT_AUTHOR_EMAIL}
VERGEN_GIT_COMMIT_AUTHOR_NAME: {VERGEN_GIT_COMMIT_AUTHOR_NAME}
VERGEN_GIT_COMMIT_COUNT: {VERGEN_GIT_COMMIT_COUNT}
VERGEN_GIT_COMMIT_DATE: {VERGEN_GIT_COMMIT_DATE}
VERGEN_GIT_COMMIT_MESSAGE: {VERGEN_GIT_COMMIT_MESSAGE}
VERGEN_GIT_COMMIT_TIMESTAMP: {VERGEN_GIT_COMMIT_TIMESTAMP}
VERGEN_GIT_DESCRIBE: {VERGEN_GIT_DESCRIBE}
VERGEN_GIT_SHA: {VERGEN_GIT_SHA}
VERGEN_RUSTC_CHANNEL: {VERGEN_RUSTC_CHANNEL}
VERGEN_RUSTC_COMMIT_DATE: {VERGEN_RUSTC_COMMIT_DATE}
VERGEN_RUSTC_COMMIT_HASH: {VERGEN_RUSTC_COMMIT_HASH}
VERGEN_RUSTC_HOST_TRIPLE: {VERGEN_RUSTC_HOST_TRIPLE}
VERGEN_RUSTC_LLVM_VERSION: {VERGEN_RUSTC_LLVM_VERSION}
VERGEN_RUSTC_SEMVER: {VERGEN_RUSTC_SEMVER}
VERGEN_SYSINFO_CPU_BRAND: {VERGEN_SYSINFO_CPU_BRAND}
VERGEN_SYSINFO_CPU_CORE_COUNT: {VERGEN_SYSINFO_CPU_CORE_COUNT}
VERGEN_SYSINFO_CPU_FREQUENCY: {VERGEN_SYSINFO_CPU_FREQUENCY}
VERGEN_SYSINFO_CPU_NAME: {VERGEN_SYSINFO_CPU_NAME}
VERGEN_SYSINFO_CPU_VENDOR: {VERGEN_SYSINFO_CPU_VENDOR}
VERGEN_SYSINFO_NAME: {VERGEN_SYSINFO_NAME}
VERGEN_SYSINFO_OS_VERSION: {VERGEN_SYSINFO_OS_VERSION}
VERGEN_SYSINFO_TOTAL_MEMORY: {VERGEN_SYSINFO_TOTAL_MEMORY}
VERGEN_SYSINFO_USER: {VERGEN_SYSINFO_USER}
    ");

        std::process::exit(0);
    }

    log::info!(target: "remote_text_server::main", "Searching for repositories");
    let repositories = files::repos();

    log::trace!(target: "remote_text_server::main", "Setting up filters");
    // Set up the warp wrapper with CORS (Cross-Origin Resource Sharing), allowing any origin point
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"]);
    // Sets up logging for api requests
    let log = warp::log("remote_text_server::api");
    // Sets up the root path for the api
    let api_root = warp::path("api");

    log::trace!(target: "remote_text_server::main", "Setting up routes");
    // Creates a chain of filters that checks/runs each function in the API
    let routes = api_root.and(routes::get_routes(repositories.clone()))
        // .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*"))
        .with(cors)
        .with(log);

    log::info!(target: "remote_text_server::main", "Running server");
    // Runs the server with the set up filters
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}