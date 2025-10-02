use crate::Commands;

impl Commands {
  pub fn is_verbose(&self) -> bool {
    matches!(*self, Self::Install { verbose: true, .. })
  }

  pub fn is_dev(&self) -> bool {
    matches!(*self, Self::Install { dev: true, .. })
  }

  pub fn is_force(&self) -> bool {
    matches!(*self, Self::Install { force: true, .. })
  }
}
