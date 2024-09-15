/// This file contains exported macros that are useful for the godot SDK usage.

/// Call an async function on an instance of a node.
///
/// This is useful for calling a function with a node from another thread.
#[macro_export]
macro_rules! call_async {
    ($id:expr, $func:literal) => {
        Gd::<Node>::from_instance_id($id)
            .call_deferred(StringName::from($func), &[])
    };

    ($id:expr, $func:literal, $($args:expr),*) => {
     Gd::<Node>::from_instance_id($id)
        .call_deferred(StringName::from($func), &[
         $($args),+
     ])
    };
 }

/// Emits a new event asynchronously.
///
/// This macro can be called in a secondary thread, as long as you have the id of the instance.
#[macro_export]
macro_rules! emit_event {
    ($id:expr, $event_name:literal) => {
        call_async!($id, "emit_signal", Variant::from($event_name))
    };

    ($id:expr, $event_name:literal $(, $args:expr)*) => {
     call_async!($id, "emit_signal", Variant::from($event_name), $($args),+)
    };
 }
