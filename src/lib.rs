#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use lightningcss::{
  properties::custom::{Token, TokenOrValue},
  selector::{Component, Selector},
  stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
  targets::Browsers,
  values::ident::Ident,
  values::length::LengthValue,
  values::string::CSSString,
  visit_types,
  visitor::{Visit, VisitTypes, Visitor},
};
use parcel_selectors::{
  attr::{AttrSelectorOperator, ParsedCaseSensitivity},
  parser::LocalName,
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
    match token {
      TokenOrValue::Token(value) => match value {
        Token::Dimension {
          ref mut value,
          unit,
          ..
        } => {
          if *unit == "rpx" {
            // 把当前 token 替换成  __RPX__(value) 的形式
            *token =
              TokenOrValue::Token(Token::String(format!("__RPX__({})", value).into()).into());
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
    // 修改 selector 的样式名, 添加一个 __PREFIX__ 前缀
    for component in &mut selector.iter_mut_raw_match_order() {
      match component {
        // 将类名替换成 __PREFIX__ 类名
        Component::Class(class) => {
          *class = format!("__PREFIX__{}", class).into();
        }

        // 处理 * 选择器 * => unsupport-star
        Component::ExplicitUniversalType => {
          *component = Component::LocalName(LocalName {
            name: "unsupport-star".into(),
            lower_name: "unsupport-star".into(),
          });
        }

        // 处理 :host 选择器 :host => [is=__HOST__]
        Component::Host(_host) => {
          *component = Component::AttributeInNoNamespace {
            local_name: Ident::from("is"),
            operator: AttrSelectorOperator::Equal,
            value: CSSString::from("__HOST__".to_string()),
            case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
            never_matches: false,
          };
        }

        // 将标签替换成 attribute 属性选择符  div => [meta:tag="div"]
        Component::LocalName(local_name) => {
          *component = Component::AttributeInNoNamespace {
            local_name: Ident::from("meta:tag"),
            operator: AttrSelectorOperator::Equal,
            value: CSSString::from(local_name.name.to_string()),
            case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
            never_matches: false,
          };
        }
        // 递归处理子选择器
        Component::Negation(selectors)
        | Component::Is(selectors)
        | Component::Where(selectors)
        | Component::Has(selectors) => {
          for sub_selector in selectors.iter_mut() {
            self.visit_selector(sub_selector)?;
          }
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
    let input = ".body .h1 { 
    color: #ffffff; 
    height: 10px; 
    width: 100rpx;
    }"
    .to_string();
    let expected =
      ".__PREFIX__body .__PREFIX__h1{color:#fff;height:20px;width:\"__RPX__(100)\"}".to_string();
    assert_eq!(style_factory(input), expected);
  }

  #[test]
  fn test_pseudo_class() {
    let input = "#abc .a:not(div.b:not(.c:not(.d))) .e::affter {
  color: red;
}"
    .to_string();
    let expected =
      "#abc .__PREFIX__a:not([meta\\:tag=div].__PREFIX__b:not(.__PREFIX__c:not(.__PREFIX__d))) .__PREFIX__e::affter{color:red}".to_string();
    assert_eq!(style_factory(input), expected);
  }

  #[test]
  fn test_is_selector() {
    let input = ".a:is(.b, .c) { color: blue; }".to_string();
    let expected = ".__PREFIX__a:is(.__PREFIX__b,.__PREFIX__c){color:#00f}".to_string();
    assert_eq!(style_factory(input), expected);
  }

  #[test]
  fn test_where_selector() {
    let input = ".a:where(.b, .c) { color: green; }".to_string();
    let expected = ".__PREFIX__a:where(.__PREFIX__b,.__PREFIX__c){color:green}".to_string();
    assert_eq!(style_factory(input), expected);
  }

  #[test]
  fn test_has_selector() {
    let input = ".a:has(.b) { color: purple; }".to_string();
    let expected = ".__PREFIX__a:has(.__PREFIX__b){color:purple}".to_string();
    assert_eq!(style_factory(input), expected);
  }

  #[test]
  fn test_star_selector() {
    let input = "* { color: black; } .a * {height: 100px;}".to_string();
    let expected =
      "unsupport-star{color:#000}.__PREFIX__a unsupport-star{height:200px}".to_string();
    assert_eq!(style_factory(input), expected);
  }

  #[test]
  fn test_host_selector() {
    let input = ".a :host { color: black; }".to_string();
    let expected = ".__PREFIX__a [is=__HOST__]{color:#000}".to_string();
    assert_eq!(style_factory(input), expected);
  }
}
