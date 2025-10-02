use crate::static_const::{CLI, DOWNLOAD_HEADERS};
use crate::static_const::{DOWNLOAD_URL, MIN_TAR_SIZE};
use crate::{Maybe, dl_template};
use anyhow::bail;
use bytes::Bytes;
use flate2::read::GzDecoder;
use futures_util::stream::BoxStream;
use futures_util::{Stream, StreamExt};
use indicatif::{HumanBytes, ProgressBar};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use std::task::Poll::Ready;
use std::task::{Context, Poll, ready};
use std::time::Duration;
use tar::{Entries, Unpacked};

type Response = Result<Bytes, reqwest::Error>;

pub struct Downloader {
  progress: Option<ProgressBar>,
  stream: BoxStream<'static, Response>,
  archive: Option<Vec<u8>>,
}

impl Downloader {
  fn new_progress() -> ProgressBar {
    let progress = ProgressBar::new_spinner();
    progress.set_style(dl_template());
    progress.enable_steady_tick(Duration::from_secs_f32(crate::static_const::FPS));
    progress
  }

  pub async fn new(tag: &str) -> Maybe<Self> {
    let url = DOWNLOAD_URL.join(&format!("php-{tag}"))?;
    let response = reqwest::Client::new()
      .get(url)
      .headers(DOWNLOAD_HEADERS.clone())
      .send()
      .await?
      .error_for_status()?;

    Ok(Self {
      progress: Self::new_progress().into(),
      stream: Box::pin(response.bytes_stream()),
      archive: None,
    })
  }

  pub async fn start(&mut self) -> Maybe<()> {
    let mut archive = Vec::with_capacity(MIN_TAR_SIZE);

    while let Some(res) = self.next().await {
      archive.extend(res?)
    }

    self.archive.replace(archive);
    Ok(())
  }

  pub fn extract(&mut self, tag: &str, verbose: bool) -> Maybe<PathBuf> {
    let Some(archive) = self.archive.take() else {
      bail!("No archive have been downloaded")
    };

    println!("Downloaded {}", HumanBytes(archive.len() as u64));

    let path = CLI.archives().join(tag);
    let mut tar = tar::Archive::new(GzDecoder::new(archive.as_slice()));
    tar.set_overwrite(true);
    tar.set_preserve_permissions(true);
    tar.set_preserve_mtime(true);
    let entries = tar.entries()?;
    extract_unwrap(entries, &path, verbose)?;

    Ok(path)
  }
}

impl Stream for Downloader {
  type Item = Response;
  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let this = self.get_mut();
    let item = ready!(this.stream.poll_next_unpin(cx));

    if let Some(progress) = this.progress.as_ref() {
      if let Some(Ok(bytes)) = item.as_ref() {
        progress.inc(bytes.len() as u64);
      } else {
        progress.finish();
        this.progress.take();
      }
    }

    Ready(item)
  }
}

#[allow(dead_code)]
fn extract_unwrap<R>(entries: Entries<'_, R>, dst: impl AsRef<Path>, verbose: bool) -> Maybe<()>
where
  R: std::io::Read,
{
  if !dst.as_ref().exists() {
    fs::create_dir(dst.as_ref())?;
  }

  let dst = dst.as_ref();
  let progress = Downloader::new_progress();
  let mut total = 0;
  progress.println("Extracting..");

  for mut entry in entries.flatten() {
    let relative_path = get_extract_path(dst, entry.header())?;
    if verbose {
      let message = format!("Extracting {}", relative_path.strip_prefix(dst).expect("Invalid basedir").display());
      progress.set_message(message);
    }

    if relative_path == dst {
      continue;
    }

    if let Unpacked::File(f) = entry.unpack(&relative_path)?
      && let Ok(metadata) = f.metadata()
    {
      total += metadata.len();
      if verbose {
        progress.inc(metadata.len());
      }
    }
  }

  progress.println(format!("Extracted {}", HumanBytes(total)));

  Ok(())
}

const PREFIX: &str = "php-php-src";
fn skip_if_prefix((pos, c): (usize, Component)) -> Option<Component> {
  (pos != 0 || !c.as_os_str().to_string_lossy().starts_with(PREFIX)).then_some(c)
}

fn get_extract_path(dir: &Path, header: &tar::Header) -> Maybe<PathBuf> {
  let path_info = header.path()?.components().enumerate().filter_map(skip_if_prefix).collect::<PathBuf>();

  Ok(dir.join(path_info))
}

#[cfg(test)]
mod tests {
  use flate2::read::GzDecoder;
  use indicatif::HumanBytes;
  use std::fs;

  #[test]
  fn text_extract() {
    const TEST_DIR: &str = "test-extract";

    fs::create_dir_all(TEST_DIR).unwrap();

    let file = fs::File::open("php-php-src-php-8.4.11-0-ga42bbd3.tar.gz").unwrap();
    println!("Loaded {} tar file", HumanBytes(file.metadata().unwrap().len()));

    let mut tar = tar::Archive::new(GzDecoder::new(file));
    tar.set_overwrite(true);
    tar.set_preserve_permissions(true);
    tar.set_preserve_mtime(true);
    let entries = tar.entries().unwrap();
    super::extract_unwrap(entries, TEST_DIR, true).unwrap();
  }
}
