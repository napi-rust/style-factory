use crate::transform_css::{transform_css, TransformCssOptions};
use napi_derive::napi;

#[napi(object)]
pub struct JSTransformCSSResult {
  pub code: String,
}

#[napi(object)]
pub struct JSTransformCssOptions {
  pub input: String,
  pub minify: Option<bool>,
}

#[napi(js_name = "transformCSS")]
pub fn js_transform_css(option: Option<JSTransformCssOptions>) -> Result<JSTransformCSSResult, napi::Error> {
  let options = option.map_or_else(
    || TransformCssOptions {
      input: "".to_string(),
      minify: false,
    },
    |opt| TransformCssOptions {
      input: opt.input,
      minify: opt.minify.unwrap_or(false),
    },
  );

  // 调用 transform_css 函数
  match transform_css(options) {
    Ok(css) => Ok(JSTransformCSSResult { code: css }),
    Err(err) => Err(napi::Error::from_reason(err.to_string())),
  }
}
