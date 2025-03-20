use md5::{Digest, Md5};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub enum CssTransformError {
  RegexError,
}

impl std::fmt::Display for CssTransformError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "CSS Transform Error")
  }
}

impl Error for CssTransformError {}

// ---- 正则替换器 ----
struct Replacers {
  prefix: Regex,
  host: Regex,
  rpx: Regex,
  import: Regex,
}

impl Replacers {
  fn new() -> Result<Self, Box<dyn Error>> {
    Ok(Self {
      prefix: Regex::new(r"__PREFIX__")?,
      host: Regex::new(r"__HOST__")?,
      rpx: Regex::new(r#"\\"__RPX__\(([^)]+)\)\\""#)?,
      import: Regex::new(r#"@import-style \("([^"]+)"\);"#)?,
    })
  }
}

static REPLACERS: Lazy<Result<Replacers, CssTransformError>> =
  Lazy::new(|| Replacers::new().map_err(|_| CssTransformError::RegexError));

// ---- 核心逻辑 ----
pub struct Css2CodeOptions<'a> {
  pub css: &'a str,
  pub host_css: Option<&'a str>,
}

pub fn css_to_code(options: Css2CodeOptions<'_>) -> Result<String, CssTransformError> {
  let replacers = REPLACERS
    .as_ref()
    .map_err(|_| CssTransformError::RegexError)?;
  let mut imports = HashMap::new();

  // 处理主 CSS
  let css_code = process_text(options.css, replacers, Some(&mut imports))?;

  // 处理 Host CSS
  let host_css_code = if let Some(hc) = options.host_css {
    process_text(hc, replacers, None)?
  } else {
    String::new()
  };

  Ok(generate_output(&css_code, &host_css_code, &imports))
}

// ---- 私有辅助函数 ----
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

fn process_text(
  text: &str,
  replacers: &Replacers,
  mut imports: Option<&mut HashMap<String, String>>,
) -> Result<String, CssTransformError> {
  let escaped = json_escape(text);

  let replaced = replacers.prefix.replace_all(&escaped, r#"" + prefix + ""#);
  let replaced = replacers.host.replace_all(&replaced, r#"" + host + ""#);
  let replaced = replacers.rpx.replace_all(&replaced, |caps: &Captures| {
    format!(r#"" + rpx({}) + "px"#, &caps[1])
  });

  let replaced = if let Some(imports) = &mut imports {
    replacers.import.replace_all(&replaced, |caps: &Captures| {
      let url = caps.get(1).unwrap().as_str();
      let fn_name = format!("I_{}", md5_hash(url));
      imports.insert(url.to_string(), fn_name.clone());
      format!(r#"" + {}(options) + ""#, fn_name)
    })
  } else {
    replaced
  };

  Ok(replaced.into_owned())
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
    let output = css_to_code(options).unwrap();
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
