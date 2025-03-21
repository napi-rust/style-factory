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
    '@font-face{font-family:iconfont;src:url(/components/foo/iconfont.woff2?t=1642945972391)format("woff2")}',
    '',
  ].join('');

  return css;
}
