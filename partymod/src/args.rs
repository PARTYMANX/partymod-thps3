use partymod_common::{args::ArgCollection, syncunsafecell::SyncUnsafeCell};

pub static ARGS_CONTEXT: SyncUnsafeCell<Option<ArgCollection>> = SyncUnsafeCell::new(None);

pub fn init() {
    unsafe {
        let ctx = &mut *ARGS_CONTEXT.get();
        *ctx = Some(ArgCollection::new());
    }
}

pub fn has_arg(arg: &str) -> bool {
    let args_context = match unsafe { &mut *ARGS_CONTEXT.get() } {
        Some(v) => v,
        None => panic!("Tried to use uninitialized args context!"),
    };

    args_context.has_arg(arg)
}
