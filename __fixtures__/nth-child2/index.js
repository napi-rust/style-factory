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
    'ty-scroll-view-loading-black .',
    prefix,
    'ty-scroll-view-dot:nth-child(1), .',
    prefix,
    'ty-scroll-view-loading-white .',
    prefix,
    'ty-scroll-view-dot:nth-child(1){animation:ty-scroll-view-keyframes 0.9s infinite}.',
    prefix,
    'ty-scroll-view-loading-black .',
    prefix,
    'ty-scroll-view-dot:nth-child(2), .',
    prefix,
    'ty-scroll-view-loading-white .',
    prefix,
    'ty-scroll-view-dot:nth-child(2){animation:ty-scroll-view-keyframes 0.9s 0.3s infinite}.',
    prefix,
    'ty-scroll-view-loading-black .',
    prefix,
    'ty-scroll-view-dot:nth-child(3), .',
    prefix,
    'ty-scroll-view-loading-white .',
    prefix,
    'ty-scroll-view-dot:nth-child(3){animation:ty-scroll-view-keyframes 0.9s 0.6s infinite}',
  ].join('');

  return css;
}
