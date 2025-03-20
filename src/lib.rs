#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod css_to_code;
mod transform_css;

use crate::css_to_code::{css_to_code, Css2CodeOptions};
use indoc::indoc;
use std::string::String;
use transform_css::transform_css;

#[napi]
pub fn style_factory(css_text: String) -> Result<String, napi::Error> {
  let css = transform_css(css_text)
    .map_err(|e| napi::Error::from_reason(format!("Transform error: {}", e)))?;

  let css_code = css_to_code(Css2CodeOptions {
    css: &css,
    host_css: None,
  });
  Ok(css_code)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_style_factory() {
    let css_text = r#".a { color: red }"#.to_string();
    let res = style_factory(css_text.clone());
    let expected = indoc! { r#"
      export default function styleFactory(options) {
        var prefix = options.prefix || '';
        var tag = options.tag || (tag => tag);
        var rpx = options.rpx;
        var host = options.host || 'host-placeholder';
        var css = "." + prefix + "a{color:red}";
        
        return css;
      }"#
    };
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
