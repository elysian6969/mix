use self::stdio::Imp;
use crate::config::Config;
use crate::shell::Text;
use crossterm::style::Colorize;
use std::ffi::OsStr;
use std::fmt;
use std::fmt::Write;
use std::os::unix::process::CommandExt;
use std::path::Path;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::io::Result;

pub use self::stdio::Stdio;
pub use std::process::{CommandArgs, CommandEnvs};
pub use tokio::process::Child;

pub mod stdio;

pub struct Command {
    std: std::process::Command,
    stderr: Stdio,
    stdin: Stdio,
    stdout: Stdio,
    kill_on_drop: bool,
}

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self {
            std: std::process::Command::new(program),
            stderr: Stdio::inherit(),
            stdin: Stdio::inherit(),
            stdout: Stdio::inherit(),
            kill_on_drop: false,
        }
    }

    pub fn program_name<S: AsRef<OsStr>>(&mut self, name: S) -> &mut Self {
        self.std.arg0(name);
        self
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.std.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.std.args(args);
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.std.env(key, val);
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.std.envs(vars);
        self
    }

    pub fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Self {
        self.std.env_remove(key);
        self
    }

    pub fn env_clear(&mut self) -> &mut Self {
        self.std.env_clear();
        self
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.std.current_dir(dir);
        self
    }

    pub fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.stderr = cfg.into();
        self
    }

    pub fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.stdin = cfg.into();
        self
    }

    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.stdout = cfg.into();
        self
    }

    pub fn kill_on_drop(&mut self, kill_on_drop: bool) -> &mut Self {
        self.kill_on_drop = kill_on_drop;
        self
    }

    pub fn uid(&mut self, id: u32) -> &mut Self {
        self.std.uid(id);
        self
    }

    pub fn gid(&mut self, id: u32) -> &mut Self {
        self.std.gid(id);
        self
    }

    pub fn get_program(&self) -> &OsStr {
        self.std.get_program()
    }

    pub fn get_args(&self) -> CommandArgs<'_> {
        self.std.get_args()
    }

    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.std.get_envs()
    }

    pub fn get_current_dir(&self) -> Option<&Path> {
        self.std.get_current_dir()
    }

    pub async fn print(&self, config: &Config, action: impl fmt::Display) -> crate::Result<()> {
        unsafe {
            let mut buffer = String::from(">>> ");

            write!(&mut buffer, "{}", action.to_string().green())?;

            let mut args = self.get_args();
            let len = args.len();

            if len > 0 {
                ufmt::uwrite!(&mut buffer, " ").unwrap_unchecked();
            }

            for arg in args.by_ref().take(len.saturating_sub(1)) {
                let arg = arg.to_str().unwrap_unchecked();

                ufmt::uwrite!(&mut buffer, "\x1b[38;5;4m\"{}\"\x1b[m, ", arg).unwrap_unchecked();
            }

            if let Some(arg) = args.next() {
                let arg = arg.to_str().unwrap_unchecked();

                ufmt::uwrite!(&mut buffer, "\x1b[38;5;4m\"{}\"\x1b[m", arg).unwrap_unchecked();
            }

            let current_dir = self
                .get_current_dir()
                .unwrap_or_else(|| Path::new("."))
                .to_str()
                .unwrap_unchecked();

            ufmt::uwrite!(&mut buffer, " in \"{}\"\n", current_dir).unwrap_unchecked();

            Text::new(buffer).render(config.shell()).await?;
        }

        Ok(())
    }

    pub fn spawn(self) -> crate::Result<Child> {
        let mut tokio = tokio::process::Command::from(self.std);

        tokio.kill_on_drop(self.kill_on_drop);

        if let Imp::Std(stdio) = self.stderr.imp {
            tokio.stderr(stdio);
        }

        if let Imp::Std(stdio) = self.stdin.imp {
            tokio.stdin(stdio);
        }

        if let Imp::Std(stdio) = self.stdout.imp {
            tokio.stdout(stdio);
        }

        let child = tokio.spawn()?;

        Ok(child)
    }

    pub async fn fancy_spawn(self) -> crate::Result<()> {
        let mut child = self.spawn()?;

        let stderr = unsafe { child.stderr.take().unwrap_unchecked() };
        let stdout = unsafe { child.stdout.take().unwrap_unchecked() };

        let mut stderr = BufReader::new(stderr).lines();
        let mut stdout = BufReader::new(stdout).lines();

        let wait = tokio::spawn(async move { child.wait().await });
        let stderr = tokio::spawn(async move {
            while let Some(line) = stderr.next_line().await? {
                println!("!!! {line:?}");
            }

            Ok::<_, crate::Error>(())
        });

        let stdout = tokio::spawn(async move {
            while let Some(line) = stdout.next_line().await? {
                println!(">>> {line:?}");
            }

            Ok::<_, crate::Error>(())
        });

        stderr.await??;
        stdout.await??;
        wait.await??;

        Ok(())
    }
}
