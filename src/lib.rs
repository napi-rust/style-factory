#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::Undefined;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};

#[napi]
pub fn style_factory(css: String) -> Undefined {
    let stylesheet = StyleSheet::parse(&css, ParserOptions::default()).unwrap();
    let res = stylesheet
        .to_css(PrinterOptions::default())
        .unwrap();
    println!("{}", res.code);
    ()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_factory() {
        assert!(std::panic::catch_unwind(|| {
            style_factory("body { color: red; }".to_string());
        })
        .is_ok());
    }
}