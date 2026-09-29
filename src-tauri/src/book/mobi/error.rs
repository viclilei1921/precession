//! MOBI 解析错误

#[derive(Debug)]
pub struct MobiError(pub String);

impl std::fmt::Display for MobiError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::error::Error for MobiError {}
