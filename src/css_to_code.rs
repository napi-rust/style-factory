use indoc::formatdoc;
use lazy_regex::{lazy_regex, regex::Captures, Regex};
use md5::{Digest, Md5};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// ---- Core Logic ----
#[derive(Debug, Clone)]
pub struct Css2CodeOptions<'css_string> {
  pub css: &'css_string str,
  pub host_css: Option<&'css_string str>,
}

pub fn css_to_code(options: Css2CodeOptions<'_>) -> String {
  let imports = Mutex::new(HashMap::new());

  // Process main CSS
  let css_code = process_text(options.css, Some(&imports));

  // Process Host CSS
  let host_css_code = if let Some(hc) = options.host_css {
    process_text(hc, None)
  } else {
    String::new()
  };

  generate_output(&css_code, &host_css_code, &imports.into_inner().unwrap())
}

// ---- Private Helper Functions ----
fn md5_hash(input: &str) -> String {
  let mut hasher = Md5::new();
  hasher.update(input.as_bytes());
  format!("{:x}", hasher.finalize())
}

fn json_escape(s: &str) -> String {
  s.replace('\\', r"\\")
    .replace('"', r#"\""#)
    .replace('\n', r"\n")
}

static PREFIX_REGEX: Lazy<Regex> = lazy_regex!(r"__PREFIX__");
static HOST_REGEX: Lazy<Regex> = lazy_regex!(r"__HOST__");
static RPX_REGEX: Lazy<Regex> = lazy_regex!(r#"\\"__RPX__\(([^)]+)\)\\""#);
static IMPORT_REGEX: Lazy<Regex> = lazy_regex!(r#"\@import-style \(\\"([^\)]+)\\"\);"#);

fn process_text(text: &str, imports: Option<&Mutex<HashMap<String, String>>>) -> String {
  let mut result = json_escape(text);

  result = PREFIX_REGEX
    .replace_all(&result, r#"" , prefix , ""#)
    .into_owned();
  result = HOST_REGEX
    .replace_all(&result, r#"" , host , ""#)
    .into_owned();
  result = RPX_REGEX
    .replace_all(&result, |caps: &Captures<'_>| {
      format!(r#"" , rpx({}) , "px"#, &caps[1])
    })
    .into_owned();

  if let Some(imports_map) = imports {
    result = IMPORT_REGEX
      .replace_all(&result, |caps: &Captures<'_>| {
        let url = caps[1].into();
        let fn_name = format!("I_{}", md5_hash(url));
        imports_map
          .lock()
          .unwrap()
          .insert(url.to_string(), fn_name.clone());
        format!(r#"" , {}(options) , ""#, fn_name)
      })
      .into_owned();
  }

  result
}

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
    .into()
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
  .into()
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
  fn test_tow_prefix() {
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
}
