
use hyper;



use std::path::PathBuf;
use std::process::{Command, Stdio};

use futures::{future, Future};
use tokio::runtime::current_thread::Runtime;
use tokio_retry::{strategy::FixedInterval, Retry};

#[test]
fn test() {
    let path = PathBuf::from(::std::env::args().next().unwrap());
    let exe = path
        .parent()
        .and_then(|path| path.parent())
        .expect("server executable")
        .join(env!("CARGO_PKG_NAME"));
    let mut child = Command::new(exe).stdout(Stdio::piped()).spawn().unwrap();
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(future::lazy(move || {
                let client = hyper::Client::new();
                let strategy = FixedInterval::from_millis(500).take(20);
                Retry::spawn(strategy, move || {
                    client.get("http://localhost".parse().unwrap())
                }).map(|response| assert_eq!(response.status(), hyper::StatusCode::OK))
            })).unwrap();
    }
    child.kill().unwrap();
}
