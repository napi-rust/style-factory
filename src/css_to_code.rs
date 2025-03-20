use lazy_regex::{regex, Regex};
use md5::{Digest, Md5};
use std::collections::HashMap;

// ---- Static Regex Patterns ----
lazy_static::lazy_static! {
  static ref PREFIX_REGEX: &'static Regex = regex!("__PREFIX__");
  static ref HOST_REGEX: &'static Regex = regex!("__HOST__");
  static ref RPX_REGEX: &'static Regex = regex!(r#"\\"__RPX__\(([^)]+)\)\\""#);
  static ref IMPORT_REGEX: &'static Regex = regex!(r#"@import-style \("([^"]+)"\);"#);
}

// ---- Core Logic ----
pub struct Css2CodeOptions<'a> {
  pub css: &'a str,
  pub host_css: Option<&'a str>,
}

pub fn css_to_code(options: Css2CodeOptions<'_>) -> String {
  let mut imports = HashMap::new();

  // Process main CSS
  let css_code = process_text(options.css, Some(&mut imports));

  // Process Host CSS
  let host_css_code = if let Some(hc) = options.host_css {
    process_text(hc, None)
  } else {
    String::new()
  };

  generate_output(&css_code, &host_css_code, &imports)
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

fn process_text(text: &str, mut imports: Option<&mut HashMap<String, String>>) -> String {
  let escaped = json_escape(text);

  let replaced = PREFIX_REGEX
    .replace_all(&escaped, |_: &regex::Captures| "\" + prefix + \"")
    .to_string();
  let replaced = HOST_REGEX
    .replace_all(&replaced, |_: &regex::Captures| "\" + host + \"")
    .to_string();
  let replaced = RPX_REGEX.replace_all(&replaced, |caps: &regex::Captures| {
    format!(r#"" + rpx({}) + "px"#, &caps[1])
  });

  let replaced = if let Some(imports) = &mut imports {
    IMPORT_REGEX
      .replace_all(&replaced, |caps: &regex::Captures| {
        let url = caps[1].to_string();
        let fn_name = format!("I_{}", md5_hash(url.as_str()));
        imports.insert(url.to_string(), fn_name.clone());
        format!(r#"" + {}(options) + ""#, fn_name)
      })
      .to_string()
  } else {
    replaced.to_string()
  };

  replaced
}

fn generate_output(
  css_code: &str,
  host_css_code: &str,
  imports: &HashMap<String, String>,
) -> String {
  let host_code = if !host_css_code.is_empty() {
    format!(
      r#"var hostStyleText = "{host_css_code}";
  if (options.hostStyle) {{
      options.hostStyle(hostStyleText);
  }} else {{
      css = hostStyleText + css;
  }}"#
    )
  } else {
    String::new()
  };

  let import_code = imports
    .iter()
    .map(|(url, fn_name)| format!("import {fn_name} from '{url}';"))
    .collect::<Vec<_>>()
    .join("\n");

  format!(
    r#"{import_code}
export default function styleFactory(options) {{
  var prefix = options.prefix || '';
  var tag = options.tag || (tag => tag);
  var rpx = options.rpx;
  var host = options.host || 'host-placeholder';
  var css = "{css_code}";
  {host_code}
  return css;
}}"#
  )
  .trim()
  .replace("{css_code}", css_code)
  .replace("{host_css_code}", host_css_code)
}

// ---- 测试用例 ----
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tow_prefix() {
    let input = r#".__PREFIX__a{width:"__RPX__(100)"}.__PREFIX__b{height:"__RPX__(50)"}"#;
    let options = Css2CodeOptions {
      css: input,
      host_css: None,
    };
    let output = css_to_code(options);
    let expected = r#"export default function styleFactory(options) {
  var prefix = options.prefix || '';
  var tag = options.tag || (tag => tag);
  var rpx = options.rpx;
  var host = options.host || 'host-placeholder';
  var css = "." + prefix + "a{width:" + rpx(100) + "px}." + prefix + "b{height:" + rpx(50) + "px}";
  
  return css;
}"#;
    assert_eq!(output.trim(), expected.trim());
  }
}
