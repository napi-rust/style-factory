use indoc::formatdoc;
use lazy_regex::regex_replace_all;
use md5::{Digest, Md5};
use pipe_trait::Pipe;
use std::collections::HashMap;

// ---- Core Logic ----
#[derive(Debug, Clone)]
pub struct Css2CodeOptions<'css_string> {
  pub css: &'css_string str,
  pub host_css: Option<&'css_string str>,
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
  json_escape(text)
    .pipe(|escaped| regex_replace_all!("__PREFIX__", &escaped, r#"" + prefix + ""#).into_owned())
    .pipe(|replaced| regex_replace_all!("__HOST__", &replaced, r#"" + host + ""#).into_owned())
    .pipe(|replaced| {
      regex_replace_all!(r#"\\"__RPX__\(([^)]+)\)\\""#, &replaced, |_, unit| {
        format!(r#"" + rpx({}) + "px"#, unit)
      })
      .into_owned()
    })
    .pipe(|result| match &mut imports {
      Some(imports_map) => {
        regex_replace_all!(r#"@import-style \("([^"]+)"\);"#, &result, |_, url| {
          let fn_name = format!("I_{}", md5_hash(url));
          imports_map.insert(url.to_string(), fn_name.clone());
          format!(r#"" + {}(options) + ""#, fn_name)
        })
        .into_owned()
      }
      None => result,
    })
}

fn generate_output(
  css_code: &str,
  host_css_code: &str,
  imports: &HashMap<String, String>,
) -> String {
  let host_code = if !host_css_code.is_empty() {
    formatdoc! {r#"
      var hostStyleText = "{host_css_code}";
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
    .map(|(url, fn_name)| format!("import {fn_name} from '{url}';"))
    .collect::<Vec<_>>()
    .join("\n");

  formatdoc! {r#"
    {import_code}
    export default function styleFactory(options) {{
      var prefix = options.prefix || '';
      var tag = options.tag || (tag => tag);
      var rpx = options.rpx;
      var host = options.host || 'host-placeholder';
      var css = "{css_code}";
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
  fn test_tow_prefix() {
    let input = r#".__PREFIX__a{width:"__RPX__(100)"}.__PREFIX__b{height:"__RPX__(50)"}"#;
    let options = Css2CodeOptions {
      css: input,
      host_css: None,
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
}
