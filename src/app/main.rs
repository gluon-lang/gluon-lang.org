extern crate env_logger;
extern crate failure;
extern crate futures;
extern crate hubcaps;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate log;
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

use std::fs;
use std::ops::Deref;

use futures::{future, Future};

use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

use gluon::{
    vm::{
        self,
        api::{OwnedFunction, RuntimeResult, IO},
        ExternModule,
    },
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
            make_eval_vm => primitive!(1, "make_eval_vm", |()| {
                RuntimeResult::from(gluon_master::make_eval_vm().map(TryThread))
            }),
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
            )).into_iter()
            .collect(),
        }).map_err(|err| err.to_string())
        .map(|response| PostGist {
            id: response.id,
            html_url: response.html_url,
        }).then(Ok)
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
        help = "The port to start the server on"
    )]
    port: Option<u16>,
    #[structopt(
        long = "https",
        help = "Whether to run the server with https"
    )]
    https: bool,
    #[structopt(
        long = "host",
        default_value = "\"gluon-lang.org\"",
        help = "The hostname for the server"
    )]
    host: String,
    #[structopt(
        long = "staging",
        help = "Whether to use letsencrypt's staging environment"
    )]
    staging: bool,
}

fn main() {
    if let Err(err) = main_() {
        eprintln!("{}\n{}", err, err.backtrace());
    }
}

fn main_() -> Result<(), failure::Error> {
    env_logger::init();

    let opts = Opts::from_args();

    let mut runtime = tokio::runtime::Runtime::new()?;

    let vm = gluon::new_vm();
    gluon::add_extern_module(&vm, "gluon.try", gluon::load);
    gluon::add_extern_module(&vm, "gluon.try.master", load_master);
    gluon::add_extern_module(&vm, "gluon.http_server", |vm| {
        ExternModule::new(
            vm,
            record!{
                type Opts => Opts
            },
        )
    });
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
            ).and_then(|(mut f, _)| f.call_async(opts).from_err())
    }))?;

    Ok(())
}
