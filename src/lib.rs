#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet, MinifyOptions};

#[napi]
pub fn style_factory(css: String) -> String {
    // 1. 解析CSS
    let mut stylesheet = StyleSheet::parse(&css, ParserOptions::default()).unwrap();
    
    // 2. 执行压缩（原地修改）
    stylesheet.minify(MinifyOptions::default()).unwrap();
    
    // 3. 生成压缩后的CSS代码
    let printer_options = PrinterOptions {
      minify: true,
      ..Default::default()
  };
    let res = stylesheet.to_css(printer_options).unwrap();
    
    // 4.返回压缩后的CSS代码
    (res.code).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_factory_basic() {
        let input = "body { color: #ffffff;     }".to_string();
        let expected = "body{color:#fff}";
        assert_eq!(style_factory(input), expected);
    }
}