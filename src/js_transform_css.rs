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

impl Default for JSTransformCssOptions {
  fn default() -> Self {
    JSTransformCssOptions {
      input: String::new(),
      minify: None,
    }
  }
}

#[napi(js_name = "transformCSS")]
pub fn js_transform_css(
  option: Option<JSTransformCssOptions>,
) -> Result<JSTransformCSSResult, napi::Error> {
  let option = option.unwrap_or_default();
  let minify = option.minify.unwrap_or(false);
  let input = option.input;

  let result = transform_css(TransformCssOptions {
    input: &input,
    minify,
  })
  .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e.to_string()))?;

  Ok(JSTransformCSSResult { code: result })
}
