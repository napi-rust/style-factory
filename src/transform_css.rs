use crate::options::{get_minify_options, get_parser_options, get_printer_options};
use lightningcss::stylesheet::{PrinterOptions, StyleSheet};
use std::error::Error;

#[derive(Debug)]
pub struct TransformCssOptions<'a> {
  pub input: &'a str,
  pub minify: bool,
}

pub fn transform_css(options: TransformCssOptions) -> Result<String, Box<dyn Error + '_>> {
  // 将 input 的所有权转移到 parse 方法中
  let mut stylesheet = StyleSheet::parse(options.input, get_parser_options())?;

  stylesheet.minify(get_minify_options())?;

  let output = stylesheet.to_css(PrinterOptions {
    minify: options.minify, // 根据传入参数决定是否压缩输出
    ..get_printer_options()
  })?;

  Ok(output.code)
}

#[cfg(test)]
mod tests {
  use super::*;
  use indoc::indoc;
  use insta::assert_snapshot;

  #[test]
  fn test_transform_css_minify_true() {
    let options = TransformCssOptions {
      input: indoc! {r#"
      .c {
        color: green;
        backdrop-filter: blur(2px);
        background-image: url(//abc.ttt.com/abc?adfsd%3F=1231);
      }
      "#},
      minify: false,
    };

    let result = transform_css(options).unwrap();
    assert_snapshot!(result)
  }

  #[test]
  fn test_transform_css_minify_false() {
    let options = TransformCssOptions {
      input: "body { color: red; .a { color: blue } }",
      minify: false,
    };

    let result = transform_css(options).unwrap();
    assert_snapshot!(result)
  }

  #[test]
  fn test_transform_css_invalid_input() {
    let options = TransformCssOptions {
      input: "invalid-css",
      minify: true,
    };

    let result = transform_css(options);
    assert!(result.is_err());
    if let Err(e) = result {
      assert_snapshot!(e.to_string());
    }
  }

  #[test]
  fn test_transform_css_empty_input() {
    let options = TransformCssOptions {
      input: "",
      minify: true,
    };

    let result = transform_css(options).unwrap();
    assert_snapshot!(result)
  }

  #[test]
  fn test_transform_css_complex_input() {
    let options = TransformCssOptions {
      input: "h1 { font-size: 20px; } p { margin: 10px; }",
      minify: true,
    };

    let result = transform_css(options).unwrap();
    assert_snapshot!(result)
  }
}
