use std::path::Path;

/*
 * Get the basename of a path
 * @param path: &str
 * @param with_extension: bool
 */
#[allow(dead_code)]
pub fn get_basename<P: AsRef<Path>>(path: P, with_extension: bool) -> Option<String> {
  let path = path.as_ref();
  if with_extension {
    path.file_name()?.to_str().map(String::from)
  } else {
    path.file_stem()?.to_str().map(String::from)
  }
}
