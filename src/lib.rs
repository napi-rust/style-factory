#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod css_to_code;
mod style_factory;
mod transform_css;

pub use style_factory::style_factory;
