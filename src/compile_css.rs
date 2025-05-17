use crate::options::{get_parser_options, get_printer_options};
use indexmap::IndexSet;
use lightningcss::bundler::{Bundler, FileProvider, SourceProvider};
use std::collections::HashMap;
use std::error::Error;
use std::io::{Error as IoError, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock}; // 新增

#[derive(Debug)]
pub struct CompileResult {
  pub css: String,
  pub dependencies: Vec<PathBuf>,
  pub imports: HashMap<PathBuf, Vec<PathBuf>>,
}

struct TrackingProvider {
  file_provider: FileProvider,
  dependencies: Arc<Mutex<IndexSet<PathBuf>>>, // 保证顺序
  imports: Arc<RwLock<HashMap<PathBuf, Vec<PathBuf>>>>,
}

impl TrackingProvider {
  pub fn new() -> Self {
    TrackingProvider {
      file_provider: FileProvider::new(),
      dependencies: Arc::new(Mutex::new(IndexSet::new())),
      imports: Arc::new(RwLock::new(HashMap::new())),
    }
  }
}

impl SourceProvider for TrackingProvider {
  type Error = <FileProvider as SourceProvider>::Error;

  fn read(&self, path: &Path) -> Result<&str, Self::Error> {
    let result = self.file_provider.read(path)?;
    self.dependencies.lock().unwrap().insert(path.to_path_buf());
    Ok(result)
  }

  fn resolve(&self, specifier: &str, originating_file: &Path) -> Result<PathBuf, Self::Error> {
    let result: PathBuf = self.file_provider.resolve(specifier, originating_file)?;
    let mut imports = self.imports.write().unwrap();
    imports
      .entry(originating_file.to_path_buf())
      .or_default()
      .push(result.clone());
    Ok(result)
  }
}

pub fn compile_css(entry: &Path) -> Result<CompileResult, Box<dyn Error>> {
  let fs = TrackingProvider::new();
  let mut bundler = Bundler::new(&fs, None, get_parser_options());
  let stylesheet = bundler.bundle(entry).map_err(|e| {
    let error: IoError = IoError::new(ErrorKind::Other, format!("BundleErrorKind: {}", e));
    Box::new(error)
  })?;
  let result = stylesheet.to_css(get_printer_options())?;

  let dependencies = fs.dependencies.lock().unwrap().iter().cloned().collect(); // 顺序稳定
  let imports = fs.imports.read().unwrap().clone();

  Ok(CompileResult {
    css: result.code,
    dependencies,
    imports,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::node_path::get_basename;
  use indoc::indoc;
  use insta::assert_snapshot;
  use std::fs;
  use tempfile::tempdir;

  #[test]
  fn test_basic_bundle() {
    let dir = tempdir().unwrap();
    let css_path = dir.path().join("a.css");

    fs::write(
      &css_path,
      indoc! { r#"
      @import "./b.css";
      .a { color: red; .a-child { color: blue; } }
    "#},
    )
    .unwrap();

    let png_data: &[u8] = &[
      0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
      0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
      0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
      0xFF, 0xFF, 0x3F, 0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC, 0x59, 0xE7, 0x00, 0x00, 0x00,
      0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    fs::write(dir.path().join("b.png"), png_data).unwrap();

    fs::write(
      dir.path().join("b.css"),
      indoc! { r#"
      @import "./c.css";
      .b { padding: 0; background: url("./b.png"); }
    "#},
    )
    .unwrap();

    fs::write(
      dir.path().join("c.css"),
      indoc! { r#"
      .c { padding: 0 }
    "#},
    )
    .unwrap();

    let return_compile_result: CompileResult = compile_css(css_path.as_path()).unwrap();
    let return_dependencies = return_compile_result.dependencies;

    assert_snapshot!(return_compile_result.css);
    assert_eq!(return_dependencies.len(), 3);
    let dependencies_names: Vec<String> = return_dependencies
      .iter()
      .map(|path| get_basename(path, true).unwrap())
      .collect();
    assert_eq!(dependencies_names, vec!["a.css", "b.css", "c.css"]);

    let return_imports = return_compile_result.imports;

    let mut keys: Vec<&PathBuf> = return_imports.keys().collect();
    keys.sort();

    for key in keys {
      let key_str = key.to_string_lossy().to_string();
      let values = return_imports.get(key).unwrap();
      let values_str: Vec<String> = values
        .iter()
        .map(|v| get_basename(v, true).unwrap())
        .collect();
      let key_str = get_basename(key_str, true).unwrap();

      assert_snapshot!(format!("{}: {:?}", key_str, values_str));
    }
  }

  #[test]
  fn test_bundle_err() {
    let dir = tempdir().unwrap();
    let css_path = dir.path().join("a.css");
    fs::write(
      &css_path,
      indoc! { r#"
      @import "https://example.com/b.css";
        .a { color: red; .a-child { color: blue; } }
      "# },
    )
    .unwrap();

    let result = compile_css(css_path.as_path());

    assert!(result.is_err());
  }

  #[test]
  fn test_parse_err() {
    let dir = tempdir().unwrap();
    let css_path = dir.path().join("a.css");
    fs::write(
      &css_path,
      indoc! { r#"
        "a { color: red; } .a color: blue; height: 100px}"
      "# },
    )
    .unwrap();

    let result = compile_css(css_path.as_path());

    assert!(result.is_err());
  }
}
