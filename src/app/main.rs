extern crate env_logger;
extern crate futures;
extern crate gluon;
extern crate gluon_format;
extern crate gluon_master;
#[macro_use]
extern crate iron;
#[macro_use]
extern crate log;
extern crate mount;
extern crate persistent;
extern crate regex;
extern crate serde_json;
extern crate staticfile;

use std::fs::{read_dir, File};
use std::io::{self, Read};
use std::time::Instant;

use futures::Async;

use regex::Regex;

use iron::mime::Mime;
use iron::modifiers::RedirectRaw;
use iron::prelude::*;
use iron::{status, Handler};
use iron::typemap::Key;

use serde_json::Value;

use staticfile::Static;

use mount::Mount;

use gluon::base::symbol::{Symbol, SymbolRef};
use gluon::base::kind::{ArcKind, KindEnv};
use gluon::base::types::{Alias, ArcType, RecordSelector, TypeEnv};
use gluon::vm::thread::{RootedThread, Thread, ThreadInternal};
use gluon::vm::{self, Error};
use gluon::vm::internal::ValuePrinter;
use gluon::vm::api::{Hole, OpaqueValue};
use gluon::Compiler;
use gluon::import::{add_extern_module, DefaultImporter, Import};

use gluon_format::format_expr;

pub struct EmptyEnv;

impl KindEnv for EmptyEnv {
    fn find_kind(&self, _type_name: &SymbolRef) -> Option<ArcKind> {
        None
    }
}

impl TypeEnv for EmptyEnv {
    fn find_type(&self, _id: &SymbolRef) -> Option<&ArcType> {
        None
    }
    fn find_type_info(&self, _id: &SymbolRef) -> Option<&Alias<Symbol, ArcType>> {
        None
    }
    fn find_record(&self, _fields: &[Symbol], _: RecordSelector) -> Option<(ArcType, ArcType)> {
        None
    }
}

pub fn eval_stable(global_vm: &Thread, body: &str) -> IronResult<String> {
    let vm = match global_vm.new_thread() {
        Ok(vm) => vm,
        Err(err) => return Ok(format!("{}", err)),
    };

    // Prevent a single thread from allocating to much memory
    vm.set_memory_limit(2_000_000);

    {
        let mut context = vm.context();

        // Prevent the stack from consuming to much memory
        context.set_max_stack_size(10000);

        // Prevent infinite loops from running forever
        let start = Instant::now();
        context.set_hook(Some(Box::new(move |_, _| {
            if start.elapsed().as_secs() < 10 {
                Ok(Async::Ready(()))
            } else {
                Err(Error::Message(
                    "Thread has exceeded the allowed exection time".into(),
                ))
            }
        })));
    }

    let (value, typ) =
        match Compiler::new().run_expr::<OpaqueValue<&Thread, Hole>>(&vm, "<top>", &body) {
            Ok(value) => value,
            Err(err) => return Ok(format!("{}", err)),
        };

    unsafe {
        Ok(format!(
            "{} : {}",
            ValuePrinter::new(&EmptyEnv, &typ, value.get_value()).max_level(6),
            typ
        ))
    }
}

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

struct Config;

impl Key for Config {
    type Value = String;
}

fn config(req: &mut Request) -> IronResult<Response> {
    let s = req.get::<persistent::Read<Config>>().unwrap();
    Ok(Response::with((status::Ok, (*s).clone())))
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
        })
        .collect::<io::Result<_>>()
        .unwrap();

    let lock_file = include_str!("../../Cargo.lock");

    let crates_io_version = Regex::new("checksum gluon ([^ ]+).+registry")
        .unwrap()
        .captures(lock_file)
        .expect("crates.io version")
        .get(1)
        .unwrap()
        .as_str();
    let git_master_version = &Regex::new("git\\+[^#]+gluon#([^\"]+)")
        .unwrap()
        .captures(lock_file)
        .expect("gluon master version")
        .get(1)
        .unwrap()
        .as_str()[0..6];

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

fn make_eval_vm() -> RootedThread {
    let vm = RootedThread::new();

    // Ensure the import macro cannot be abused to to open files
    {
        // Ensure the lock to `paths` are released
        let import = Import::new(DefaultImporter);
        import.paths.write().unwrap().clear();
        vm.get_macros().insert(String::from("import"), import);
    }

    // Initialize the basic types such as `Bool` and `Option` so they are available when loading
    // other modules
    Compiler::new()
        .implicit_prelude(false)
        .run_expr::<OpaqueValue<&Thread, Hole>>(&vm, "", r#" import! "std/types.glu" "#)
        .unwrap();

    add_extern_module(&vm, "std.prim", vm::primitives::load);
    add_extern_module(&vm, "std.int.prim", vm::primitives::load_int);
    add_extern_module(&vm, "std.float.prim", vm::primitives::load_float);
    add_extern_module(&vm, "std.string.prim", vm::primitives::load_string);
    add_extern_module(&vm, "std.char.prim", vm::primitives::load_char);
    add_extern_module(&vm, "std.array.prim", vm::primitives::load_array);

    add_extern_module(&vm, "std.lazy", vm::lazy::load);
    add_extern_module(&vm, "std.reference", vm::reference::load);

    // Load the io library so the prelude can be loaded
    // (`IO` actions won't actually execute however)
    add_extern_module(&vm, "std.io.prim", gluon::io::load);

    vm
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

fn main() {
    env_logger::init().unwrap();

    let mut try_mount = Mount::new();
    try_mount.mount("/", Static::new("dist/try"));
    {
        let mut middleware = Chain::new(config);
        let config_string = serde_json::to_string(&load_config()).unwrap();

        middleware.link(persistent::Read::<Config>::both(config_string));
        try_mount.mount("/config", middleware);
    }

    {
        let vm = make_eval_vm();
        mount_eval(&mut try_mount, "", move |body| eval_stable(&vm, body));

        try_mount.mount("/format", move |req: &mut Request| format(req, format_expr));
    }

    {
        let vm = gluon_master::make_eval_vm();
        mount_eval(&mut try_mount, "/master", move |body| {
            Ok(match gluon_master::eval(&vm, body) {
                Ok(x) => x,
                Err(err) => err.to_string(),
            })
        });

        try_mount.mount("/master/format", |req: &mut Request| {
            format(req, gluon_master::format::format_expr)
        });
    }

    let mut mount = Mount::new();
    mount.mount("/try/", try_mount);
    mount.mount("/", |req: &mut Request| -> IronResult<Response> {
        Ok(Response::with((
            status::TemporaryRedirect,
            RedirectRaw(format!(
                "/try/{}",
                req.url
                    .query()
                    .map(|q| format!("?{}", q))
                    .unwrap_or(String::new())
            )),
        )))
    });

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
    }).http(address)
        .unwrap();

    println!("Server started at `{}`", address);
}
