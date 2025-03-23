use crate::compile_css::compile_css;
use std::path::Path;

#[napi(object)]
pub struct JSCompileResult {
  pub css: String,
  pub dependencies: Vec<String>,
}

#[napi(js_name = "compileCSS")]
pub fn js_compile_css(entry: String) -> Result<JSCompileResult, napi::Error> {
  let entry = Path::new(&entry);
  let result = compile_css(entry);

  match result {
    Ok(result) => Ok(JSCompileResult {
      css: result.css,
      dependencies: result.dependencies,
    }),
    Err(err) => Err(napi::Error::new(
      napi::Status::GenericFailure,
      format!("Error: {}", err),
    )),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use indoc::indoc;
  use insta::assert_snapshot;
  use std::fs;
  use tempfile::tempdir;

  #[test]
  fn test_js_compile_css() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir().unwrap();
    let entry = dir.path().join("entry.css");
    fs::write(
      &entry,
      indoc! {"
      @import 'foo.css';
      body {
        color: red;
      }
    "},
    )?;
    let foo = dir.path().join("foo.css");
    fs::write(&foo, "p { color: blue; }")?;

    let result = js_compile_css(entry.to_string_lossy().to_string()).unwrap();
    let dependencies = result.dependencies;
    assert_eq!(dependencies.len(), 2);
    assert_snapshot!(result.css);
    let dependencies_names = dependencies
      .iter()
      .map(|d| Path::new(d).file_name().unwrap().to_str().unwrap())
      .collect::<Vec<_>>();
    assert_eq!(dependencies_names, vec!["entry.css", "foo.css"]);
    Ok(())
  }
}
