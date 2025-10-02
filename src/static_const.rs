use crate::Cli;
use clap::Parser;
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue};
use std::ffi::OsStr;
use std::mem;
use std::path::Path;
use std::sync::atomic::AtomicU64;

pub const REPO_NAME: &str = "php-src";
pub const MIN_TAR_SIZE: usize = 1024 * 1024 * 12; // 12 MB
pub const FPS: f32 = 1f32 / 60f32;

#[repr(transparent)]
pub struct Slice {
  pub inner: [u8],
}
impl Slice {
  #[inline]
  const unsafe fn from_encoded_bytes_unchecked(s: &[u8]) -> &Slice {
    unsafe { mem::transmute(s) }
  }

  const fn from_str(s: &str) -> &Slice {
    unsafe { Slice::from_encoded_bytes_unchecked(s.as_bytes()) }
  }
}

pub const DEFAULT_WORK_DIR: &Path =
  unsafe { &*(mem::transmute::<&Slice, &OsStr>(Slice::from_str("/home/febri/.maphp")) as *const OsStr as *const Path) };

#[test]
fn test_dwd() {
  let rp = dunce::canonicalize(DEFAULT_WORK_DIR).unwrap();
  assert_eq!(rp, std::path::PathBuf::from("~"));
}

pub static PAGE: AtomicU64 = AtomicU64::new(1);

#[rustfmt::skip]
lazy_static! {
  pub static ref DOWNLOAD_URL: reqwest::Url = reqwest::Url::parse("https://api.github.com/repos/php/php-src/tarball/refs/tags/").unwrap();
  pub static ref THEME: dialoguer::theme::ColorfulTheme = dialoguer::theme::ColorfulTheme::default();
  pub static ref CLI: Cli = Cli::parse().setup().unwrap();
  pub static ref TAG_HEADERS: HeaderMap<HeaderValue> = {
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:143.0) Gecko/20100101 Firefox/143.0"));
    headers.insert("Accept", HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.5"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate, br, zstd"));
    headers.insert("Sec-GPC", HeaderValue::from_static("1"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("cross-site"));
    headers.insert("Priority", HeaderValue::from_static("u=0, i"));
    headers.insert("Pragma", HeaderValue::from_static("no-cache"));
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
    headers.insert("TE", HeaderValue::from_static("trailers"));
    headers
  };

  pub static ref DOWNLOAD_HEADERS: HeaderMap<HeaderValue> = {
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:143.0) Gecko/20100101 Firefox/143.0"));
    headers.insert("Accept", HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.5"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate, br, zstd"));
    headers.insert("Sec-GPC", HeaderValue::from_static("1"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("none"));
    headers.insert("Sec-Fetch-User", HeaderValue::from_static("?1"));
    headers.insert("Priority", HeaderValue::from_static("u=0, i"));
    headers.insert("Pragma", HeaderValue::from_static("no-cache"));
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
    headers
  };
}
