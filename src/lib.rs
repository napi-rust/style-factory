#![deny(clippy::all)]

// 引入 napi 派生宏
#[macro_use]
extern crate napi_derive;

// 模块导入
mod compile_css;
mod convert_css;
mod css_to_code; // CSS 转换模块
mod options;
mod style_factory;
// 样式工厂模块 // Token 转换模块

// 导出样式工厂函数
pub use compile_css::compile_css;
pub use style_factory::style_factory;
