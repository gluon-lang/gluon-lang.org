pub use gluon_doc;

use std::{result::Result as StdResult, task::Poll, time::Instant};

pub use gluon::{
    base::{
        kind::{ArcKind, KindEnv},
        symbol::{Symbol, SymbolRef},
        types::{Alias, ArcType, TypeEnv},
    },
    import::{add_extern_module, DefaultImporter, Import},
    vm::{
        self,
        api::{Hole, OpaqueValue},
        internal::ValuePrinter,
        thread::ThreadInternal,
    },
    Result,
};

pub use gluon::*;

pub struct EmptyEnv;

impl KindEnv for EmptyEnv {
    fn find_kind(&self, _type_name: &SymbolRef) -> Option<ArcKind> {
        None
    }
}

impl TypeEnv for EmptyEnv {
    type Type = ArcType;

    fn find_type(&self, _id: &SymbolRef) -> Option<ArcType> {
        None
    }
    fn find_type_info(&self, _id: &SymbolRef) -> Option<Alias<Symbol, ArcType>> {
        None
    }
}

pub fn make_eval_vm() -> Result<RootedThread> {
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
    add_extern_module(&vm, "std.prim", crate::vm::primitives::load);

    vm.run_expr::<OpaqueValue<&Thread, Hole>>(
        "",
        r#"//@NO-IMPLICIT-PRELUDE
           let _ = import! std.types
           let _ = import! std.prim
           ()
        "#,
    )
    .unwrap_or_else(|err| panic!("{}", err));

    add_extern_module(&vm, "std.byte.prim", crate::vm::primitives::load_byte);
    add_extern_module(&vm, "std.int.prim", crate::vm::primitives::load_int);
    add_extern_module(&vm, "std.float.prim", crate::vm::primitives::load_float);
    add_extern_module(&vm, "std.string.prim", crate::vm::primitives::load_string);
    add_extern_module(&vm, "std.fs.prim", crate::vm::primitives::load_fs);
    add_extern_module(&vm, "std.path.prim", crate::vm::primitives::load_path);
    add_extern_module(&vm, "std.char.prim", crate::vm::primitives::load_char);
    add_extern_module(&vm, "std.array.prim", crate::vm::primitives::load_array);

    add_extern_module(&vm, "std.lazy.prim", crate::vm::lazy::load);
    add_extern_module(&vm, "std.reference.prim", crate::vm::reference::load);

    // add_extern_module(&vm, "std.channel.prim", crate::vm::channel::load_channel);
    // add_extern_module(&vm, "std.thread.prim", crate::vm::channel::load_thread);
    // add_extern_module(&vm, "std.debug.prim", crate::vm::debug::load);
    add_extern_module(&vm, "std.io.prim", crate::std_lib::io::load);
    add_extern_module(&vm, "std.process.prim", crate::std_lib::process::load);

    add_extern_module(&vm, "std.json.prim", crate::vm::api::json::load);

    Ok(vm)
}

pub fn eval(global_vm: &Thread, body: &str) -> StdResult<String, String> {
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
            Poll::Ready(if start.elapsed().as_secs() < 10 {
                Ok(())
            } else {
                Err(vm::Error::Message(
                    "Thread has exceeded the allowed exection time".into(),
                ))
            })
        })));
    }

    let (value, typ) = match vm.run_expr::<OpaqueValue<&Thread, Hole>>("<top>", &body) {
        Ok(value) => value,
        Err(err) => return Ok(format!("{}", err)),
    };

    Ok(format!(
        "{} : {}",
        ValuePrinter::new(&EmptyEnv, &typ, value.get_variant(), &Default::default()).max_level(6),
        typ
    ))
}

pub fn format_expr(thread: &Thread, input: &str) -> StdResult<String, String> {
    thread
        .format_expr(&mut gluon_format::Formatter::default(), "try", input)
        .map_err(|err| err.to_string())
}

pub fn generate_doc(options: &gluon_doc::Options) -> StdResult<(), anyhow::Error> {
    gluon_doc::generate(options, &gluon::new_vm())
}
