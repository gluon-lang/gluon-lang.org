extern crate failure;
extern crate futures;

extern crate gluon;
extern crate gluon_format;

use std::time::Instant;

use self::futures::Async;

use self::gluon::base::kind::{ArcKind, KindEnv};
use self::gluon::base::symbol::{Symbol, SymbolRef};
use self::gluon::base::types::{Alias, ArcType, TypeEnv};
use self::gluon::import::{add_extern_module, DefaultImporter, Import};
use self::gluon::vm;
use self::gluon::vm::api::{Hole, OpaqueValue};
use self::gluon::vm::internal::ValuePrinter;
use self::gluon::vm::thread::ThreadInternal;

pub use self::gluon::*;

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
}

pub fn make_eval_vm() -> RootedThread {
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
    add_extern_module(&vm, "std.byte.prim", vm::primitives::load_byte);
    add_extern_module(&vm, "std.int.prim", vm::primitives::load_int);
    add_extern_module(&vm, "std.float.prim", vm::primitives::load_float);
    add_extern_module(&vm, "std.string.prim", vm::primitives::load_string);
    add_extern_module(&vm, "std.char.prim", vm::primitives::load_char);
    add_extern_module(&vm, "std.array.prim", vm::primitives::load_array);

    add_extern_module(&vm, "std.lazy", vm::lazy::load);
    add_extern_module(&vm, "std.reference", vm::reference::load);

    add_extern_module(&vm, "std.json.prim", ::vm::api::json::load);

    // add_extern_module(&vm, "std.channel", ::vm::channel::load_channel);
    // add_extern_module(&vm, "std.thread.prim", ::vm::channel::load_thread);
    // add_extern_module(&vm, "std.debug", ::vm::debug::load);
    add_extern_module(&vm, "std.io.prim", io::load);

    vm
}

pub fn eval(global_vm: &Thread, body: &str) -> Result<String> {
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
                Err(vm::Error::Message(
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

    Ok(format!(
        "{} : {}",
        ValuePrinter::new(&EmptyEnv, &typ, value.get_variant()).max_level(6),
        typ
    ))
}

pub fn format_expr(thread: &Thread, input: &str) -> Result<String> {
    gluon_format::format_expr(&mut Compiler::new(), thread, "try", input)
}
