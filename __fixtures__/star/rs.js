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
    'unsupport-star{padding:0}[meta\\:tag=page],.',
    prefix,
    'abc unsupport-star{color:red}',
    '',
  ].join('');

  return css;
}
