use lightningcss::printer::PrinterOptions;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions};
use lightningcss::targets::{Browsers, Targets};

pub fn get_targets<'t>() -> Targets {
  Targets::from(Browsers {
    safari: Some(13 << 16),
    chrome: Some(55 << 16),
    ..Browsers::default()
  })
}

pub fn get_printer_options<'a>() -> PrinterOptions<'a> {
  PrinterOptions {
    minify: true,
    targets: get_targets(),
    ..PrinterOptions::default()
  }
}

pub fn get_parser_options<'p, 'q>() -> ParserOptions<'p, 'q> {
  ParserOptions::default()
}

pub fn get_minify_options() -> MinifyOptions {
  MinifyOptions {
    targets: get_targets(),
    ..MinifyOptions::default()
  }
}
