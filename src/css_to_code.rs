pub fn css_to_code(css_text: String) -> Result<String, String> {
  /*
   * 将 css_text 以 转造成一个 js 的 函数文本
   * @example
   * function style_factory(css) {
   *  return css_text
   * }
   */
  let css_text = format!(
    r#"function style_factory(optoins) {{
  var prefix = options.prefix || '';
  return "{}";
}}"#,
    css_text,
  );

  return Ok(css_text);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_css_to_code() {
    let css_text = r#".a { color: red }"#.to_string();
    let res = css_to_code(css_text.clone());
    let expected = r#"function style_factory(optoins) {
  var prefix = options.prefix || '';
  return ".a { color: red }";
}"#;
    assert_eq!(res.unwrap(), expected);
  }
}
