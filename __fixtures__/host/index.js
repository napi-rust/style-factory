export default function styleFactory(options) {
  var prefix = options.prefix || '';
  var tag =
    options.tag ||
    function (tag) {
      return tag;
    };
  var rpx = options.rpx;
  var host = options.host || 'host-placeholder';
  var css = ['.', prefix, 'a ', "[is='" + host + "']", '{height:', rpx(100) + 'px', '}'].join('');
  var hostStyleText = [
    "[is='" + host + "']",
    '{color:red;width:',
    rpx(100) + 'px',
    '}',
    "[is='" + host + "']",
    '{font-size:10px}',
  ].join('');
  if (options.hostStyle) {
    options.hostStyle(hostStyleText);
  } else {
    css = hostStyleText + css;
  }
  return css;
}
