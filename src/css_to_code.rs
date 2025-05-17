use std::collections::HashMap;

use indoc::formatdoc;
use lazy_regex::{lazy_regex, regex::Captures, Regex};
use md5::{Digest, Md5};
use once_cell::sync::Lazy;
use serde_json;

#[derive(Debug, Clone)]
pub struct Css2CodeOptions<'a> {
  pub css: &'a str,
  pub host_css: Option<&'a str>,
}

pub fn css_to_code(options: Css2CodeOptions<'_>) -> String {
  let mut imports = HashMap::new();
  let css_code = process_text(options.css, &mut imports);
  let host_css_code = options
    .host_css
    .map(|hc| process_text(hc, &mut HashMap::new()))
    .unwrap_or_default();
  generate_output(&css_code, &host_css_code, &imports)
}

/// 处理文本，替换自定义标记和处理 import
fn process_text(text: &str, imports: &mut HashMap<String, String>) -> String {
  let escaped = json_escape(text);

  let with_prefix = PREFIX_REGEX
    .replace_all(&escaped, r#"" , prefix , ""#)
    .into_owned();

  let with_host = HOST_REGEX
    .replace_all(&with_prefix, r#"'" , host , "'"#)
    .into_owned();

  let with_rpx = RPX_REGEX
    .replace_all(&with_host, |caps: &Captures<'_>| {
      format!(r#"" , rpx({}) , "px"#, &caps[1])
    })
    .into_owned();

  let with_imports = IMPORT_REGEX
    .replace_all(&with_rpx, |caps: &Captures<'_>| {
      let url = &caps[1];
      let fn_name = format!("I_{}", md5_hash(url));
      imports.insert(url.to_string(), fn_name.clone());
      format!(r#"" , {}(options) , ""#, fn_name)
    })
    .into_owned();

  with_imports
}

fn md5_hash(input: &str) -> String {
  let mut hasher = Md5::new();
  hasher.update(input.as_bytes());
  format!("{:x}", hasher.finalize())
}

fn json_escape(s: &str) -> String {
  serde_json::to_string(s)
    .unwrap()
    .trim_matches('"')
    .to_string()
}

static PREFIX_REGEX: Lazy<Regex> = lazy_regex!(r"__PREFIX__");
static HOST_REGEX: Lazy<Regex> = lazy_regex!(r"__HOST__");
static RPX_REGEX: Lazy<Regex> = lazy_regex!(r#"\\"__RPX__\(([^)]+)\)\\""#);
static IMPORT_REGEX: Lazy<Regex> = lazy_regex!(r#"\@import-style \(\\"([^\)]+)\\"\);"#);

/// 生成最终 JS 输出
fn generate_output(
  css_code: &str,
  host_css_code: &str,
  imports: &HashMap<String, String>,
) -> String {
  let host_code = if !host_css_code.is_empty() {
    formatdoc! {r#"
      var hostStyleText = ["{host_css_code}", ""].join("");
      if (options.hostStyle) {{
          options.hostStyle(hostStyleText);
      }} else {{
          css = hostStyleText + css;
      }}"#, host_css_code = host_css_code
    }
    .trim()
    .to_string()
  } else {
    String::new()
  };

  let import_code = imports
    .iter()
    .map(|(url, fn_name)| format!(r#"import {fn_name} from "{url}";"#))
    .collect::<Vec<_>>()
    .join("\n");

  formatdoc! {r#"
    {import_code}
    export default function styleFactory(options) {{
      var prefix = options.prefix || '';
      var tag = options.tag || function (tag) {{ return tag; }};
      var rpx = options.rpx;
      var host = options.host || 'host-placeholder';
      var css = ["{css_code}", ""].join("");
      {host_code}
      return css;
    }}
  "#, import_code = import_code, css_code = css_code, host_code = host_code
  }
  .trim()
  .to_string()
}

// ---- 测试用例 ----
#[cfg(test)]
mod tests {
  use super::*;
  use insta::assert_snapshot;

  #[test]
  fn test_keep_is_prefix() {
    let css = r#".__PREFIX__h5-blockquote:not(:is(:lang(ae),:lang(yi))){margin-left:"__RPX__(40)";margin-right:"__RPX__(40)"}"#;
    let options = Css2CodeOptions {
      css,
      host_css: None,
    };
    let output = css_to_code(options);

    assert_snapshot!(output.trim());
  }

  #[test]
  fn test_two_prefix() {
    let css = r#".__PREFIX__a{width:"__RPX__(100)"}.__PREFIX__b{height:"__RPX__(50)"}"#;
    let host_css = r#"[is=__HOST__]{color:#000;width:"__RPX__(100)";height:"__RPX__(20)"}"#;
    let options = Css2CodeOptions {
      css,
      host_css: Some(host_css),
    };
    let output = css_to_code(options);

    assert_snapshot!(output.trim());
  }

  #[test]
  fn test_host_css() {
    let input = r#"[is=__HOST__]{color:#fff}"#;
    let options = Css2CodeOptions {
      css: input,
      host_css: None,
    };
    let output = css_to_code(options);

    assert_snapshot!(output.trim());
  }

  #[test]
  fn test_import_style() {
    let input = r#"@import-style ("./a.css");"#;
    let options = Css2CodeOptions {
      css: input,
      host_css: None,
    };
    let output = css_to_code(options);

    assert_snapshot!(output.trim());
  }

  #[test]
  fn test_empty_css() {
    let options = Css2CodeOptions {
      css: "",
      host_css: None,
    };
    let output = css_to_code(options);
    assert_snapshot!(output.trim());
  }

  #[test]
  fn test_special_chars() {
    let css = r#".foo{content:"a\"b\\c\nd"}"#;
    let options = Css2CodeOptions {
      css,
      host_css: None,
    };
    let output = css_to_code(options);
    assert_snapshot!(output.trim());
  }
}
