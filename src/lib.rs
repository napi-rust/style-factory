#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use lightningcss::{
  properties::custom::{Token, TokenOrValue},
  selector::{Component, Selector},
  stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
  targets::Browsers,
  values::length::LengthValue,
  visit_types,
  visitor::{Visit, VisitTypes, Visitor},
};
use std::convert::Infallible;
use std::string::String;

struct MyVisitor;

impl<'i> Visitor<'i> for MyVisitor {
  type Error = Infallible;

  fn visit_types(&self) -> VisitTypes {
    visit_types!(LENGTHS | TOKENS | SELECTORS)
  }

  fn visit_token(&mut self, token: &mut TokenOrValue<'i>) -> Result<(), Self::Error> {
    // println!("token: {:?}", token);
    // 判断如果 unit 是 rpx 的话, 就返回 2 倍 px
    match token {
      TokenOrValue::Token(value) => match value {
        Token::Dimension {
          ref mut value,
          unit,
          ..
        } => {
          if *unit == "rpx" {
            *value *= 2.0;
            *unit = "px".into();
          }
        }
        _ => {}
      },
      _ => {}
    }
    Ok(())
  }

  fn visit_length(&mut self, length: &mut LengthValue) -> Result<(), Self::Error> {
    match length {
      LengthValue::Px(px) => *length = LengthValue::Px(*px * 2.0),
      _ => {}
    }
    Ok(())
  }

  fn visit_selector(&mut self, selector: &mut Selector<'i>) -> Result<(), Self::Error> {
    // 修改 selector 的样式名, 添加一个前缀
    for component in &mut selector.iter_mut_raw_match_order() {
      match component {
        Component::Class(class) => {
          *class = format!("prefix-{}", class).into();
        }
        _ => {}
      }
    }

    Ok(())
  }
}

#[napi]
pub fn style_factory(css: String) -> String {
  let mut stylesheet = StyleSheet::parse(&css, ParserOptions::default()).unwrap();

  let targets = Browsers {
    safari: Some(11),
    ios_saf: Some(11),
    android: Some(6),
    chrome: Some(55),
    ..Browsers::default()
  };
  let minify_options = MinifyOptions {
    targets: targets.into(),
    ..Default::default()
  };

  stylesheet.minify(minify_options).unwrap();

  let printer_options = PrinterOptions {
    minify: true,
    ..Default::default()
  };

  stylesheet.visit(&mut MyVisitor).unwrap();

  let res: lightningcss::stylesheet::ToCssResult = stylesheet.to_css(printer_options).unwrap();

  (res.code).to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_style_factory_basic() {
    let input = ".body .h1{ color: #ffffff; height: 10px; width: 100rpx;  }".to_string();
    let expected = ".prefix-body .prefix-h1{color:#fff;height:20px;width:200px}".to_string();
    assert_eq!(style_factory(input), expected);
  }
}
