#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::Undefined;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet, MinifyOptions};

#[napi]
pub fn style_factory(css: String) -> Undefined {
    // 1. 解析CSS
    let mut stylesheet = StyleSheet::parse(&css, ParserOptions::default()).unwrap();
    
    // 2. 执行压缩（原地修改）
    stylesheet.minify(MinifyOptions::default()).unwrap();
    
    // 3. 生成压缩后的CSS代码
    let res = stylesheet.to_css(PrinterOptions::default()).unwrap();
    
    println!("测试输出：{:?}", res.code);
    () // 直接返回单元结构体
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_factory() {
        assert!(std::panic::catch_unwind(|| {
            style_factory("body { color: #ffffff;     }".to_string());
        }).is_ok());
    }
}