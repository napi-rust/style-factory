#![deny(clippy::all)]

// 引入 napi 派生宏
#[macro_use]
extern crate napi_derive;

// 模块导入
mod css_to_code;    // CSS 转换模块
mod style_factory;  // 样式工厂模块
mod convert_token;  // Token 转换模块

// 导出样式工厂函数
pub use style_factory::style_factory;
