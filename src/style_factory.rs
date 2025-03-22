use crate::convert_css::convert_css;
use crate::css_to_code::{css_to_code, Css2CodeOptions};
use std::string::String;

#[napi(js_name = "styleFactory")]
pub fn style_factory(css_text: String) -> Result<String, napi::Error> {
  let transform_return = convert_css(css_text)
    .map_err(|e| napi::Error::from_reason(format!("Transform error: {}", e)))?;

  let css_code = css_to_code(Css2CodeOptions {
    css: &transform_return.css,
    host_css: transform_return.host_css.as_deref(),
  });
  Ok(css_code)
}

#[cfg(test)]
mod tests {
  use super::*;
  use insta::assert_snapshot;

  #[test]
  fn test_style_factory() {
    let css_text = r#".a { color: red }"#.to_string();
    let res = style_factory(css_text);
    assert!(res.is_ok());
    assert_snapshot!(res.unwrap());
  }

  #[test]
  fn test_style_factory_error() {
    let css_text = r#".a color: red}"#.to_string();
    let res = style_factory(css_text);
    assert!(res.is_err());
    match res {
      Err(e) => {
        assert_snapshot!(e.reason,);
      }
      _ => panic!("Unexpected result"),
    }
  }
}
