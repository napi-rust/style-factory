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
    'ty-scroll-view-dot:first-child,.',
    prefix,
    'ty-scroll-view-loading-white .',
    prefix,
    'ty-scroll-view-dot:first-child{animation:.9s infinite ty-scroll-view-keyframes}.',
    prefix,
    'ty-scroll-view-loading-black .',
    prefix,
    'ty-scroll-view-dot:nth-child(2),.',
    prefix,
    'ty-scroll-view-loading-white .',
    prefix,
    'ty-scroll-view-dot:nth-child(2){animation:.9s .3s infinite ty-scroll-view-keyframes}.',
    prefix,
    'ty-scroll-view-loading-black .',
    prefix,
    'ty-scroll-view-dot:nth-child(3),.',
    prefix,
    'ty-scroll-view-loading-white .',
    prefix,
    'ty-scroll-view-dot:nth-child(3){animation:.9s .6s infinite ty-scroll-view-keyframes}',
    '',
  ].join('');

  return css;
}
