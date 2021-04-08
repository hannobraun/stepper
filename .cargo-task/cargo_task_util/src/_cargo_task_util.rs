//! Common cargo_task_util mod will be available to all tasks.
//!
//! Simply include it in your task module:
//! - `mod cargo_task_util;`
//! - `use cargo_task_util::*;`
//!
//! Note, the macros `ct_info!`, `ct_warn!`, `ct_fatal!`, and `ct_check_fatal!`
//! are all defined in this module, and thus are available to task code too.
//! But rust hoists all macros up to the root, so their docs are up there : )

use std::{collections::BTreeMap, ffi::OsString, path::PathBuf, rc::Rc};

/// Force install a new CTEnv
#[doc(hidden)]
pub(crate) fn ct_force_new_env() -> Rc<CTEnv> {
    CT_ENV.with(|env| {
        let mut env = env.lock().unwrap();
        *env = priv_new_env();
        env.clone()
    })
}

/// Fetch the current CTEnv
pub fn ct_env() -> Rc<CTEnv> {
    CT_ENV.with(|env| env.lock().unwrap().clone())
}

/// format! style helper for printing out info messages.
#[macro_export]
macro_rules! ct_info {
    ($($tt:tt)*) => { $crate::_cargo_task_util::ct_info(&format!($($tt)*)); };
}

/// format! style helper for printing out warn messages.
#[macro_export]
macro_rules! ct_warn {
    ($($tt:tt)*) => { $crate::_cargo_task_util::ct_warn(&format!($($tt)*)); };
}

/// format! style helper for printing out fatal messages.
#[macro_export]
macro_rules! ct_fatal {
    ($($tt:tt)*) => { $crate::_cargo_task_util::ct_fatal(&format!($($tt)*)); };
}

/// takes a result, if the result is error, runs ct_fatal!
#[macro_export]
macro_rules! ct_check_fatal {
    ($code:expr) => {
        match { $code } {
            Err(e) => $crate::ct_fatal!("{:#?}", e),
            Ok(r) => r,
        }
    };
}

/// Cargo-task environment info struct.
#[derive(Debug)]
pub struct CTEnv {
    /// The path to the cargo binary.
    pub cargo_path: PathBuf,

    /// The .cargo-task directory.
    pub cargo_task_path: PathBuf,

    /// The targe dir for cargo-task builds.
    pub cargo_task_target: PathBuf,

    /// The root of the cargo task execution environment.
    pub work_dir: PathBuf,

    /// Task list specified by user.
    pub task_list: Vec<String>,

    /// Additional arguments specified by user.
    pub arg_list: Vec<String>,

    /// All tasks defined in the task directory.
    pub tasks: BTreeMap<String, CTTaskMeta>,
}

impl CTEnv {
    /// Create a new cargo std::process::Command
    pub fn cargo(&self) -> std::process::Command {
        std::process::Command::new(&self.cargo_path)
    }

    /// Execute a rust std::process::Command
    pub fn exec(&self, mut cmd: std::process::Command) -> std::io::Result<()> {
        let non_zero_err = format!("{:?} exited non-zero", cmd);
        if !cmd.spawn()?.wait()?.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                non_zero_err,
            ));
        }
        Ok(())
    }

    /// Export an environment variable up to the parent task runner env.
    /// Also sets the varible in the current environment.
    pub fn set_env<N: AsRef<str>, V: AsRef<str>>(&self, name: N, val: V) {
        let name = name.as_ref();
        let val = val.as_ref();

        std::env::set_var(name, val);

        let directive = format!("@ct-set-env@ {}={} @@\n", name, val);

        let mut p = self.cargo_task_target.clone();
        let directive_file_name =
            format!("task-directive-{}.atat", std::process::id());
        p.push(directive_file_name);

        FRESH_DIRECTIVE.call_once(|| {
            let _ = std::fs::remove_file(&p);
        });

        let mut f = ct_check_fatal!(std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&p));

        use std::io::Write;
        ct_check_fatal!(f.write_all(directive.as_bytes()));
        ct_check_fatal!(f.sync_all());
    }
}

static FRESH_DIRECTIVE: std::sync::Once = std::sync::Once::new();

/// Cargo-task task metadata struct.
#[derive(Debug)]
pub struct CTTaskMeta {
    /// task name
    pub name: String,

    /// `true` if this task was specified as a single `*.ct.rs` script file.
    /// `false` if this task was specified as a full crate directory.
    pub is_script: bool,

    /// Minimum cargo-task utility version required for this task.
    pub min_version: Option<String>,

    /// task "crate" path
    pub path: PathBuf,

    /// does this path run on default `cargo task` execution?
    pub default: bool,

    /// should this task always run before other tasks?
    /// task list will be reloaded after all bootstrap tasks run.
    pub bootstrap: bool,

    /// help info for this task
    pub help: String,

    /// any cargo (Cargo.toml) dependencies for a script task
    pub cargo_deps: Option<String>,

    /// any cargo-task task dependencies
    pub task_deps: Vec<String>,
}

/// Log Level enum for CT logging
#[derive(Clone, Copy)]
pub enum CTLogLevel {
    /// Informational message
    Info,

    /// Warning message
    Warn,

    /// Fatal message
    Fatal,
}

/// Generic CT log function
pub fn ct_log(lvl: CTLogLevel, text: &str) {
    let with_color = std::env::var_os("CT_NO_COLOR").is_none()
        && (std::env::var_os("CT_WITH_COLOR").is_some() || DEFAULT_WITH_COLOR);

    let task_name = std::env::var_os("CT_CUR_TASK")
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "".to_string());

    let t_colon = if task_name.is_empty() { "" } else { ":" };

    let base = if with_color { "\x1b[97m" } else { "" };
    let reset = if with_color { "\x1b[0m" } else { "" };

    let (lvl_name, log) = match lvl {
        CTLogLevel::Info => ("INFO", "\x1b[92m"),
        CTLogLevel::Warn => ("WARN", "\x1b[93m"),
        CTLogLevel::Fatal => ("FATAL", "\x1b[91m"),
    };

    let log = if with_color { log } else { "" };

    for line in text.split('\n') {
        eprintln!(
            "{}[ct:{}{}{}{}{}]{} {}",
            base, log, lvl_name, base, t_colon, task_name, reset, line
        );
    }

    if let CTLogLevel::Fatal = lvl {
        std::process::exit(1);
    }
}

/// Info level log function
pub fn ct_info(text: &str) {
    ct_log(CTLogLevel::Info, text)
}

/// Warn level log function
pub fn ct_warn(text: &str) {
    ct_log(CTLogLevel::Warn, text)
}

/// Fatal level log function
pub fn ct_fatal(text: &str) -> ! {
    ct_log(CTLogLevel::Fatal, text);
    std::process::exit(1);
}

// -- private -- //

#[cfg(windows)]
const DEFAULT_WITH_COLOR: bool = false;
#[cfg(not(windows))]
const DEFAULT_WITH_COLOR: bool = true;

thread_local! {
    static CT_ENV: std::sync::Mutex<Rc<CTEnv>> = std::sync::Mutex::new(priv_new_env());
}

/// Gather data from environment variables to create a cargo task "env" item.
fn priv_new_env() -> Rc<CTEnv> {
    let cargo_path = match std::env::var_os("CARGO").map(PathBuf::from) {
        Some(cargo_path) => cargo_path,
        None => ct_fatal!("CARGO binary path not set in environment"),
    };
    let work_dir = match std::env::var_os("CT_WORK_DIR").map(PathBuf::from) {
        Some(work_dir) => work_dir,
        None => ct_fatal!("CT_WORK_DIR environment variable not set"),
    };
    let cargo_task_path = match std::env::var_os("CT_PATH").map(PathBuf::from) {
        Some(cargo_task_path) => cargo_task_path,
        None => ct_fatal!("CT_PATH environment variable not set"),
    };
    let cargo_task_target =
        match std::env::var_os("CT_TARGET").map(PathBuf::from) {
            Some(cargo_task_target) => cargo_task_target,
            None => ct_fatal!("CT_TARGET environment variable not set"),
        };
    let task_list = match std::env::var_os("CT_TASKS") {
        Some(args) => args
            .to_string_lossy()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        None => Vec::with_capacity(0),
    };
    let arg_list = match std::env::var_os("CT_ARGS") {
        Some(args) => args
            .to_string_lossy()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        None => Vec::with_capacity(0),
    };
    let tasks = ct_check_fatal!(enumerate_task_metadata());

    Rc::new(CTEnv {
        cargo_path,
        work_dir,
        cargo_task_path,
        cargo_task_target,
        task_list,
        arg_list,
        tasks,
    })
}

/// Loads task metadata from environment.
fn enumerate_task_metadata(
) -> Result<BTreeMap<String, CTTaskMeta>, &'static str> {
    let mut out = BTreeMap::new();

    let env = std::env::vars_os().collect::<BTreeMap<OsString, OsString>>();
    for (env_k, env_v) in env.iter() {
        let env_k = env_k.to_string_lossy();
        if env_k.starts_with("CT_TASK_") && env_k.ends_with("_PATH") {
            let name = env_k[8..env_k.len() - 5].to_string();
            let script_name = format!("CT_TASK_{}_IS_SCRIPT", name);
            let is_script = env.contains_key(&OsString::from(script_name));
            let mv_name = format!("CT_TASK_{}_MIN_VER", name);
            let min_version = env
                .get(&OsString::from(mv_name))
                .map(|v| v.to_string_lossy().to_string());
            let def_name = format!("CT_TASK_{}_DEFAULT", name);
            let default = env.contains_key(&OsString::from(def_name));
            let bs_name = format!("CT_TASK_{}_BOOTSTRAP", name);
            let bootstrap = env.contains_key(&OsString::from(bs_name));
            let help_name = format!("CT_TASK_{}_HELP", name);
            let help = env
                .get(&OsString::from(help_name))
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "".to_string());
            let deps_name = format!("CT_TASK_{}_CARGO_DEPS", name);
            let cargo_deps = env
                .get(&OsString::from(deps_name))
                .map(|v| v.to_string_lossy().to_string());
            let deps_name = format!("CT_TASK_{}_TASK_DEPS", name);
            let mut task_deps = Vec::new();
            if let Some(deps) = env.get(&OsString::from(deps_name)) {
                for dep in deps.to_string_lossy().split_whitespace() {
                    task_deps.push(dep.to_string());
                }
            }
            let path = PathBuf::from(env_v);
            out.insert(
                name.clone(),
                CTTaskMeta {
                    name,
                    is_script,
                    min_version,
                    path,
                    default,
                    bootstrap,
                    help,
                    cargo_deps,
                    task_deps,
                },
            );
        }
    }

    Ok(out)
}
