use crate::Maybe;
use crate::static_const::TAG_HEADERS;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use std::sync::Arc;

mod imp;
#[derive(Debug, Clone, Hash, PartialOrd, PartialEq, Ord, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Tag {
  pub name: Arc<str>,
  pub tarball_url: Url,
  pub zipball_url: Url,
  pub commit: Commit,
  pub node_id: Arc<str>,
}

impl Tag {
  pub fn as_semver(&self) -> &str {
    if self.name.starts_with("php-") {
      self.name[4..].trim()
    } else {
      self.name.trim()
    }
  }

  pub fn is_alpha(&self) -> bool {
    self.name.contains("alpha") || self.name.contains("ALPHA")
  }

  pub fn is_beta(&self) -> bool {
    self.name.contains("beta") || self.name.contains("BETA")
  }

  pub fn is_rc(&self) -> bool {
    self.name.contains("RC") || self.name.contains("rc")
  }

  pub fn is_stable(&self) -> bool {
    !self.is_beta() && !self.is_alpha() && !self.is_rc()
  }
}

#[derive(Debug, Clone, Hash, PartialOrd, PartialEq, Ord, Eq, Deserialize, Serialize)]
pub struct Commit {
  sha: Arc<str>,
  url: Url,
}

/// Fetch tags from official repo and return sets of the tags.
///
/// Returns `None` if page doesn't exist.
///
/// # Arguments
///
/// * `page` - Pagination
///
/// # Returns
///
/// * [`Ok(Some(HashSet<Tag>))`] - If there was some tags
/// * [`Ok(None)`] - if 404 or non tags
/// * [`Err(reqwest::Error)`] - otherwise reqwest errors
///
/// # Errors
///
/// Returns error if this was the case:
/// - Network connection gagal
/// - Request timeout
/// - Invalid response dari server
///
/// # Examples
///
/// ```no_run
/// use std::num::NonZero;
///
/// let page = NonZero::new(1).unwrap();
/// match get_tags(page).await? {
///     Some(tags) => println!("Found {} tags", tags.len()),
///     None => println!("Page not found"),
/// }
/// ```
pub async fn get_tags(page: NonZeroU64) -> Maybe<Option<Vec<Tag>>, reqwest::Error> {
  let url = "https://api.github.com/repos/php/php-src/tags";
  let query = [("page", page.to_string()), ("per_page", 100.to_string())];
  let client = reqwest::Client::builder().default_headers(TAG_HEADERS.clone()).build()?;
  let response = client.get(url).query(&query).send().await?;

  if response.status().as_u16() == 404 {
    return Ok(None);
  }

  let tags: Vec<_> = response
    .json::<Vec<Tag>>()
    .await?
    .into_iter()
    .filter_map(|tag| tag.name.starts_with("php-").then_some(tag))
    .collect();

  Ok(tags.len().gt(&1).then_some(tags))
}

#[tokio::test]
async fn test_fetch_tags() {
  let tags = get_tags(NonZeroU64::new(1).unwrap()).await;
  assert!(tags.is_ok());
  let tags = tags.unwrap().unwrap();
  assert_eq!(tags.len(), 100);

  // PHP repo tags has whoping 1400+ tags
  let tags = get_tags(NonZeroU64::new(10).unwrap()).await;
  assert!(tags.is_ok());
  let tags = tags.unwrap().unwrap();
  assert_eq!(tags.len(), 100);
}
