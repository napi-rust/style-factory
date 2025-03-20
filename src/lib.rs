#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod css_to_code;
mod printer_options;
mod transform_css;

use std::string::String;
use transform_css::transform_css;

#[napi]
pub fn style_factory(css_text: String) -> Result<String, napi::Error> {
  let css = transform_css(css_text)
    .map_err(|e| napi::Error::from_reason(format!("Transform error: {}", e)))?;
  Ok(css)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_style_factory() {
    let css_text = r#".a { color: red }"#.to_string();
    let res = style_factory(css_text.clone());
    let expected = r#".__PREFIX__a{color:red}"#;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), expected);
  }

  #[test]
  fn test_style_factory_error() {
    let css_text = r#".a color: red}"#.to_string();
    let res = style_factory(css_text.clone());
    assert!(res.is_err());
    match res {
      Err(e) => {
        assert_eq!(
          e.reason,
          "Transform error: Parse error: Unexpected end of input at :0:15"
        );
      }
      _ => panic!("Unexpected result"),
    }
  }
}
