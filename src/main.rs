#[cfg(windows)]
compile_error!("Windows feature unimplemented yet");

use crate::actions::list::ListArgs;
use crate::downloader::Downloader;
use crate::source::SourcePHP;
use crate::static_const::{CLI, THEME};
use anyhow::{bail, ensure};
use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{HumanBytes, ProgressState, ProgressStyle};
use std::env::var;
use std::ffi::{OsStr, OsString};
use std::fs::create_dir_all;
use std::path::PathBuf;

type Maybe<T, E = anyhow::Error> = Result<T, E>;

pub mod actions;
mod downloader;
mod imp;
pub mod source;
pub mod static_const;
pub mod stats;

#[derive(Parser, Debug)]
#[command(author, version, about = "A PHP CLI manager", long_about = None)]
pub struct Cli {
  /// All available commands
  #[command(subcommand)]
  command: Commands,

  /// Change default root managed directory
  #[arg(long, env, default_value = static_const::DEFAULT_WORK_DIR.as_os_str(), value_parser = parse_work_dir)]
  work_dir: PathBuf,
}

fn parse_work_dir(value: &str) -> Result<PathBuf, String> {
  if value == "~"
    && let Ok(home) = var("HOME")
  {
    Ok(PathBuf::from(home))
  } else {
    Ok(PathBuf::from(value))
  }
}

impl Cli {
  pub fn setup(mut self) -> Maybe<Self> {
    if !self.work_dir.ends_with(".maphp") {
      self.work_dir = self.work_dir.join(".maphp");
    }

    self.work_dir = match dunce::canonicalize(&self.work_dir) {
      Ok(dir) => dir,
      Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
        create_dir_all(&self.work_dir)?;
        dunce::canonicalize(self.work_dir)?
      }
      Err(err) => return Err(err.into()),
    };

    create_dir_all(self.archives())?;
    Ok(self)
  }

  pub fn tags_file(&self) -> PathBuf {
    self.work_dir.join("tags.json")
  }

  pub fn bin(&self) -> PathBuf {
    self.work_dir.join("bin")
  }

  pub fn archives(&self) -> PathBuf {
    self.work_dir.join("archives")
  }

  pub async fn run(&self) -> Maybe<()> {
    match self.command {
      Commands::Install { ref tag, .. } => self.install(tag).await,
      Commands::Remove { ref tag } => self.remove(tag.as_deref()),
      Commands::List(ref args) => args.handle().await,
      Commands::Use { ref tag } => self.r#use(tag.as_deref()).await,
    }
  }

  async fn install(&self, tag: &str) -> Maybe<()> {
    let src = self.archives().join(tag);

    if self.command.is_force() || !src.join("buildconf").is_file() {
      let mut downloader = Downloader::new(tag).await?;
      downloader.start().await?;
      if self.command.is_force() {
        _ = std::fs::remove_dir_all(&src);
        _ = std::fs::remove_file(&src);
      }
      downloader.extract(tag, self.command.is_verbose())?;
    }

    let source = SourcePHP::new(&src);
    if self.command.is_force() || !source.is_installed() {
      let compilation = source.install().await;
      if compilation.is_err() {
        _ = std::fs::remove_dir_all(src.join("dist"));
        compilation?;
      }
    }

    source.setup_ini().await?;

    if dialoguer::Confirm::with_theme(&*THEME)
      .with_prompt("✅ Installation completed, Use it?")
      .interact()?
    {
      source.link().await?;
      println!("Sucess!");

      if !self.path_registered() {
        self.print_path_register();
      }
    }

    Ok(())
  }

  fn path_registered(&self) -> bool {
    let Ok(path) = var("PATH") else { return false };
    path.contains(".maphp/bin:") || env!("PATH").contains(".maphp/bin/:")
  }

  fn print_path_register(&self) {
    println!(r"# Add the following to your find PATH env:");
    println!(r#"export PATH="{}:$PATH""#, self.bin().display());
  }

  async fn r#use(&self, tag: Option<&str>) -> Maybe<()> {
    let src = match tag {
      None => self.select("Choose installed version you want to use")?,
      Some(t) => self.archives().join(t),
    };

    SourcePHP::new(src).link().await?;
    Ok(())
  }

  fn select(&self, prompt: impl AsRef<str>) -> Maybe<PathBuf> {
    let archives = std::fs::read_dir(self.archives())?
      .flatten()
      .filter_map(|dir| dir.path().is_dir().then_some(dir.file_name()))
      .collect::<Vec<_>>();

    ensure!(!archives.is_empty(), "No available installed version found");

    let pos = dialoguer::FuzzySelect::with_theme(&*THEME)
      .with_prompt(prompt.as_ref())
      .items(archives.iter().map(OsString::as_os_str).map(OsStr::to_string_lossy))
      .interact()?;

    Ok(self.archives().join(&archives[pos]))
  }

  fn remove(&self, tag: Option<&str>) -> Maybe<()> {
    let src = match tag {
      Some(t) => self.archives().join(t),
      None => self.select("Choose installed version you want to remove")?,
    };

    let source = SourcePHP::new(&src);
    let true = dialoguer::Confirm::with_theme(&*THEME)
      .with_prompt(format!("Are you sure want to remove ({})?", source.details().red()))
      .default(false)
      .interact()
      .unwrap_or(false)
    else {
      bail!("Operation canceled")
    };

    if source.is_installed() {
      std::fs::remove_dir_all(src)?;
      if source.is_in_path() {
        std::fs::remove_dir_all(self.bin()).ok();
      }

      println!("PHP {} successfully deleted", source.name());
      return Ok(());
    }

    bail!("No matching version `{}` as found", source.name())
  }
}

fn strip_php(value: &str) -> Result<String, String> {
  if &value[..4] == "php-" {
    Ok(value[4..].to_owned())
  } else {
    Ok(value.to_owned())
  }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
  /// Install PHP Version
  Install {
    #[command()]
    #[arg(value_parser = strip_php)]
    tag: String,
    #[arg(long, default_value_t = false)]
    dev: bool,
    #[arg(long, default_value_t = false)]
    verbose: bool,
    #[arg(long, default_value_t = false)]
    force: bool,
  },

  /// Removes installed PHP Version
  Remove {
    #[command()]
    #[arg(default_value = None)]
    tag: Option<String>,
  },
  /// Lists all PHP version
  List(ListArgs),

  /// Change PHP version
  Use {
    #[arg(default_value = None)]
    tag: Option<String>,
  },
}

#[tokio::main]
async fn main() -> Maybe<()> {
  CLI.run().await
}

fn dl_template() -> ProgressStyle {
  ProgressStyle::with_template("{spinner:.green} [{bar:30.cyan/blue}] {bytes} | {speed}")
    .unwrap()
    .with_key("speed", |s: &ProgressState, w: &mut dyn std::fmt::Write| {
      let persec = s.per_sec().abs() as u64;
      write!(w, "{}/s", HumanBytes(persec)).unwrap()
    })
    .progress_chars("#>-")
}
