use hyper;

use std::{
    fs,
    path::PathBuf,
    process::{Child, Command},
};

use futures::{future, Future};
use tokio::runtime::current_thread::Runtime;
use tokio_retry::{strategy::FixedInterval, Retry};

struct DropKill(Child);

impl Drop for DropKill {
    fn drop(&mut self) {
        self.0.kill().unwrap();
    }
}

#[test]
fn test_pages() {
    let pages = &[
        ("", hyper::StatusCode::OK),
        ("/not_existing.html", hyper::StatusCode::NOT_FOUND),
        ("/404.html", hyper::StatusCode::OK),
    ];

    let path = PathBuf::from(::std::env::args().next().unwrap());
    let exe = path
        .parent()
        .and_then(|path| path.parent())
        .expect("server executable")
        .join(env!("CARGO_PKG_NAME"));
    let _child = DropKill(Command::new(exe).args(&["--port", "4567"]).spawn().unwrap());
    let mut runtime = Runtime::new().unwrap();

    for (page, expected_status) in pages {
        runtime
            .block_on(future::lazy(move || {
                let client = hyper::Client::new();
                let strategy = FixedInterval::from_millis(500).take(20);
                let url = format!("http://localhost:4567{}", page);
                Retry::spawn(strategy, {
                    let url = url.clone();
                    move || client.get(url.parse().unwrap())
                })
                .map(move |response| {
                    assert_eq!(
                        response.status(),
                        *expected_status,
                        "Unexpected status for {}",
                        url
                    )
                })
            }))
            .unwrap();
    }
}

#[test]
fn test_examples_master() {
    use gluon_master::{
        make_eval_vm,
        vm::api::{Hole, OpaqueValue},
        RootedThread, ThreadExt,
    };

    let thread = make_eval_vm().unwrap();

    for example_path in fs::read_dir("public/examples").unwrap() {
        let example_path = &example_path.as_ref().unwrap().path();
        eprintln!("{}", example_path.display());
        let contents = fs::read_to_string(example_path).unwrap();
        thread
            .run_expr::<OpaqueValue<RootedThread, Hole>>(
                &example_path.display().to_string(),
                &contents,
            )
            .unwrap_or_else(|err| panic!("{}", err));
    }
}

#[test]
fn test_examples_crates_io() {
    use gluon_crates_io::{
        make_eval_vm,
        vm::api::{Hole, OpaqueValue},
        RootedThread, ThreadExt,
    };
    let thread = make_eval_vm().unwrap();

    for example_path in fs::read_dir("public/examples").unwrap() {
        let example_path = &example_path.as_ref().unwrap().path();
        eprintln!("{}", example_path.display());
        let contents = fs::read_to_string(example_path).unwrap();
        thread
            .run_expr::<OpaqueValue<RootedThread, Hole>>(
                &example_path.display().to_string(),
                &contents,
            )
            .unwrap_or_else(|err| panic!("{}", err));
    }
}
