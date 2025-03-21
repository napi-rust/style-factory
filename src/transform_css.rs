use lightningcss::{
  properties::custom::{Function, Token, TokenList, TokenOrValue},
  rules::{unknown::UnknownAtRule, CssRule},
  selector::{Component, Selector, SelectorList},
  stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
  traits::ToCss,
  values::{ident::Ident, string::CSSString},
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

struct FactoryVisitor {
  types: VisitTypes,
  host_css_vec: Vec<String>,
}

impl<'i> Visitor<'i> for FactoryVisitor {
  type Error = Box<dyn Error>;

  fn visit_types(&self) -> VisitTypes {
    self.types
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
      TokenOrValue::Var(ref mut var) => {
        var.fallback.visit_children(self)?;
      }
      _ => {}
    }
    Ok(())
  }

  fn visit_function(&mut self, function: &mut Function<'i>) -> Result<(), Self::Error> {
    let token_list = &mut function.arguments;
    token_list.visit_children(self)?;
    Ok(())
  }

  fn visit_token_list(&mut self, tokens: &mut TokenList<'i>) -> Result<(), Self::Error> {
    if self.types.contains(VisitTypes::TOKENS) {
      for token in &mut tokens.0 {
        token.visit_children(self)?;
      }
    }
    tokens.visit_children(self)?;
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
        _ => {
          // 其他选择器不做处理
        }
      }
    }

    Ok(())
  }

  fn visit_rule(&mut self, rule: &mut CssRule<'i>) -> Result<(), Self::Error> {
    match rule {
      CssRule::Import(ref import_rule) => {
        // @import url('./a.css'); => @import-style ("./a.css");
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
      _ => {
        // 确保其他规则也能被访问
        rule.visit_children(self)?;
      }
    }

    // println!("rule: {:?}", rule);

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
        let host_css = rule.to_css_string(PrinterOptions::default()).unwrap();
        // 怎么返回这个 host_css, 添加到一个数组里面，最后返回
        self.host_css_vec.push(host_css);
        *rule = CssRule::Ignored;
      }
    }

    Ok(())
  }
}

#[derive(Debug)]
pub struct TransformReturn {
  pub css: String,
  pub host_css: Option<String>,
}

pub fn transform_css(css: String) -> Result<TransformReturn, Box<dyn Error>> {
  // 1. 解析 CSS（处理解析错误）
  let mut stylesheet =
    StyleSheet::parse(&css, ParserOptions::default()).map_err(|e| format!("Parse error: {}", e))?;

  let mut visitor = FactoryVisitor {
    types: VisitTypes::all(),
    host_css_vec: Vec::new(),
  };

  // 2. 遍历规则（处理访问错误）
  stylesheet
    .visit(&mut visitor)
    .map_err(|e| format!("Visit error: {}", e))?;

  stylesheet
    .minify(MinifyOptions::default())
    .map_err(|e| format!("Minify error: {}", e))?;

  // 3. 生成 CSS（处理序列化错误）
  let res: lightningcss::stylesheet::ToCssResult = stylesheet
    .to_css(get_printer_options())
    .map_err(|e| format!("Serialize error: {}", e))?;

  let host_css_string = if visitor.host_css_vec.len() > 0 {
    let host_css_css = visitor.host_css_vec.join("\n");

    let mut host_stylesheet = StyleSheet::parse(&host_css_css, ParserOptions::default())
      .map_err(|e| format!("Parse host error: {}", e))?;

    host_stylesheet
      .minify(MinifyOptions::default())
      .map_err(|e| format!("Minify host error: {}", e))?;

    let host_css_css = host_stylesheet
      .to_css(get_printer_options())
      .map_err(|e| format!("Serialize host error: {}", e))?;

    Some(host_css_css.code)
  } else {
    None
  };

  // 4. 返回成功结果
  Ok(TransformReturn {
    css: res.code,
    host_css: host_css_string,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use indoc::indoc;
  use insta::assert_snapshot;

  #[test]
  fn test_transform_css_basic() {
    let input = indoc! {r#"
      .body .h1 { 
        --width: 100rpx;
        color: #ffffff; 
        height: -20rpx; 
        width: var(--width, 200rpx);
        transform: translateX(100rpx);
        calc: calc(100rpx+ -100rpx);
      }
    "#
    };

    let result = transform_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_pseudo_class() {
    let input = "#abc .a:not(div.b:not(.c:not(.d))) .e::affter {
  color: red;
}"
    .to_string();
    assert_snapshot!(transform_css(input).unwrap().css);
  }

  #[test]
  fn test_is_selector() {
    let input = ".a:is(.b, .c) { height: calc(50rpx - var(--abc, 100rpx)); }".to_string();
    let result = transform_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_where_selector() {
    let input = ".a:where(.b, .c) { color: green; }".to_string();
    let result = transform_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_has_selector() {
    let input = ".a:has(.b) { color: purple; }".to_string();
    let result = transform_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_star_selector() {
    let input = "* { color: black; } .a * {height: 100px;}".to_string();
    let result = transform_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_host_selector() {
    let input = ".a :host { color: black; }".to_string();
    let result = transform_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_import() {
    let input = "@import url('./a.css');".to_string();
    let result = transform_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_remove_single_host() {
    let input = ":host { color: black; }".to_string();
    let result = transform_css(input.to_string());
    let result_unwrapped = result.unwrap();
    assert_snapshot!(result_unwrapped.css);
    assert_snapshot!(result_unwrapped.host_css.unwrap_or_default());
  }

  #[test]
  fn test_remove_mutil_host() {
    let input = ":host { color: black; width: 100rpx } :host { height: 20rpx; }".to_string();
    let result = transform_css(input.to_string());
    let result_unwrapped = result.unwrap();
    assert_snapshot!(result_unwrapped.css);
    assert_snapshot!(result_unwrapped.host_css.unwrap_or_default());
  }

  #[test]
  fn test_keyframes() {
    let input = indoc! {r#"
      @-webkit-keyframes anim-show {
        100% {
          opacity: 1;
          width: 30rpx
        }
      }

      @keyframes anim-show {
        100% {
          opacity: 1;
          height: -30rpx
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

    let result = transform_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  // 不支持 media query 进行 rpx 设置会报错
  fn test_media_query() {
    let input = indoc! {r#"
    @media screen and (max-width: 600rpx) {
      .responsive {
        font-size: 24rpx;
      }
    }
  "#}
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
