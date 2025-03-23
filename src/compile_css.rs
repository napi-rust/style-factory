use crate::options::{get_parser_options, get_printer_options};
use lightningcss::bundler::{Bundler, FileProvider, SourceProvider};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct CompileResult {
  pub css: String,
  pub dependencies: Vec<String>,
}

struct TrackingProvider {
  file_provider: FileProvider,
  dependencies: Arc<Mutex<Vec<String>>>,
}

impl TrackingProvider {
  pub fn new() -> Self {
    TrackingProvider {
      file_provider: FileProvider::new(),
      dependencies: Arc::new(Mutex::new(Vec::new())),
    }
  }
}

impl SourceProvider for TrackingProvider {
  type Error = <FileProvider as SourceProvider>::Error;

  fn read(&self, path: &Path) -> Result<&str, Self::Error> {
    let result = self.file_provider.read(path)?;
    self
      .dependencies
      .lock()
      .unwrap()
      .push(path.to_string_lossy().to_string());
    Ok(result)
  }

  fn resolve(&self, specifier: &str, originating_file: &Path) -> Result<PathBuf, Self::Error> {
    self.file_provider.resolve(specifier, originating_file)
  }
}

pub fn compile_css(entry: &Path) -> Result<CompileResult, Box<dyn Error>> {
  let fs = TrackingProvider::new();
  let mut bundler = Bundler::new(&fs, None, get_parser_options());
  let stylesheet = bundler.bundle(entry).unwrap();
  let result = stylesheet.to_css(get_printer_options())?;
  let dependencies = fs.dependencies.lock().unwrap().clone();

  Ok(CompileResult {
    css: result.code,
    dependencies,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use indoc::indoc;
  use insta::assert_snapshot;
  use std::fs;
  use tempfile::tempdir;

  #[test]
  fn test_basic_bundle() -> Result<(), Box<dyn Error>> {
    let dir = tempdir()?;
    let css_path = dir.path().join("a.css");

    fs::write(
      &css_path,
      indoc! { r#"
      @import "./b.css";
      .a { color: red; .a-child { color: blue; } }
    "#},
    )?;

    let png_data: &[u8] = &[
      0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
      0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
      0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
      0xFF, 0xFF, 0x3F, 0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC, 0x59, 0xE7, 0x00, 0x00, 0x00,
      0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    fs::write(dir.path().join("b.png"), png_data)?;

    fs::write(
      dir.path().join("b.css"),
      indoc! { r#"
      @import "./c.css";
      .b { padding: 0; background: url("./b.png"); }
    "#},
    )?;

    fs::write(
      dir.path().join("c.css"),
      indoc! { r#"
      .c { padding: 0 }
    "#},
    )?;

    let result: CompileResult = compile_css(css_path.as_path())?;
    let dependencies = result.dependencies;

    assert_snapshot!(result.css);
    assert_eq!(dependencies.len(), 3);
    let dependencies_names: Vec<String> = dependencies
      .iter()
      .map(|path| path.split('/').last().unwrap().to_string())
      .collect();
    assert_eq!(dependencies_names, vec!["a.css", "b.css", "c.css"]);

    Ok(())
  }
}
