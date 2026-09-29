//! EPUB 解析错误类型

/// EPUB 解析错误
#[derive(Debug)]
pub struct EpubError(pub String);

impl std::fmt::Display for EpubError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::error::Error for EpubError {}
