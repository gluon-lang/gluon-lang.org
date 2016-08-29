#[macro_use]
extern crate iron;
extern crate persistent;
extern crate staticfile;
extern crate mount;
extern crate gluon;
extern crate serde_json;

use std::io::Read;

use iron::prelude::*;
use iron::status;
use iron::typemap::Key;

use staticfile::Static;

use mount::Mount;

use gluon::vm::thread::RootedThread;
use gluon::vm::api::generic::A;
use gluon::vm::api::Generic;
use gluon::{Compiler, new_vm};

pub struct VMKey;
impl Key for VMKey { type Value = RootedThread; }

fn eval(req: &mut Request) -> IronResult<Response> {
    eval_(req)
        .map(|s| Response::with((status::Ok, ::serde_json::to_string(&s).unwrap())))
}
fn eval_(req: &mut Request) -> IronResult<String> {
    let mut body = String::new();
    itry!(req.body.read_to_string(&mut body));
    let global_vm = req.get::<persistent::Read<VMKey>>().unwrap();
    let vm = global_vm.new_thread().expect("New thread");
    let (value, typ) = match Compiler::new().run_expr::<Generic<A>>(&vm, "<top>", &body) {
        Ok(value) => value,
        Err(err) => return Ok(format!("{}", err)),
    };
    Ok(format!("{:?} : {}", value, typ))
}

fn main() {
    let mut mount = Mount::new();

    mount.mount("/", Static::new("public"));

    let mut middleware = Chain::new(eval);
    let vm = new_vm();
    middleware.link(persistent::Read::<VMKey>::both(vm));
    mount.mount("/eval", middleware);

    Iron::new(mount).http("0.0.0.0:8080").unwrap();
}
