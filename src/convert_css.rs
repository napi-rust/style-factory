use lightningcss::{
  properties::custom::{Token, TokenList, TokenOrValue},
  rules::{unknown::UnknownAtRule, CssRule},
  selector::{Component, Selector, SelectorList},
  stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
  targets::{Browsers, Targets},
  traits::ToCss,
  values::{ident::Ident, string::CSSString},
  visitor::{Visit, VisitTypes, Visitor},
};

use parcel_selectors::{
  attr::{AttrSelectorOperator, ParsedCaseSensitivity},
  parser::LocalName,
};
use smallvec::SmallVec;
use std::error::Error;
use std::string::String;

// Define constants for magic strings
const PREFIX: &str = "__PREFIX__";
const HOST: &str = "__HOST__";
const RPX_FUNC: &str = "__RPX__";
const IMPORT_STYLE: &str = "import-style";
const META_TAG: &str = "meta:tag";
const UNSUPPORTED_STAR: &str = "unsupported-star";
const UNSUPPORTED_WEB_VIEW: &str = "unsupported-web-view";

fn get_minify_targets() -> Targets {
  Targets::from(Browsers {
    safari: Some(13 << 16),
    chrome: Some(55 << 16),
    ..Browsers::default()
  })
}

fn get_printer_options<'a>() -> PrinterOptions<'a> {
  PrinterOptions {
    minify: true,
    targets: get_minify_targets(),
    ..PrinterOptions::default()
  }
}

struct FactoryVisitor {
  types: VisitTypes,
  host_css_vec: Vec<String>,
}

impl FactoryVisitor {
  fn replace_rpx_token(&self, token: &mut Token) {
    if let Token::Dimension {
      ref mut value,
      unit,
      ..
    } = token
    {
      if unit == &"rpx" {
        // 把当前 token 替换成  RPX_FUNC(value) 的形式
        *token = Token::String(format!("{}({})", RPX_FUNC, value).into());
      }
    }
  }

  fn is_host_selector(&self, selector: &Selector) -> bool {
    if selector.iter().selector_length() != 1 {
      return false;
    }
    // 判断是否是 :host 选择器
    selector.iter().any(|component| {
      if let Component::AttributeInNoNamespace {
        local_name,
        operator,
        value,
        ..
      } = component
      {
        return local_name == &Ident::from("is")
          && operator == &AttrSelectorOperator::Equal
          && value == &CSSString::from(HOST);
      }
      false
    })
  }

  fn has_single_selector(&self, selectors: &SelectorList) -> bool {
    // 判断列表里是否有单个 :host 选择器
    selectors.0.iter().any(|selector| {
      return self.is_host_selector(selector);
    })
  }

  fn remove_single_selector<'i>(&self, selectors: &SelectorList<'i>) -> SelectorList<'i> {
    // 移除列表里的单个 :host 选择器
    SelectorList::new(
      selectors
        .0
        .iter()
        .cloned()
        .filter(|selector| !self.is_host_selector(selector))
        .collect(),
    )
  }

  fn create_host_selector<'s>(&self) -> Selector<'s> {
    Selector::from(vec![self.create_host_component()])
  }

  fn create_host_component<'c>(&self) -> Component<'c> {
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
      CssRule::Import(ref import_rule) => {
        // @import url('./a.css'); => @import-style ("./a.css")
        let new_rule = CssRule::Unknown(UnknownAtRule {
          name: IMPORT_STYLE.into(),
          prelude: TokenList(vec![
            TokenOrValue::Token(Token::ParenthesisBlock),
            TokenOrValue::Token(Token::String(import_rule.url.to_string().into())),
            TokenOrValue::Token(Token::CloseParenthesis),
          ]),
          block: None,
          loc: import_rule.loc,
        });
        *rule = new_rule;
      }
      _ => {
        rule.visit_children(self)?;
      }
    }

    // rule_exit 时，处理一些特殊的选择器
    if let CssRule::Style(style) = rule {
      let selectors: &mut SelectorList<'i> = &mut style.selectors;
      let has_single_host = self.has_single_selector(selectors);

      if has_single_host {
        let cloned_selectors = selectors.clone();
        let omit_single_selectors: SelectorList = self.remove_single_selector(&cloned_selectors);

        // 移除后，如果没有选择器了，则将当前 rule 设置为 Ignored
        // 并将 host_css 添加到 host_css_vec 中
        if omit_single_selectors.0.len() == 0 {
          let cloned_rule = rule.clone();
          let host_css = cloned_rule.to_css_string(get_printer_options()).unwrap();

          self.host_css_vec.push(host_css);
          *rule = CssRule::Ignored;
        } else {
          // 复制原来选择器的样式, 生成一个新的 rule
          style.selectors = omit_single_selectors;
          let mut clone_style = style.clone();
          let mut single_selectors = SelectorList::new(SmallVec::new());
          single_selectors.0.push(self.create_host_selector());
          clone_style.selectors = single_selectors;

          self
            .host_css_vec
            .push(clone_style.to_css_string(get_printer_options()).unwrap());
        }
      }
    }
    Ok(())
  }

  fn visit_selector(&mut self, selector: &mut Selector<'i>) -> Result<(), Self::Error> {
    // 修改 selector 的样式名, 添加一个 PREFIX 前缀
    if self.types.contains(VisitTypes::SELECTORS) {
      for component in &mut selector.iter_mut_raw_match_order() {
        match component {
          // 将类名替换成 PREFIX 类名
          Component::Class(class) => {
            *class = format!("{}{}", PREFIX, class).into();
          }

          // 处理 * 选择器 * => unsupported-star
          Component::ExplicitUniversalType => {
            *component = Component::LocalName(LocalName {
              name: UNSUPPORTED_STAR.into(),
              lower_name: UNSUPPORTED_STAR.into(),
            });
          }

          // 处理 :host 选择器 :host => [is=HOST]
          Component::Host(_host) => {
            *component = self.create_host_component();
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
          self.replace_rpx_token(token);
        }
      }
      TokenOrValue::Function(function) => {
        function.arguments.visit_children(self)?;
      }
      TokenOrValue::Var(ref mut var) => {
        var.fallback.visit_children(self)?;
      }
      _ => {
        // 其他 token 不做处理
        // println!("token: {:?}", token);
      }
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct TransformReturn {
  pub css: String,
  pub host_css: Option<String>,
}

pub fn convert_css(css: String) -> Result<TransformReturn, Box<dyn Error>> {
  if css.is_empty() {
    return Ok(TransformReturn {
      css: "".to_string(),
      host_css: None,
    });
  }

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
    .minify(MinifyOptions {
      targets: get_minify_targets(),
      ..MinifyOptions::default()
    })
    .map_err(|e| format!("Minify error: {}", e))?;

  // 3. 生成 CSS（处理序列化错误）
  let res = stylesheet
    .to_css(get_printer_options())
    .map_err(|e| format!("Serialize error: {}", e))?;

  let host_css_string = process_host_css(&visitor.host_css_vec)?;

  // 4. 返回成功结果
  Ok(TransformReturn {
    css: res.code,
    host_css: host_css_string,
  })
}

// Extract host CSS processing into a separate function
fn process_host_css(host_css_vec: &[String]) -> Result<Option<String>, Box<dyn Error>> {
  if host_css_vec.is_empty() {
    return Ok(None);
  }

  let host_css_css = host_css_vec.join("\n");

  let mut host_stylesheet = StyleSheet::parse(&host_css_css, ParserOptions::default())
    .map_err(|e| format!("Parse host error: {}", e))?;

  host_stylesheet
    .minify(MinifyOptions::default())
    .map_err(|e| format!("Minify host error: {}", e))?;

  let host_css_css = host_stylesheet
    .to_css(get_printer_options())
    .map_err(|e| format!("Serialize host error: {}", e))?;

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
    assert_eq!(result_unwrapped.css.trim(), "");
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
