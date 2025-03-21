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
    'h1{height:calc(',
    rpx(50) + 'px',
    ' - ',
    rpx(20) + 'px',
    ')}.',
    prefix,
    'h2{width:calc(',
    rpx(50) + 'px',
    ' - ',
    rpx(-20) + 'px',
    ')}',
  ].join('');

  return css;
}
