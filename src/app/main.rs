extern crate env_logger;
extern crate failure;
extern crate futures;
extern crate glob;
extern crate home;
extern crate hubcaps;
extern crate hyper;
#[macro_use]
extern crate iron;
#[macro_use]
extern crate log;
extern crate mount;
extern crate persistent;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate staticfile;
#[allow(unused_imports)]
#[macro_use]
extern crate structopt;
extern crate tokio_core;

extern crate gluon_master;

use std::env;
use std::fs::{read_dir, File};
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;

use regex::Regex;

use iron::mime::Mime;
use iron::modifiers::RedirectRaw;
use iron::prelude::*;
use iron::typemap::Key;
use iron::{status, Handler};

use serde_json::Value;

use staticfile::Static;

use structopt::StructOpt;

use mount::Mount;

mod gluon;

fn format<F, E>(req: &mut Request, format_expr: F) -> IronResult<Response>
where
    F: Fn(&str) -> Result<String, E>,
    E: ::std::fmt::Display,
{
    let mut body = String::new();

    itry!(req.body.read_to_string(&mut body));
    info!("Format: `{}`", body);
    let mime: Mime = "text/plain".parse().unwrap();
    match format_expr(&body) {
        Ok(formatted) => Ok(Response::with((
            status::Ok,
            mime,
            serde_json::to_string(&formatted).unwrap(),
        ))),
        Err(err) => Ok(Response::with((
            status::NotAcceptable,
            mime,
            serde_json::to_string(&err.to_string()).unwrap(),
        ))),
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Gist {
    pub code: String,
}

#[derive(Debug, Default, Serialize)]
pub struct PostGist<'a> {
    pub id: &'a str,
    pub html_url: &'a str,
}

fn share<C>(
    core: &mut tokio_core::reactor::Core,
    gists: &hubcaps::gists::Gists<C>,
    req: &mut Request,
) -> IronResult<Response>
where
    C: Clone + hyper::client::Connect,
{
    let mut body = String::new();

    itry!(req.body.read_to_string(&mut body));
    info!("Share: `{}`", body);
    let mime: Mime = "text/plain".parse().unwrap();
    match serde_json::from_str::<Gist>(&body) {
        Ok(gist) => {
            let result = core.run(
                gists.create(&hubcaps::gists::GistOptions {
                    description: Some("Gluon code shared from try_gluon".into()),
                    public: Some(true),
                    files: Some((
                        "try_gluon.glu".into(),
                        hubcaps::gists::Content {
                            filename: None,
                            content: gist.code,
                        },
                    )).into_iter()
                    .collect(),
                }),
            );
            let response = match result {
                Ok(r) => r,
                Err(err) => {
                    error!("{}", err);
                    return Ok(Response::with((status::InternalServerError, mime, "")));
                }
            };

            Ok(Response::with((
                status::Ok,
                mime,
                serde_json::to_string(&PostGist {
                    id: &response.id,
                    html_url: &response.html_url,
                }).unwrap(),
            )))
        }
        Err(err) => Ok(Response::with((
            status::NotAcceptable,
            mime,
            serde_json::to_string(&err.to_string()).unwrap(),
        ))),
    }
}

struct Config;

impl Key for Config {
    type Value = String;
}

fn config(req: &mut Request) -> IronResult<Response> {
    let s = req.get::<persistent::Read<Config>>().unwrap();
    Ok(Response::with((status::Ok, (*s).clone())))
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
    let vec = read_dir("public/examples")
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
        }).collect::<io::Result<_>>()
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
        ].into_iter()
        .collect(),
    )
}
fn mount_eval<M>(mount: &mut Mount, prefix: &str, eval: M)
where
    M: Fn(&str) -> IronResult<String> + Send + Sync + 'static,
{
    let middleware = Chain::new(move |req: &mut Request| {
        let mut body = String::new();

        itry!(req.body.read_to_string(&mut body));
        info!("Eval: `{}`", body);

        eval(&body).map(|s| {
            let mime: Mime = "text/plain".parse().unwrap();

            Response::with((status::Ok, mime, serde_json::to_string(&s).unwrap()))
        })
    });

    mount.mount(&format!("{}/eval", prefix), middleware);
}

fn gluon_git_path() -> Result<PathBuf, failure::Error> {
    let std_glob_path = home::cargo_home()?
        .join(&format!(
            "git/checkouts/gluon-*/{}",
            &git_master_version()[..7]
        )).display()
        .to_string();
    Ok(glob::glob(&std_glob_path)?
        .next()
        .expect("git repo in cargo home")?)
}

fn create_docs(path: &str) -> Result<(), failure::Error> {
    let git_dir = gluon_git_path()?;

    let exit_status = Command::new("cp")
        .args(&["-r", &git_dir.join("std").to_string_lossy(), "std"])
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

#[derive(StructOpt)]
struct Opts {
    #[structopt(
        long = "gist-access-token",
        env = "GIST_ACCESS_TOKEN",
        help = "The access tokens used to create gists"
    )]
    gist_access_token: Option<String>,
}

fn main() {
    if let Err(err) = main_() {
        eprintln!("{}\n{}", err, err.backtrace());
    }
}

fn main_() -> Result<(), failure::Error> {
    env_logger::init();

    let opts = Opts::from_args();

    let try_mount = {
        let mut try_mount = Mount::new();

        try_mount.mount("/", Static::new("dist/try"));

        {
            let mut middleware = Chain::new(config);
            let config_string = serde_json::to_string(&load_config()).unwrap();

            middleware.link(persistent::Read::<Config>::both(config_string));
            try_mount.mount("/config", middleware);
        }

        {
            let vm = gluon::make_eval_vm();
            {
                let vm = vm.clone();
                mount_eval(&mut try_mount, "", move |body| {
                    Ok(match gluon::eval(&vm, body) {
                        Ok(x) => x,
                        Err(err) => err.to_string(),
                    })
                });
            }

            try_mount.mount("/format", move |req: &mut Request| {
                format(req, |input| gluon::format_expr(&vm, input))
            });
        }

        {
            let vm = gluon_master::make_eval_vm();
            {
                let vm = vm.clone();
                mount_eval(&mut try_mount, "/master", move |body| {
                    Ok(match gluon_master::eval(&vm, body) {
                        Ok(x) => x,
                        Err(err) => err.to_string(),
                    })
                });
            }

            try_mount.mount("/master/format", move |req: &mut Request| {
                format(req, |input| gluon_master::format_expr(&vm, input))
            });
        }

        if let Some(gist_access_token) = opts.gist_access_token {
            try_mount.mount("/share", move |req: &mut Request| {
                let mut core = tokio_core::reactor::Core::new().unwrap();
                let github = hubcaps::Github::new(
                    "try_gluon".to_string(),
                    hubcaps::Credentials::Token(gist_access_token.clone()),
                    &core.handle(),
                );

                share(&mut core, &github.gists(), req)
            });
        } else {
            warn!("Gist sharing is not enabled!");
            try_mount.mount("/share", |_: &mut Request| {
                Ok(Response::with((
                    status::InternalServerError,
                    "Gist sharing is not enabled!",
                )))
            });
        }

        try_mount
    };

    let mut mount = Mount::new();
    mount.mount("/try/", try_mount);
    mount.mount("/", Static::new("dist"));

    let doc_path = "dist/doc/nightly";
    create_docs(doc_path)?;
    mount.mount("doc/nightly", Static::new(doc_path));
    mount.mount("book", Static::new("dist/book"));

    let address = "0.0.0.0:8080";
    // Dropping `server` causes it to block so keep it alive until the end of scope
    let _server = Iron::new(move |req: &mut Request| {
        // Redirect `try` to `try/` to make relative paths work
        // Need to hack it in here since `Mount` strips trailing `/` ...
        if req.url.path() == ["try"] {
            Ok(Response::with((
                status::PermanentRedirect,
                RedirectRaw(format!(
                    "/try/{}",
                    req.url
                        .query()
                        .map(|q| format!("?{}", q))
                        .unwrap_or(String::new())
                )),
            )))
        } else {
            mount.handle(req)
        }
    }).http(address)?;

    println!("Server started at `{}`", address);
    Ok(())
}
