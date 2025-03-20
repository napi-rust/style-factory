use lightningcss::stylesheet::PrinterOptions;
use lightningcss::targets::{Browsers, Targets};

pub fn get_printer_options<'a>() -> PrinterOptions<'a> {
  return PrinterOptions {
    minify: true,
    targets: Targets::from(Browsers {
      safari: Some(11),
      ios_saf: Some(11),
      android: Some(6),
      chrome: Some(55),
      ..Browsers::default()
    }),
    ..Default::default()
  };
}
