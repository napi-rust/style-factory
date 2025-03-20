export default function styleFactory(options) {
  var prefix = options.prefix || '';
  var tag =
    options.tag ||
    function (tag) {
      return tag;
    };
  var rpx = options.rpx;
  var host = options.host || 'host-placeholder';
  var css = ['@keyframes slidein{from{transform:translateX(0%)}to{transform:translateX(100%)}}'].join('');

  return css;
}
