use crate::options::{get_minify_options, get_parser_options, get_printer_options};
use lightningcss::stylesheet::{PrinterOptions, StyleSheet};
use std::error::Error;
use std::io::{Error as IoError, ErrorKind};

#[derive(Debug)]
pub struct TransformCssOptions {
  pub input: String,
  pub minify: bool,
}

pub fn transform_css(options: TransformCssOptions) -> Result<String, Box<dyn Error>> {
  // 解析 CSS 文本
  let mut stylesheet = StyleSheet::parse(&options.input, get_parser_options()).map_err(|e| {
    // 处理解析错误
    let error: IoError = IoError::new(ErrorKind::Other, format!("CSS parsing error:: {}", e));
    Box::new(error)
  })?;

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
      "#}
      .parse()
      .unwrap(),
      minify: false,
    };

    let result = transform_css(options).unwrap();
    assert_snapshot!(result)
  }

  #[test]
  fn test_transform_css_minify_false() {
    let options = TransformCssOptions {
      input: "body { color: red; .a { color: blue } }".to_string(),
      minify: false,
    };

    let result = transform_css(options).unwrap();
    assert_snapshot!(result)
  }

  #[test]
  fn test_transform_css_invalid_input() {
    let options = TransformCssOptions {
      input: "invalid-css".to_string(),
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
      input: "".to_string(),
      minify: true,
    };

    let result = transform_css(options).unwrap();
    assert_snapshot!(result)
  }

  #[test]
  fn test_transform_css_complex_input() {
    let options = TransformCssOptions {
      input: "h1 { font-size: 20px; } p { margin: 10px; }".to_string(),
      minify: true,
    };

    let result = transform_css(options).unwrap();
    assert_snapshot!(result)
  }
}
