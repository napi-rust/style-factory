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
    '@-webkit-keyframes anim-show{100%{opacity:1}}@keyframes anim-show{100%{opacity:1}}@-webkit-keyframes anim-hide{100%{opacity:0}}@keyframes anim-hide{100%{opacity:0}}',
  ].join('');

  return css;
}
