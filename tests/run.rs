use hyper;

use std::path::PathBuf;
use std::process::Command;

use futures::{future, Future};
use tokio::runtime::current_thread::Runtime;
use tokio_retry::{strategy::FixedInterval, Retry};

#[test]
fn test() {
    test_pages(&[
        ("", hyper::StatusCode::OK),
        ("/not_existing.html", hyper::StatusCode::NOT_FOUND),
        ("/404.html", hyper::StatusCode::OK),
    ]);
}

fn test_pages(pages: &[(&str, hyper::StatusCode)]) {
    let path = PathBuf::from(::std::env::args().next().unwrap());
    let exe = path
        .parent()
        .and_then(|path| path.parent())
        .expect("server executable")
        .join(env!("CARGO_PKG_NAME"));
    let mut child = Command::new(exe).args(&["--port", "4567"]).spawn().unwrap();
    {
        let mut runtime = Runtime::new().unwrap();

        for (page, expected_status) in pages {
            runtime
                .block_on(future::lazy(move || {
                    let client = hyper::Client::new();
                    let strategy = FixedInterval::from_millis(500).take(20);
                    Retry::spawn(strategy, move || {
                        client.get(format!("http://localhost:4567{}", page).parse().unwrap())
                    })
                    .map(move |response| assert_eq!(response.status(), *expected_status))
                }))
                .unwrap();
        }
    }
    child.kill().unwrap();
}
