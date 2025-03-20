export default function styleFactory(options) {
  var prefix = options.prefix || '';
  var tag =
    options.tag ||
    function (tag) {
      return tag;
    };
  var rpx = options.rpx;
  var host = options.host || 'host-placeholder';
  var css = [
    '/*注释*/.',
    prefix,
    'foo{color:red;height:',
    rpx(100) + 'px',
    '}#',
    'id, .',
    prefix,
    'class{color:red}.',
    prefix,
    'foo{color:green}.',
    prefix,
    'a .',
    prefix,
    'b{color:red}.',
    prefix,
    'a > .',
    prefix,
    'b{color:red}.',
    prefix,
    'a ~ .',
    prefix,
    'b{color:red}.',
    prefix,
    'a + .',
    prefix,
    'b{color:red}.',
    prefix,
    'a:after{color:red}:root{--main-bg-color:#fefefe;/*浅色背景*/--main-text-color:#363636;/*深色文字*/}[theme="dark"]:root{--main-bg-color:#2f3a44;/*深色背景*/--main-text-color:#c5c5c5;/*浅色文字*/}',
  ].join('');
  var hostStyleText = ["[is='" + host + "']", '{color:red}'].join('');
  if (options.hostStyle) {
    options.hostStyle(hostStyleText);
  } else {
    css = hostStyleText + css;
  }
  return css;
}
