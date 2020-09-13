use std::{fs, ops::Deref};

use {
    futures::{future, prelude::*},
    serde::Serialize,
};

use gluon_codegen::{Getable, Pushable, Trace, Userdata, VmType};

use gluon::{
    vm::{
        self,
        api::{OwnedFunction, RuntimeResult, IO},
        primitive, record, ExternModule,
    },
    Thread, ThreadExt,
};

use structopt::StructOpt;

type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

pub fn load_master(thread: &Thread) -> vm::Result<ExternModule> {
    #[derive(Debug, VmType, Userdata, Trace)]
    #[gluon(vm_type = "MasterTryThread")]
    #[gluon_trace(skip)]
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

pub fn load(thread: &Thread) -> vm::Result<ExternModule> {
    #[derive(Debug, VmType, Userdata, Trace)]
    #[gluon(vm_type = "TryThread")]
    #[gluon_trace(skip)]
    pub struct TryThread(gluon_crates_io::RootedThread);

    impl Deref for TryThread {
        type Target = gluon_crates_io::Thread;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    thread.register_type::<TryThread>("TryThread", &[])?;

    ExternModule::new(
        thread,
        record! {
            make_eval_vm => primitive!(1, "make_eval_vm", |()| {
                RuntimeResult::from(gluon_crates_io::make_eval_vm().map(TryThread))
            }),
            eval => primitive!(2, "eval", |t: &TryThread, s: &str| gluon_crates_io::eval(t, s)),
            format_expr => primitive!(2, |t: &TryThread, s: &str| gluon_crates_io::format_expr(t, s))
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

#[derive(Debug, VmType, Userdata, Trace)]
#[gluon(vm_type = "Github")]
#[gluon_trace(skip)]
struct Github(hubcaps::Github);

fn new_github(gist_access_token: &str) -> Github {
    Github(
        hubcaps::Github::new(
            "try_gluon".to_string(),
            hubcaps::Credentials::Token(gist_access_token.into()),
        )
        .unwrap(),
    )
}

fn share(github: &Github, gist: Gist<'_>) -> impl Future<Output = Result<PostGist, String>> {
    log::info!("Share: `{}`", gist.code);

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
        .map_ok(|response| PostGist {
            id: response.id,
            html_url: response.html_url,
        })
}

#[cfg(unix)]
async fn exit_server() -> Result<()> {
    use tokio::signal::unix::{signal, SignalKind};
    let mut stream = futures::stream::select(
        signal(SignalKind::interrupt())?,
        signal(SignalKind::terminate())?,
    );

    match stream.next().await {
        Some(()) => eprintln!("Signal received. Shutting down"),
        None => eprintln!("Signal handler shutdown. Shutting down"),
    }
    Ok(())
}

#[cfg(not(unix))]
async fn exit_server() -> Result<()> {
    Ok(tokio::signal::ctrl_c().await?)
}

#[derive(StructOpt, Pushable, VmType)]
struct Opts {
    #[structopt(
        long = "gist-access-token",
        env = "GIST_ACCESS_TOKEN",
        help = "The access tokens used to create gists"
    )]
    gist_access_token: Option<String>,
    #[structopt(short = "p", long = "port", help = "The port to start the server on")]
    port: Option<u16>,
    #[structopt(long = "https", help = "Whether to run the server with https")]
    https: bool,
    #[structopt(
        long = "host",
        default_value = "gluon-lang.org",
        help = "The hostname for the server"
    )]
    host: String,
    #[structopt(
        long = "staging",
        help = "Whether to use letsencrypt's staging environment"
    )]
    staging: bool,

    #[structopt(long = "num-threads", help = "How many threads to run the server with")]
    num_threads: Option<usize>,
}

fn main() {
    let opts = Opts::from_args();

    let result = (|| {
        let mut runtime = {
            let mut builder = tokio::runtime::Builder::new();
            if let Some(num_threads) = opts.num_threads {
                builder.core_threads(num_threads);
            }
            builder.enable_all().threaded_scheduler().build()?
        };
        runtime.block_on(main_(opts))
    })();
    if let Err(err) = result {
        eprintln!("{}", err);
    }
}

async fn main_(opts: Opts) -> Result<()> {
    env_logger::init();

    let vm = gluon::new_vm_async().await;
    gluon::import::add_extern_module(&vm, "gluon.try", load);
    gluon::import::add_extern_module(&vm, "gluon.try.master", load_master);
    gluon::import::add_extern_module(&vm, "gluon.http_server", |vm| {
        ExternModule::new(
            vm,
            record! {
                type Opts => Opts,
                log => record! {
                    error => primitive!(1, "log.error", |s: &str| {
                        log::error!("{}", s);
                        IO::Value(())
                    }),
                    warn => primitive!(1, "log.warn", |s: &str| {
                        log::warn!("{}", s);
                        IO::Value(())
                    }),
                    info => primitive!(1, "log.info", |s: &str| {
                        log::info!("{}", s);
                        IO::Value(())
                    }),
                    debug => primitive!(1, "log.debug", |s: &str| {
                        log::debug!("{}", s);
                        IO::Value(())
                    })
                }
            },
        )
    });
    gluon::import::add_extern_module(&vm, "github", |vm| {
        vm.register_type::<Github>("Github", &[])?;
        ExternModule::new(
            vm,
            record! {
                new_github => primitive!(1, new_github),
                share => primitive!(2, async fn share)
            },
        )
    });

    let server_source = fs::read_to_string("src/app/server.glu")?;

    future::try_select(
        Box::pin(async move {
            let (mut f, _) = vm
                .run_expr_async::<OwnedFunction<fn(Opts) -> IO<()>>>(
                    "src.app.server",
                    &server_source,
                )
                .await?;
            f.call_async(opts).await?;
            Ok(())
        }),
        Box::pin(exit_server()),
    )
    .map_err(|either| match either {
        futures::future::Either::Left((err, _)) | futures::future::Either::Right((err, _)) => err,
    })
    .await?;

    Ok(())
}
