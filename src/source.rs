use crate::static_const::CLI;
use crate::{Commands, Maybe};
use anyhow::ensure;
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug)]
pub struct SourcePHP(
  /// The path of PHP source code not the compiled
  PathBuf,
);

impl SourcePHP {
  const TEMPLATE_STR: &'static str = "{spinner:.green} {prefix}\n\
  {wide_msg}";

  pub(crate) fn new_spinner() -> ProgressBar {
    let spinner = ProgressBar::new_spinner()
      .with_style(ProgressStyle::default_spinner().template(Self::TEMPLATE_STR).unwrap())
      .with_finish(ProgressFinish::AndLeave);
    spinner.enable_steady_tick(Duration::from_secs_f32(crate::static_const::FPS));
    spinner
  }

  pub fn new(src: impl Into<PathBuf>) -> Self {
    let spinner = ProgressBar::new_spinner().with_style(ProgressStyle::default_spinner().template(Self::TEMPLATE_STR).unwrap());
    spinner.enable_steady_tick(Duration::from_secs_f32(crate::static_const::FPS));

    Self(src.into())
  }

  pub fn is_installed(&self) -> bool {
    self.0.join("dist/bin/php").is_file()
  }

  /// # Return
  /// dist pathbuf
  pub async fn install(&self) -> Maybe<PathBuf> {
    if self.is_installed() && !CLI.command.is_force() {
      return Ok(self.0.join("dist"));
    }

    self.build_conf().await?;
    self.configure().await?;
    self.make_install().await?;

    Ok(self.0.join("dist"))
  }

  async fn build_conf(&self) -> Maybe<()> {
    let mut build_conf = Command::new("sh");
    let cmd = build_conf.arg(self.0.join("buildconf")).arg("--force").current_dir(&self.0);

    self.run_with_spinner("sh buildconnf --force", cmd).await?;

    Ok(())
  }

  fn get_args(&self) -> Vec<String> {
    macro_rules! emit {
      ($ident:ident, $a:ident $(,)?) => {
        if $ident {
          $a.push(::core::concat!("--", ::core::stringify!($ident)).replace("_", "-"));
        }
      };
      (PathBuf($value:ident), $vec:ident $(,)?) => {
        if $value.as_os_str() == "default" {
          $vec.push(::core::concat!("--", ::core::stringify!($value)).replace("_", "-"));
        } else if !$value.as_os_str().is_empty() {
          $vec.push(::core::concat!("--", ::core::stringify!($value), "=").replace("_", "-") + $value.as_os_str().to_str().unwrap());
        }
      };
    }

    let Commands::Install {
      enable_calendar,
      enable_intl,
      enable_mbstring,
      enable_pcntl,
      enable_bcmath,
      enable_mysqlnd,
      with_curl,
      with_openssl,
      with_pear,
      with_zip,
      with_zlib,
      with_password_argon2,
      ref with_mysqli,
      ref with_pdo_mysqli,
      ref with_pgsql,
      ref with_pdo_pgsql,
      ref configure_args,
      ..
    } = CLI.command
    else {
      unreachable!("Unreachable code!")
    };

    let mut args = vec![];
    emit!(enable_calendar, args);
    emit!(enable_intl, args);
    emit!(enable_mbstring, args);
    emit!(enable_pcntl, args);
    emit!(enable_bcmath, args);
    emit!(enable_mysqlnd, args);
    emit!(with_curl, args);
    emit!(with_openssl, args);
    emit!(with_pear, args);
    emit!(with_zip, args);
    emit!(with_zlib, args);
    emit!(with_password_argon2, args);

    emit!(PathBuf(with_mysqli), args);
    emit!(PathBuf(with_pdo_mysqli), args);
    emit!(PathBuf(with_pgsql), args);
    emit!(PathBuf(with_pdo_pgsql), args);

    args.push(configure_args.join(" "));
    args
  }

  async fn configure(&self) -> Maybe<()> {
    let mut configure = Command::new("./configure");
    let args = self.get_args();
    let cmd = configure.arg("--prefix").arg(self.0.join("dist")).args(&args).current_dir(&self.0);
    let cmd = if !CLI.command.is_dev() { cmd } else { cmd.arg("--enable-debug") };

    let prefix = format!(
      "./configure --prefix {dist} {debug}{args}",
      dist = self.0.join("dist").display(),
      debug = if CLI.command.is_dev() { "--enable-debug " } else { " " },
      args = args.join(" "),
    );

    self.run_with_spinner(prefix, cmd).await?;

    Ok(())
  }

  async fn make_install(&self) -> Maybe<()> {
    let cpus = num_cpus::get();
    let mut make = Command::new("make");
    let cmd = make.arg("install").arg(format!("-j{cpus}")).current_dir(&self.0);

    let analogy = format!("make install with {cpus} job(s)");
    self.run_with_spinner(analogy, cmd).await?;

    Ok(())
  }

  pub async fn setup_ini(&self) -> Maybe<()> {
    println!("Setting up php.ini");
    if self.0.join("dist/lib/php.ini").is_file() {
      println!("✅ php.ini already exists, skipping");
      return Ok(());
    }

    let php_devel = self.0.join("php.ini-development");
    let php_prods = self.0.join("php.ini-production");

    let php_ini = if CLI.command.is_dev() {
      if !php_devel.is_file() {
        println!("⚠️ php.ini-development not found, skipping");
        return Ok(());
      } else {
        php_devel
      }
    } else if !php_prods.is_file() {
      println!("⚠️ php.ini-production not found, skipping");
      return Ok(());
    } else {
      php_prods
    };

    tokio::fs::copy(php_ini, self.0.join("dist/lib/php.ini")).await?;

    Ok(())
  }

  pub async fn link(&self) -> Maybe<()> {
    if CLI.bin().exists() {
      tokio::fs::remove_dir_all(CLI.bin()).await?;
    }

    #[cfg(unix)]
    tokio::fs::symlink(self.0.join("dist/bin"), CLI.bin()).await?;

    Ok(())
  }

  async fn run_with_spinner(&self, analogy: impl ToString, command: &mut Command) -> Maybe<()> {
    let spinner = Self::new_spinner();
    spinner.set_prefix(analogy.to_string());

    let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true).spawn()?;
    let stdout = child.stdout.take().expect("Unexpected STDIO piped stdout not found");
    let stderr = child.stderr.take().expect("Unexpected STDIO piped stdout not found");

    let verbose = CLI.command.is_verbose();
    let progress = spinner.clone();
    let stdout_handle = tokio::spawn(async move {
      let mut lines = BufReader::new(stdout).lines();

      while let Ok(Some(mut line)) = lines.next_line().await {
        if verbose {
          progress.println(line);
        } else {
          line.truncate(150);
          progress.set_message(line);
        }
      }
    });
    let progress = spinner.clone();
    let stderr_handle = tokio::spawn(async move {
      let mut lines = BufReader::new(stderr).lines();

      while let Ok(Some(line)) = lines.next_line().await {
        progress.println(line);
      }
    });

    let status = child.wait().await?;
    stdout_handle.await?;
    stderr_handle.await?;

    ensure!(status.success());

    Ok(())
  }

  pub fn name(&self) -> Cow<'_, str> {
    self
      .0
      .file_name()
      .map(std::ffi::OsStr::to_string_lossy)
      .unwrap_or_else(|| self.0.to_string_lossy())
  }

  pub fn details(&self) -> String {
    if !self.is_installed() || !self.is_in_path() {
      return self.name().into_owned();
    }

    let command = std::process::Command::new("php")
      .arg("-v")
      .stderr(Stdio::null())
      .stdout(Stdio::piped())
      .current_dir(self.0.join("dist/bin"))
      .output();

    let stdout = match command {
      Ok(output) if output.status.success() => output.stdout,
      _ => return self.name().into_owned(),
    };

    let Ok(stdout) = String::from_utf8(stdout) else {
      return self.name().into_owned();
    };

    match stdout.lines().next() {
      Some(line) => line.replace("PHP ", ""),
      None => self.name().into_owned(),
    }
  }

  pub fn is_in_path(&self) -> bool {
    let Ok(realpath) = dunce::realpath(CLI.bin()) else { return false };
    self.0.join("dist/bin") == realpath
  }

  pub fn scan_local() -> Maybe<Vec<Self>> {
    Ok(
      std::fs::read_dir(CLI.archives())?
        .flatten()
        .filter_map(|d| {
          let source = Self::new(d.path());
          source.is_installed().then_some(source)
        })
        .collect::<Vec<_>>(),
    )
  }
}

impl Display for SourcePHP {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.name().as_ref())
  }
}
