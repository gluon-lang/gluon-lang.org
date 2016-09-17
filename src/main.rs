#[macro_use]
extern crate iron;
extern crate persistent;
extern crate staticfile;
extern crate mount;
extern crate gluon;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::io::Read;
use std::time::Instant;

use iron::prelude::*;
use iron::status;
use iron::typemap::Key;

use staticfile::Static;

use mount::Mount;

use gluon::vm::thread::{RootedThread, ThreadInternal};
use gluon::vm::Error;
use gluon::vm::api::Generic;
use gluon::vm::api::generic::A;
use gluon::{Compiler, new_vm};

pub struct VMKey;
impl Key for VMKey {
    type Value = RootedThread;
}

fn eval(req: &mut Request) -> IronResult<Response> {
    eval_(req).map(|s| Response::with((status::Ok, ::serde_json::to_string(&s).unwrap())))
}
fn eval_(req: &mut Request) -> IronResult<String> {
    let mut body = String::new();
    itry!(req.body.read_to_string(&mut body));
    info!("Eval: `{}`", body);
    body.push(' ');
    let global_vm = req.get::<persistent::Read<VMKey>>().unwrap();
    let vm = match global_vm.new_thread() {
        Ok(vm) => vm,
        Err(err) => return Ok(format!("{}", err)),
    };

    // Prevent a single thread from allocating to much memory
    vm.set_memory_limit(2_000_000);

    // Prevent infinite loops from running forever
    let start = Instant::now();
    vm.context().set_hook(Some(Box::new(move |_| {
        if start.elapsed().as_secs() < 10 {
            Ok(())
        } else {
            Err(Error::Message("Thread has exceeded the allowed exection time".into()))
        }
    })));

    let (value, typ) = match Compiler::new().run_expr::<Generic<A>>(&vm, "<top>", &body) {
        Ok(value) => value,
        Err(err) => return Ok(format!("{}", err)),
    };
    Ok(format!("{:?} : {}", value, typ))
}

fn main() {
    env_logger::init().unwrap();
    let mut mount = Mount::new();

    mount.mount("/", Static::new("public"));

    let mut middleware = Chain::new(eval);
    let vm = new_vm();
    middleware.link(persistent::Read::<VMKey>::both(vm));
    mount.mount("/eval", middleware);

    let address = "0.0.0.0:8080";
    let _server = Iron::new(mount).http(address).unwrap();
    println!("Server started at `{}`", address);
}
