extern crate env_logger;
extern crate failure;
extern crate futures;
extern crate glob;
extern crate home;
extern crate hubcaps;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate log;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[allow(unused_imports)]
#[macro_use]
extern crate structopt;
extern crate tokio;

extern crate gluon_master;

#[macro_use]
extern crate gluon_vm;
#[macro_use]
extern crate gluon_codegen;

use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::process::Command;

use futures::{future, Future};

use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

use regex::Regex;

use serde_json::Value;

use gluon::{
    vm::{self, ExternModule},
    Thread,
};

use structopt::StructOpt;

mod gluon;

pub fn load_master(thread: &Thread) -> vm::Result<ExternModule> {
    #[derive(Debug, Userdata)]
    pub struct TryThread(gluon_master::RootedThread);

    impl Deref for TryThread {
        type Target = gluon_master::Thread;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    thread.register_type::<TryThread>("MasterTryThread", &[])?;

    ExternModule::new(
        thread,
        record! {
            make_eval_vm => primitive!(1, "make_eval_vm", |x| TryThread(gluon_master::make_eval_vm(x))),
            eval => primitive!(2, "eval", |t: &TryThread, s: &str| gluon_master::eval(t, s)),
            format_expr => primitive!(2, |t: &TryThread, s: &str| gluon_master::format_expr(t, s))
        },
    )
}

#[derive(Debug, Default, Getable, VmType)]
pub struct Gist<'a> {
    pub code: &'a str,
}

#[derive(Debug, Default, Serialize, Pushable, VmType)]
pub struct PostGist {
    pub id: String,
    pub html_url: String,
}

#[derive(Debug, Userdata)]
struct Github(hubcaps::Github<HttpsConnector<HttpConnector>>);

fn new_github(gist_access_token: &str) -> Github {
    Github(hubcaps::Github::new(
        "try_gluon".to_string(),
        hubcaps::Credentials::Token(gist_access_token.into()),
    ))
}

fn share(
    github: &Github,
    gist: Gist,
) -> impl Future<Item = Result<PostGist, String>, Error = vm::Error> {
    info!("Share: `{}`", gist.code);

    github
        .0
        .gists()
        .create(&hubcaps::gists::GistOptions {
            description: Some("Gluon code shared from try_gluon".into()),
            public: Some(true),
            files: Some((
                "try_gluon.glu".into(),
                hubcaps::gists::Content {
                    filename: None,
                    content: gist.code.into(),
                },
            ))
            .into_iter()
            .collect(),
        })
        .map_err(|err| err.to_string())
        .map(|response| PostGist {
            id: response.id,
            html_url: response.html_url,
        })
        .then(Ok)
}

const LOCK_FILE: &str = include_str!("../../Cargo.lock");

fn git_master_version() -> String {
    Regex::new("git\\+[^#]+gluon#([^\"]+)")
        .unwrap()
        .captures(LOCK_FILE)
        .expect("gluon master version")
        .get(1)
        .unwrap()
        .as_str()
        .to_string()
}

fn load_config() -> Value {
    let vec = fs::read_dir("public/examples")
        .unwrap()
        .map(|entry| {
            let path = try!(entry).path();
            let name = String::from(path.file_stem().unwrap().to_str().unwrap());
            let mut file = try!(File::open(path));
            let mut contents = String::new();

            try!(file.read_to_string(&mut contents));

            let value = vec![
                ("name".into(), Value::String(name)),
                ("value".into(), Value::String(contents)),
            ];

            Ok(Value::Object(value.into_iter().collect()))
        })
        .collect::<io::Result<_>>()
        .unwrap();

    let crates_io_version = Regex::new("checksum gluon ([^ ]+).+(registry|git)")
        .unwrap()
        .captures(LOCK_FILE)
        .expect("crates.io version")
        .get(1)
        .unwrap()
        .as_str();
    let git_master_version = git_master_version()[0..6].to_string();

    Value::Object(
        vec![
            (
                "last_release".to_string(),
                Value::String(crates_io_version.to_string()),
            ),
            (
                "git_master".to_string(),
                Value::String(git_master_version.to_string()),
            ),
            ("examples".to_string(), Value::Array(vec)),
        ]
        .into_iter()
        .collect(),
    )
}

fn gluon_git_path() -> Result<PathBuf, failure::Error> {
    let std_glob_path = home::cargo_home()?
        .join(&format!(
            "git/checkouts/gluon-*/{}",
            &git_master_version()[..7]
        ))
        .display()
        .to_string();
    Ok(glob::glob(&std_glob_path)?
        .next()
        .expect("git repo in cargo home")?)
}

fn create_docs(path: &str) -> Result<(), failure::Error> {
    let git_dir = gluon_git_path()?;

    let exit_status = Command::new("cp")
        .args(&["-r", &git_dir.join("std").to_string_lossy(), "."])
        .status()?;
    if !exit_status.success() {
        return Err(failure::err_msg("Error copying docs"));
    }

    gluon_master::generate_doc("std", path)?;

    let mut command = Command::new("mdbook");
    command.args(&[
        "build",
        "--dest-dir",
        &env::current_dir()?.join("dist/book").to_string_lossy(),
        &git_dir.join("book").to_string_lossy(),
    ]);
    println!("Building book: {:?}", command);
    let exit_status = command.status()?;
    if !exit_status.success() {
        return Err(failure::err_msg("Error building book docs"));
    }

    Ok(())
}

#[derive(StructOpt, Pushable, VmType)]
struct Opts {
    #[structopt(
        long = "gist-access-token",
        env = "GIST_ACCESS_TOKEN",
        help = "The access tokens used to create gists"
    )]
    gist_access_token: Option<String>,
    #[structopt(
        short = "p",
        long = "port",
        default_value = "80",
        help = "The port to start the server on"
    )]
    port: u16,
}

fn main() {
    if let Err(err) = main_() {
        eprintln!("{}\n{}", err, err.backtrace());
    }
}

fn main_() -> Result<(), failure::Error> {
    env_logger::init();

    let opts = Opts::from_args();

    use gluon::vm::api::{OwnedFunction, IO};

    {
        let config_string = serde_json::to_string(&load_config())?;
        let mut config_file = File::create("dist/try/config")?;
        config_file.write_all(config_string.as_bytes())?;
    }

    let doc_path = "dist/doc/nightly";
    create_docs(doc_path)?;

    let mut runtime = tokio::runtime::Runtime::new()?;

    let vm = gluon::new_vm();
    gluon::add_extern_module(&vm, "gluon.try", gluon::load);
    gluon::add_extern_module(&vm, "gluon.try.master", load_master);
    gluon::add_extern_module(&vm, "github", |vm| {
        vm.register_type::<Github>("Github", &[])?;
        ExternModule::new(
            vm,
            record!{
                new_github => primitive!(1, new_github),
                share => primitive!(2, async fn share)
            },
        )
    });

    let server_source = fs::read_to_string("src/app/server.glu")?;

    runtime.block_on(future::lazy(move || {
        gluon::Compiler::new()
            .run_expr_async::<OwnedFunction<fn(Opts) -> IO<()>>>(
                &vm,
                "src.app.server",
                &server_source,
            )
            .and_then(|(mut f, _)| f.call_async(opts).from_err())
    }))?;

    Ok(())
}
