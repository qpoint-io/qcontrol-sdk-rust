//! Exec logger plugin - logs all exec operations to a file
//!
//! Demonstrates the v1 exec API. Note: exec hooks are not yet
//! implemented in the agent, so this plugin will compile but
//! the callbacks won't be invoked at runtime.
//!
//! Environment variables:
//!   QCONTROL_LOG_FILE - Path to log file (default: /tmp/qcontrol.log)

use qcontrol::prelude::*;

static LOGGER: Logger = Logger::new();

fn init() -> Result<(), Error> {
    LOGGER.init();
    LOGGER.log("[exec_logger.rs] initializing...");
    Ok(())
}

fn cleanup() {
    LOGGER.log("[exec_logger.rs] cleanup complete");
}

fn on_exec(ev: &ExecEvent) -> ExecResult {
    // Log the exec event
    LOGGER.log(&format!("[exec_logger.rs] exec(\"{}\")", ev.path()));

    // Log arguments
    for (idx, arg) in ev.argv().enumerate() {
        LOGGER.log(&format!("[exec_logger.rs]   argv[{}] = \"{}\"", idx, arg));
    }

    // Log cwd if set
    if let Some(cwd) = ev.cwd() {
        LOGGER.log(&format!("[exec_logger.rs]   cwd = \"{}\"", cwd));
    }

    ExecResult::Pass
}

fn on_exec_stdin(_state: PluginState, ev: &StdinEvent) -> ExecAction {
    LOGGER.log(&format!(
        "[exec_logger.rs] stdin(pid={}, count={})",
        ev.pid(),
        ev.count()
    ));
    ExecAction::Pass
}

fn on_exec_stdout(_state: PluginState, ev: &StdoutEvent) -> ExecAction {
    LOGGER.log(&format!(
        "[exec_logger.rs] stdout(pid={}, count={}) = {}",
        ev.pid(),
        ev.count(),
        ev.result()
    ));
    ExecAction::Pass
}

fn on_exec_stderr(_state: PluginState, ev: &StderrEvent) -> ExecAction {
    LOGGER.log(&format!(
        "[exec_logger.rs] stderr(pid={}, count={}) = {}",
        ev.pid(),
        ev.count(),
        ev.result()
    ));
    ExecAction::Pass
}

fn on_exec_exit(_state: PluginState, ev: &ExitEvent) {
    if ev.exited_normally() {
        LOGGER.log(&format!(
            "[exec_logger.rs] exit(pid={}, code={})",
            ev.pid(),
            ev.exit_code()
        ));
    } else {
        LOGGER.log(&format!(
            "[exec_logger.rs] exit(pid={}, signal={})",
            ev.pid(),
            ev.exit_signal()
        ));
    }
}

export_plugin!(PluginBuilder::new("rust_exec_logger")
    .on_init(init)
    .on_cleanup(cleanup)
    .on_exec(on_exec)
    .on_exec_stdin(on_exec_stdin)
    .on_exec_stdout(on_exec_stdout)
    .on_exec_stderr(on_exec_stderr)
    .on_exec_exit(on_exec_exit));
