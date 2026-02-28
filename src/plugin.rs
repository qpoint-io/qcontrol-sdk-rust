//! Plugin macro for declaring qcontrol plugins

/// Declare a qcontrol plugin with fallible initialization.
///
/// This macro generates the required C exports automatically. The initialization
/// closure should return `Result<(), Error>` to support proper error handling.
///
/// # Example
///
/// ```rust,ignore
/// use qcontrol::{plugin, register_file_open, FileOpenContext, FilterResult, Error};
///
/// fn log_open(ctx: &FileOpenContext) -> FilterResult {
///     eprintln!("open({}) = {}", ctx.path(), ctx.result());
///     FilterResult::Continue
/// }
///
/// plugin!(|| -> Result<(), Error> {
///     register_file_open("my_logger", None, Some(log_open))?;
///     Ok(())
/// });
/// ```
#[macro_export]
macro_rules! plugin {
    ($init:expr) => {
        #[no_mangle]
        pub extern "C" fn qcontrol_plugin_init() -> i32 {
            let init_fn: fn() -> Result<(), $crate::Error> = $init;
            match init_fn() {
                Ok(()) => 0,
                Err(_) => -1,
            }
        }

        #[no_mangle]
        pub extern "C" fn qcontrol_plugin_cleanup() {}
    };
}
