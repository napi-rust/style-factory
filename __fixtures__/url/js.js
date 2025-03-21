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
    'h1:not(.',
    prefix,
    'foo){background-image:url(./abc.sbc)}',
  ].join('');

  return css;
}
