use hyper;

use std::{
    fs,
    path::PathBuf,
    process::{Child, Command},
    time::Duration,
};

use futures::prelude::*;

struct DropKill(Child);

impl Drop for DropKill {
    fn drop(&mut self) {
        self.0.kill().unwrap();
    }
}

async fn retry<F, T, E>(
    strategy: impl IntoIterator<Item = Duration>,
    mut f: impl FnMut() -> F,
) -> F::Output
where
    F: Future<Output = Result<T, E>>,
{
    let mut last_error = None;
    for t in strategy {
        match f().await {
            Ok(response) => return Ok(response),
            Err(err) => {
                tokio::time::delay_for(t).await;
                last_error = Some(err);
            }
        }
    }
    Err(last_error.unwrap())
}

#[tokio::test]
async fn test_pages() {
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

    let client = hyper::Client::new();
    for (page, expected_status) in pages {
        let strategy = std::iter::repeat(Duration::from_millis(500)).take(20);
        let url = format!("http://localhost:4567{}", page);

        let response = retry(strategy, || client.get(url.parse().unwrap()))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            *expected_status,
            "Unexpected status for {}",
            url
        );
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
