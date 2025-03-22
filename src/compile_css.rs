use crate::options::{get_parser_options, get_printer_options};
use lightningcss::bundler::{Bundler, FileProvider, SourceProvider};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[napi(object)]
pub struct CompileResult {
  pub css: String,
  pub dependencies: Vec<String>,
}

struct TrackingProvider {
  inner: FileProvider,
  dependencies: Arc<Mutex<Vec<String>>>,
}

impl TrackingProvider {
  pub fn new() -> Self {
    TrackingProvider {
      inner: FileProvider::new(),
      dependencies: Arc::new(Mutex::new(Vec::new())),
    }
  }
}

impl SourceProvider for TrackingProvider {
  type Error = napi::Error;

  fn read(&self, path: &Path) -> Result<&str, Self::Error> {
    let result = self.inner.read(path);
    if let Ok(data) = result {
      self
        .dependencies
        .lock()
        .unwrap()
        .push(path.to_string_lossy().to_string());
      Ok(data)
    } else {
      Err(napi::Error::new(
        napi::Status::GenericFailure,
        format!("Failed to read file: {:?}", path),
      ))
    }
  }

  fn resolve(&self, specifier: &str, originating_file: &Path) -> Result<PathBuf, Self::Error> {
    self
      .inner
      .resolve(specifier, originating_file)
      .map_err(|e| {
        napi::Error::new(
          napi::Status::GenericFailure,
          format!("Failed to resolve file: {:?}", e),
        )
      })
  }
}

/*
 接受一个 css 文件路径, 进行 bundle 操作
 并且收集 css 的依赖
*/
#[napi(js_name = "compileCSS")]
pub fn compile_css(entry: String) -> Result<CompileResult, napi::Error> {
  let fs = TrackingProvider::new(); // Using the new function of TrackingProvider
  let mut bundler = Bundler::new(&fs, None, get_parser_options());
  let stylesheet = match bundler.bundle(Path::new(&entry)) {
    Ok(stylesheet) => stylesheet,
    Err(e) => {
      return Err(napi::Error::new(
        napi::Status::GenericFailure,
        format!("Failed to bundle stylesheet: {:?}", e),
      ));
    }
  };

  let result = stylesheet.to_css(get_printer_options()).map_err(|e| {
    napi::Error::new(
      napi::Status::GenericFailure,
      format!("Failed to convert stylesheet to CSS: {:?}", e),
    )
  });

  let dependencies = fs.dependencies.lock().unwrap().clone();

  Ok(CompileResult {
    css: result.unwrap().code,
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
  fn test_basic_bundle() -> Result<(), napi::Error> {
    let dir = tempdir()?;
    let css_path = dir.path().join("a.css");

    // 创建测试文件
    fs::write(
      &css_path,
      indoc! { r#"
        @import "./b.css";
        .a { color: red;
          .a-child { color: blue; }
        }
    "#},
    )?;

    // 这是一个 1x1 像素红色 PNG 的二进制数据
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
        .b {
          padding: 0;
          background: url("./b.png");
        }
    "#},
    )?;

    fs::write(
      dir.path().join("c.css"),
      indoc! { r#"
        .c { padding: 0 }
    "#},
    )?;

    let result: CompileResult = compile_css(css_path.to_string_lossy().to_string())?;

    let dependencies = result.dependencies;
    // 取出文件名
    let dependencies_name: Vec<String> = dependencies
      .iter()
      .map(|x| x.split('/').last().unwrap().to_string())
      .collect();
    assert_eq!(dependencies.len(), 3);
    assert_snapshot!(result.css);
    assert_eq!(dependencies_name, vec!["a.css", "b.css", "c.css"]);
    Ok(())
  }
}
