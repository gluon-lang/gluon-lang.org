use std::{convert::Infallible, fs, ops::Deref};

use {
    anyhow::anyhow,
    bytes::Bytes,
    futures::{future, prelude::*},
    http_body_util::BodyExt,
    lambda_runtime::Diagnostic,
    serde::Serialize,
    structopt::StructOpt,
};

use gluon_codegen::{Getable, Pushable, Trace, Userdata, VmType};

use gluon::{
    vm::{
        self,
        api::{Function, OwnedFunction, RuntimeResult, IO},
        primitive, record, ExternModule,
    },
    RootedThread, Thread, ThreadExt,
};

type Error = anyhow::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

pub fn load_master(thread: &Thread) -> vm::Result<ExternModule> {
    #[derive(Debug, VmType, Userdata, Trace, Clone)]
    #[gluon(vm_type = "MasterTryThread")]
    #[gluon_userdata(clone)]
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
    #[derive(Debug, VmType, Userdata, Trace, Clone)]
    #[gluon(vm_type = "TryThread")]
    #[gluon_userdata(clone)]
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

#[derive(Debug, VmType, Userdata, Trace, Clone)]
#[gluon(vm_type = "Github")]
#[gluon_userdata(clone)]
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
    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;
    tokio::select! {
        _ = interrupt.recv() => eprintln!("Signal received. Shutting down"),
        _ = terminate.recv() => eprintln!("Signal received. Shutting down"),
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
    #[structopt(
        short = "p",
        long = "port",
        env = "PORT",
        help = "The port to start the server on"
    )]
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

    #[structopt(
        long = "lambda",
        help = "Whether to run the server as a lambda function"
    )]
    lambda: bool,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let opts = Opts::from_args();

    let result = async {
        if opts.lambda {
            let handler = mk_handler(opts).await?;
            lambda_http::run(lambda_http::service_fn(handler))
                .await
                .map_err(|err| anyhow!(err))
        } else {
            main_(opts).await
        }
    }
    .await;
    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

async fn mk_handler(
    opts: Opts,
) -> Result<
    impl Fn(
        lambda_http::Request,
    ) -> future::BoxFuture<
        'static,
        Result<
            lambda_http::Response<http_body_util::combinators::BoxBody<Bytes, Infallible>>,
            Diagnostic,
        >,
    >,
> {
    let vm = new_vm().await;
    let server_source = fs::read_to_string("src/app/server.glu")?;

    vm.load_script_async("src.app.server", &server_source)
        .await?;
    let mut load_handler: Function<RootedThread, fn(Opts) -> IO<_>> =
        vm.get_global("src.app.server.load_handler")?;
    let h = load_handler
        .call_async(opts)
        .await?
        .into_result()
        .map_err(|err| anyhow!(err))?;

    let handler = gluon::std_lib::http::Handler::new(&vm, h);

    Ok(move |req| {
        let handler = handler.clone();
        handler_fn(handler, req)
            .inspect_err(|err| log::error!("{}", err))
            .map_err(|err| Diagnostic {
                error_type: "HandlerError".into(),
                error_message: err.to_string(),
            })
            .boxed()
    })
}

async fn handler_fn(
    mut handler: gluon::std_lib::http::Handler,
    req: lambda_http::Request,
) -> Result<lambda_http::Response<http_body_util::combinators::BoxBody<Bytes, Infallible>>> {
    let (parts, body) = req.into_parts();
    let response = handler
        .handle(parts.method, parts.uri, body.into_data_stream())
        .await?;

    let (parts, body) = response.into_parts();

    let response = {
        let mut builder = lambda_http::Response::builder().status(parts.status);
        *builder.headers_mut().unwrap() = parts.headers;
        builder.body(body).unwrap()
    };
    Ok(response)
}

async fn new_vm() -> RootedThread {
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

    vm
}

async fn main_(opts: Opts) -> Result<()> {
    let vm = new_vm().await;

    let server_source = fs::read_to_string("src/app/server.glu")?;

    future::try_select(
        Box::pin(async move {
            vm.load_script_async("src.app.server", &server_source)
                .await?;
            let mut f: OwnedFunction<fn(Opts) -> IO<()>> = vm.get_global("src.app.server.start")?;
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
