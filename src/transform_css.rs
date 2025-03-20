use lightningcss::{
  properties::custom::{Token, TokenList, TokenOrValue},
  rules::{unknown::UnknownAtRule, CssRule},
  selector::{Component, Selector, SelectorList},
  stylesheet::{ParserOptions, StyleSheet},
  traits::ToCss,
  values::{ident::Ident, string::CSSString},
  visit_types,
  visitor::{Visit, VisitTypes, Visitor},
};

use parcel_selectors::{
  attr::{AttrSelectorOperator, ParsedCaseSensitivity},
  parser::LocalName,
};
use std::error::Error;
use std::string::String;

use lightningcss::stylesheet::PrinterOptions;
use lightningcss::targets::{Browsers, Targets};

fn get_printer_options<'a>() -> PrinterOptions<'a> {
  return PrinterOptions {
    minify: true,
    targets: Targets::from(Browsers {
      safari: Some(11),
      ios_saf: Some(11),
      android: Some(6),
      chrome: Some(55),
      ..Browsers::default()
    }),
    ..Default::default()
  };
}

struct MyVisitor;

impl<'i> Visitor<'i> for MyVisitor {
  type Error = Box<dyn Error>;

  fn visit_types(&self) -> VisitTypes {
    visit_types!(LENGTHS | TOKENS | SELECTORS | RULES)
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

  fn visit_rule(&mut self, rule: &mut CssRule<'i>) -> Result<(), Self::Error> {
    match rule {
      CssRule::Import(ref import_rule) => {
        *rule = CssRule::Unknown(UnknownAtRule {
          name: "import-style".into(),
          prelude: TokenList(vec![
            TokenOrValue::Token(Token::ParenthesisBlock),
            TokenOrValue::Token(Token::String(import_rule.url.to_string().into())),
            TokenOrValue::Token(Token::CloseParenthesis),
          ]),
          block: None,
          loc: import_rule.loc,
        });
      }
      _ => {}
    }
    // 确保其他规则也能被访问
    rule.visit_children(self)?;

    // rule_exit 时，处理一些特殊的选择器
    // 如果是一个独立 :host 选择器 则移除这条规则
    if let CssRule::Style(style) = rule {
      let selectors: &mut SelectorList = &mut style.selectors;
      let mut is_single_host = false;
      // 如果 SelectorList 只有一个选择器，并且这个选择器是 :host 则移除这条规则
      // .a, .b {} 不会被移除
      if selectors.0.iter().count() == 1 {
        let selector = &mut selectors.0[0];
        if selector.iter().count() == 1 {
          let selector_css_string = selector.to_css_string(get_printer_options()).unwrap();
          is_single_host = selector_css_string == "[is=__HOST__]";
        }
      }
      if is_single_host {
        // let rule_css_string = rule.to_css_string(get_printer_options()).unwrap();
        // println!("remove rule: {}", rule_css_string);
        // 修改为注释
        *rule = CssRule::Ignored;
      }
    }

    Ok(())
  }
}

pub fn transform_css(css: String) -> Result<String, Box<dyn Error>> {
  // 1. 解析 CSS（处理解析错误）
  let mut stylesheet =
    StyleSheet::parse(&css, ParserOptions::default()).map_err(|e| format!("Parse error: {}", e))?;

  // 2. 遍历规则（处理访问错误）
  stylesheet
    .visit(&mut MyVisitor)
    .map_err(|e| format!("Visit error: {}", e))?;

  // 3. 生成 CSS（处理序列化错误）
  let res: lightningcss::stylesheet::ToCssResult = stylesheet
    .to_css(get_printer_options())
    .map_err(|e| format!("Serialize error: {}", e))?;

  // 4. 返回成功结果
  Ok(res.code.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use indoc::indoc;
  use insta::assert_snapshot;

  #[test]
  fn test_transform_css_basic() {
    let input = ".body .h1 { 
    color: #ffffff; 
    height: 10px; 
    width: 100rpx;
    }"
    .to_string();
    assert_snapshot!(transform_css(input).unwrap());
  }

  #[test]
  fn test_pseudo_class() {
    let input = "#abc .a:not(div.b:not(.c:not(.d))) .e::affter {
  color: red;
}"
    .to_string();
    assert_snapshot!(transform_css(input).unwrap());
  }

  #[test]
  fn test_is_selector() {
    let input = ".a:is(.b, .c) { color: blue; }".to_string();
    assert_snapshot!(transform_css(input).unwrap());
  }

  #[test]
  fn test_where_selector() {
    let input = ".a:where(.b, .c) { color: green; }".to_string();
    assert_snapshot!(transform_css(input).unwrap());
  }

  #[test]
  fn test_has_selector() {
    let input = ".a:has(.b) { color: purple; }".to_string();
    assert_snapshot!(transform_css(input).unwrap(),);
  }

  #[test]
  fn test_star_selector() {
    let input = "* { color: black; } .a * {height: 100px;}".to_string();
    assert_snapshot!(transform_css(input).unwrap(),);
  }

  #[test]
  fn test_host_selector() {
    let input = ".a :host { color: black; }".to_string();
    assert_snapshot!(transform_css(input).unwrap());
  }

  #[test]
  fn test_import() {
    let input = "@import url('./a.css');".to_string();
    assert_snapshot!(transform_css(input).unwrap());
  }

  #[test]
  fn test_remove_single_host() {
    let input = ":host { color: black; }".to_string();
    assert_snapshot!(transform_css(input).unwrap());
  }

  #[test]
  fn test_keyframes() {
    let input = indoc! {r#"
      @-webkit-keyframes anim-show {
        100% {
          opacity: 1;
        }
      }

      @keyframes anim-show {
        100% {
          opacity: 1;
        }
      }

      @-webkit-keyframes anim-hide {
        100% {
          opacity: 0;
        }
      }

      @keyframes anim-hide {
        100% {
          opacity: 0;
        }
      }
      "#
    }
    .to_string();

    assert_snapshot!(transform_css(input).unwrap());
  }

  #[test]
  fn test_throw_error_import() {
    let input = r#"
    .a { color: red;} 
    @import url('./b.css')
  "#
    .to_string();
    let result = transform_css(input);
    match result {
      Ok(_) => panic!("Expected an error, but got Ok"),
      Err(e) => {
        assert_snapshot!(e.to_string());
      }
    }
  }

  #[test]
  fn test_throw_error_input() {
    let input = r#" .a  color: red;}"#.to_string();
    let result = transform_css(input);
    match result {
      Ok(_) => panic!("Expected an error, but got Ok"),
      Err(e) => {
        println!("Error: {}", e);
        assert_snapshot!(e.to_string());
      }
    }
  }
}
