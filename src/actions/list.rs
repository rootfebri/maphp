use crate::Maybe;
use crate::source::SourcePHP;
use crate::static_const::{CLI, THEME};
use crate::stats::Tag;
use anyhow::bail;
use clap::Args;
use std::collections::HashSet;
use std::io::Write;
use std::num::NonZeroU64;
use std::time::Duration;

#[derive(Args, Clone, Debug)]
pub struct ListArgs {
  /// List only installed versions in local
  #[arg(long, default_value_t = false)]
  only_installed: bool,

  /// List all versions including from repo
  #[arg(long, default_value_t = false, conflicts_with_all = ["rc", "beta", "alpha"])]
  all: bool,

  /// Inlcude ALPHA version
  #[arg(long, default_value_t = false)]
  alpha: bool,

  /// Include BETA version
  #[arg(long, default_value_t = false)]
  beta: bool,

  /// Include RC version
  #[arg(long, default_value_t = false)]
  rc: bool,

  /// Fetch and update known tags
  #[arg(long, default_value_t = false)]
  fetch: bool,
}

impl ListArgs {
  pub async fn handle(&self) -> Maybe<()> {
    if self.only_installed {
      self.show_local()
    } else if self.fetch {
      self.fetch().await
    } else if let Some(tags) = find_local_tags() {
      let filtered_tags = tags.into_iter().filter(|tag| self.filter_criteria(tag)).collect::<Vec<_>>();
      if filtered_tags.len() <= 20 {
        println!("Available versions");
        for tag in filtered_tags {
          println!("    - {}", tag.name);
        }
      } else {
        dialoguer::FuzzySelect::with_theme(&*THEME)
          .with_prompt("Available versions")
          .items(filtered_tags.iter().map(Tag::as_semver))
          .max_length(20)
          .interact()
          .ok();
      }

      Ok(())
    } else {
      bail!("Couldn't find any matching version")
    }
  }

  fn filter_criteria(&self, tag: &Tag) -> bool {
    self.all || self.alpha && tag.is_alpha() || self.beta && tag.is_beta() || self.rc && tag.is_rc()
  }

  async fn fetch(&self) -> Maybe<()> {
    let mut local = find_local_tags().unwrap_or_default();
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_secs_f32(crate::static_const::FPS));

    let mut total_new_tag = 0;
    let mut page = NonZeroU64::new(1).unwrap();
    spinner.set_message(format!("Fetching page {}...", page));
    'fetch: while let Some(git_tags) = crate::stats::get_tags(page).await? {
      page = page.checked_add(1).unwrap();

      for tag in git_tags {
        if !local.insert(tag) {
          break 'fetch;
        }

        total_new_tag += 1;
      }

      spinner.set_message(format!("Fetching page {}...", page));
    }

    spinner.println(format!("Found {total_new_tag} tags, updating local file.."));

    let json = serde_json::to_string(&local)?;
    let mut file = std::fs::File::options().truncate(true).write(true).create(true).open(CLI.tags_file())?;

    match file.write(json.as_bytes()) {
      Ok(size) => {
        let message = format!(
          "New tag added: {total_new_tag}\
          \n  Written {size} bytes to local file"
        );
        spinner.finish_with_message(message);
        Ok(())
      }
      Err(err) => bail!(format!("Couldn't save fetched tags to local files: {err}")),
    }
  }

  fn show_local(&self) -> Maybe<()> {
    let locals = SourcePHP::scan_local()?.iter().map(SourcePHP::details).collect::<HashSet<_>>();

    println!("All installed version:");
    for local in locals {
      println!("  - {local}");
    }
    Ok(())
  }
}

fn find_local_tags() -> Option<HashSet<Tag>> {
  let local_tags = CLI.tags_file();

  match local_tags.exists() {
    true => {
      let reader = std::fs::File::open(local_tags).ok()?;
      serde_json::from_reader(reader).ok()
    }
    false => None,
  }
}
