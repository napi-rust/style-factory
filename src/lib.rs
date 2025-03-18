#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn style_factory(css: String) -> String {
  css
}

#[cfg(test)]
mod tests {
  use super::style_factory;

  #[test]
  fn test_style_factory() {
    assert_eq!(style_factory("body { color: red; }".to_string()), "body { color: red; }");
  }
}