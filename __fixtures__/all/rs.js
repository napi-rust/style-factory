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
    '.',
    prefix,
    'foo{color:red;height:',
    rpx(100),
    'px;-webkit-backdrop-filter:blur(2px);backdrop-filter:blur(2px)}#id,.',
    prefix,
    'class{color:red}.',
    prefix,
    'foo{color:green}.',
    prefix,
    'a .',
    prefix,
    'b,.',
    prefix,
    'a>.',
    prefix,
    'b,.',
    prefix,
    'a~.',
    prefix,
    'b,.',
    prefix,
    'a+.',
    prefix,
    'b,.',
    prefix,
    'a:after{color:red}[theme=light]{--main-bg-color:#fefefe;--main-text-color:#363636}[theme=dark]{--main-bg-color:#2f3a44;--main-text-color:#c5c5c5}',
    '',
  ].join('');
  var hostStyleText = ["[is='", host, "']{color:red}", ''].join('');
  if (options.hostStyle) {
    options.hostStyle(hostStyleText);
  } else {
    css = hostStyleText + css;
  }
  return css;
}
