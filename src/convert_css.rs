use lightningcss::{
  properties::custom::{Token, TokenList, TokenOrValue},
  rules::{unknown::UnknownAtRule, CssRule},
  selector::{Component, Selector, SelectorList},
  stylesheet::StyleSheet,
  traits::ToCss,
  values::{ident::Ident, string::CSSString},
  visitor::{Visit, VisitTypes, Visitor},
};

use crate::options::{get_minify_options, get_parser_options, get_printer_options};
use parcel_selectors::{
  attr::{AttrSelectorOperator, ParsedCaseSensitivity},
  parser::LocalName,
};
use smallvec::SmallVec;
use std::error::Error;
use std::string::String;

const PREFIX: &str = "__PREFIX__";
const HOST: &str = "__HOST__";
const RPX_FUNC: &str = "__RPX__";
const IMPORT_STYLE: &str = "import-style";
const META_TAG: &str = "meta:tag";
const UNSUPPORTED_STAR: &str = "unsupported-star";
const UNSUPPORTED_WEB_VIEW: &str = "unsupported-web-view";

struct FactoryVisitor {
  types: VisitTypes,
  host_css_vec: SmallVec<[String; 2]>,
}

impl FactoryVisitor {
  #[inline]
  fn replace_rpx_token(token: &mut Token) {
    if let Token::Dimension {
      ref mut value,
      unit,
      ..
    } = token
    {
      if unit == &"rpx" {
        // 把当前 token 替换成  RPX_FUNC(value) 的形式
        *token = Token::String(format!("{RPX_FUNC}({})", value).into());
      }
    }
  }

  #[inline]
  fn is_host_selector(selector: &Selector) -> bool {
    if selector.iter().selector_length() != 1 {
      return false;
    }
    selector.iter().any(|component| {
      matches!(
          component,
          Component::AttributeInNoNamespace {
              local_name,
              operator,
              value,
              ..
          } if local_name == &Ident::from("is")
              && operator == &AttrSelectorOperator::Equal
              && value == &CSSString::from(HOST)
      )
    })
  }

  #[inline]
  fn has_single_selector(selectors: &SelectorList) -> bool {
    selectors.0.iter().any(Self::is_host_selector)
  }

  /// 返回过滤掉所有 host selector 的 SelectorList，避免 clone 整个列表
  #[inline]
  fn filter_non_host<'i>(selectors: &SelectorList<'i>) -> SelectorList<'i> {
    SelectorList::new(
      selectors
        .0
        .iter()
        .filter(|s| !Self::is_host_selector(s))
        .cloned()
        .collect(),
    )
  }

  #[inline]
  fn create_host_selector<'s>() -> Selector<'s> {
    Selector::from(vec![Self::create_host_component()])
  }

  #[inline]
  fn create_host_component<'c>() -> Component<'c> {
    Component::AttributeInNoNamespace {
      local_name: Ident::from("is"),
      operator: AttrSelectorOperator::Equal,
      value: CSSString::from(HOST),
      case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
      never_matches: false,
    }
  }
}

impl<'i> Visitor<'i> for FactoryVisitor {
  type Error = Box<dyn Error>;

  fn visit_types(&self) -> VisitTypes {
    self.types
  }

  fn visit_rule<'a>(&mut self, rule: &'a mut CssRule<'i>) -> Result<(), Self::Error> {
    match rule {
      CssRule::Import(import_rule) => {
        // @import url('./a.css'); => @import-style ("./a.css")
        *rule = CssRule::Unknown(UnknownAtRule {
          name: IMPORT_STYLE.into(),
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
        rule.visit_children(self)?;
      }
    }
    // rule_exit 时，处理一些特殊的选择器
    if let CssRule::Style(style) = rule {
      let has_single_host = Self::has_single_selector(&style.selectors);
      if has_single_host {
        let omit_single_selectors = Self::filter_non_host(&style.selectors);
        // 移除后，如果没有选择器了，则将当前 rule 设置为 Ignored
        // 并将 host_css 添加到 host_css_vec 中
        if omit_single_selectors.0.is_empty() {
          let host_css = rule.clone().to_css_string(get_printer_options())?;
          self.host_css_vec.push(host_css);
          *rule = CssRule::Ignored;
        } else {
          // 原 style 仅保留非 host selector，新 style 仅保留 host selector
          let mut clone_style = style.clone();
          style.selectors = omit_single_selectors;
          let mut single_selectors = SelectorList::new(SmallVec::new());
          single_selectors.0.push(Self::create_host_selector());
          clone_style.selectors = single_selectors;
          self
            .host_css_vec
            .push(clone_style.to_css_string(get_printer_options())?);
        }
      }
    }
    Ok(())
  }

  fn visit_selector(&mut self, selector: &mut Selector<'i>) -> Result<(), Self::Error> {
    if self.types.contains(VisitTypes::SELECTORS) {
      for component in &mut selector.iter_mut_raw_match_order() {
        match component {
          Component::Class(class) => {
            // 避免多余分配
            let mut buf = SmallVec::<[u8; 64]>::new();
            buf.extend_from_slice(PREFIX.as_bytes());
            buf.extend_from_slice(class.as_bytes());
            *class = String::from_utf8(buf.to_vec()).unwrap().into();
          }
          Component::ExplicitUniversalType => {
            *component = Component::LocalName(LocalName {
              name: UNSUPPORTED_STAR.into(),
              lower_name: UNSUPPORTED_STAR.into(),
            });
          }
          Component::Host(_) => {
            *component = Self::create_host_component();
          }
          // 将标签替换成 attribute 属性选择符  div => [meta:tag=div]
          Component::LocalName(local_name) => {
            // 如果是 web-view 标签, 则修改成 unsupported-web-view
            if local_name.name == "web-view" {
              *component = Component::LocalName(LocalName {
                name: UNSUPPORTED_WEB_VIEW.into(),
                lower_name: UNSUPPORTED_WEB_VIEW.into(),
              });
            } else {
              *component = Component::AttributeInNoNamespace {
                local_name: Ident::from(META_TAG),
                operator: AttrSelectorOperator::Equal,
                value: CSSString::from(local_name.name.to_string()),
                case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
                never_matches: false,
              };
            }
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
    } else {
      selector.visit_children(self)?;
    }
    Ok(())
  }

  fn visit_token(&mut self, token: &mut TokenOrValue<'i>) -> Result<(), Self::Error> {
    match token {
      TokenOrValue::Token(token) => {
        if let Token::Dimension { .. } = token {
          Self::replace_rpx_token(token);
        }
      }
      TokenOrValue::Function(function) => {
        function.arguments.visit_children(self)?;
      }
      TokenOrValue::Var(var) => {
        var.fallback.visit_children(self)?;
      }
      _ => {}
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct ConvertResult {
  pub css: String,
  pub host_css: Option<String>,
}

pub fn convert_css(css: String) -> Result<ConvertResult, Box<dyn Error>> {
  if css.is_empty() {
    return Ok(ConvertResult {
      css: String::new(),
      host_css: None,
    });
  }

  // 1. 解析 CSS（处理解析错误）
  let mut stylesheet =
    StyleSheet::parse(&css, get_parser_options()).map_err(|e| format!("Parse error: {}", e))?;

  let mut visitor = FactoryVisitor {
    types: VisitTypes::all(),
    host_css_vec: SmallVec::new(),
  };
  stylesheet.visit(&mut visitor)?;
  stylesheet.minify(get_minify_options())?;
  let res = stylesheet.to_css(get_printer_options())?;
  let host_css_string = process_host_css(&visitor.host_css_vec)?;
  Ok(ConvertResult {
    css: res.code,
    host_css: host_css_string,
  })
}

fn process_host_css(host_css_vec: &[String]) -> Result<Option<String>, Box<dyn Error>> {
  if host_css_vec.is_empty() {
    return Ok(None);
  }
  // 预估分配，提升效率
  let total_len: usize = host_css_vec.iter().map(|s| s.len() + 1).sum();
  let mut joined = String::with_capacity(total_len);
  for (i, css) in host_css_vec.iter().enumerate() {
    if i > 0 {
      joined.push('\n');
    }
    joined.push_str(css);
  }
  let mut host_stylesheet =
    StyleSheet::parse(&joined, get_parser_options()).map_err(|e| format!("Parse error: {}", e))?;
  host_stylesheet.minify(get_minify_options())?;
  let host_css_css = host_stylesheet.to_css(get_printer_options())?;
  Ok(Some(host_css_css.code))
}

#[cfg(test)]
mod tests {
  use super::*;
  use indoc::indoc;
  use insta::assert_snapshot;

  #[test]
  fn test_basic() {
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

    let result = convert_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_pseudo_class() {
    let input = "#abc .a:not(div.b:not(.c:not(.d))) .e::after {
            color: red;
        }"
    .to_string();
    assert_snapshot!(convert_css(input).unwrap().css);
  }

  #[test]
  fn test_is_selector() {
    let input = ".a:is(.b, .c) { height: calc(50rpx - var(--abc, 100rpx)); }".to_string();
    let result = convert_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_where_selector() {
    let input = ".a:where(.b, .c) { color: green; }".to_string();
    let result = convert_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_has_selector() {
    let input = ".a:has(.b) { color: purple; }".to_string();
    let result = convert_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_star_selector() {
    let input = "* { color: black; } .a * {height: 100px;}".to_string();
    let result = convert_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_host_selector() {
    let input = "web-view :host { color: black; }".to_string();
    let result = convert_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn split_host_selector() {
    let input = ":host, .body { height: 20rpx; }".to_string();
    let result = convert_css(input.to_string());
    let result_unwrapped = result.unwrap();
    assert_snapshot!(result_unwrapped.css);
    assert_snapshot!(result_unwrapped.host_css.unwrap_or_default());
  }

  #[test]
  fn test_import() {
    let input = "@import url('./a.css');".to_string();
    let result = convert_css(input.to_string());
    assert_snapshot!(result.unwrap().css);
  }

  #[test]
  fn test_remove_single_host() {
    let input = ":host { color: black; }".to_string();
    let result = convert_css(input.to_string());
    let result_unwrapped = result.unwrap();
    assert!(result_unwrapped.css.is_empty());
    assert_snapshot!(result_unwrapped.host_css.unwrap_or_default());
  }

  #[test]
  fn test_remove_multi_host() {
    let input = ":host { color: black; width: 100rpx } :host { height: 20rpx; }".to_string();
    let result = convert_css(input.to_string());
    let result_unwrapped = result.unwrap();
    assert_snapshot!(result_unwrapped.css);
    assert_snapshot!(result_unwrapped.host_css.unwrap_or_default());
  }

  #[test]
  fn test_webkit_any_host() {
    let input = indoc! {r#"
      :-webkit-any(.h5-article) :-webkit-any(.h5-article) :-webkit-any(.h5-article) .sf-h5-h1 {
        -webkit-margin-before: 1.33em;
        -webkit-margin-after: 1.33em;
        margin-top: 1.33em;
        margin-bottom: 1.33em;
        font-size: 1em;
      }
    "#}
    .to_string();
    let result = convert_css(input.to_string());
    let result_unwrapped = result.unwrap();
    assert_snapshot!(result_unwrapped.css);
    assert_snapshot!(result_unwrapped.host_css.unwrap_or_default());
  }

  #[test]
  fn test_any_host() {
    let input = indoc! {r#"
      :any(.h5-article) :any(.h5-article) :any(.h5-article) .sf-h5-h1 {
        -webkit-margin-before: 1.33em;
        -webkit-margin-after: 1.33em;
        margin-top: 1.33em;
        margin-bottom: 1.33em;
        font-size: 1em;
      }
    "#}
    .to_string();
    let result = convert_css(input.to_string());
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

    let result = convert_css(input.to_string());
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
    let result = convert_css(input);
    match result {
      Ok(_) => panic!("Expected an error, but got Ok"),
      Err(e) => {
        println!("Error: {}", e);
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
    let result = convert_css(input);
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
    let result = convert_css(input);
    match result {
      Ok(_) => panic!("Expected an error, but got Ok"),
      Err(e) => {
        println!("Error: {}", e);
        assert_snapshot!(e.to_string());
      }
    }
  }
}
