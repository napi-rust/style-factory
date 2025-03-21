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
    '@keyframes anim-show{to{opacity:1}}@keyframes anim-hide{to{opacity:0}}',
    '',
  ].join('');

  return css;
}
