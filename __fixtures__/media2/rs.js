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
    '@supports (display:flex){@media screen and (min-width:900px){.',
    prefix,
    'article{display:flex}}}',
    '',
  ].join('');

  return css;
}
