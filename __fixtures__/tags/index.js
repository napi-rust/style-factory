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
    '[meta\\:tag=view]',
    ' + ',
    '[meta\\:tag=button]',
    '{color:#363636}',
    '[meta\\:tag=web-view]',
    '{color:#363636}',
    '[meta\\:tag=button]',
    '{color:#363636}',
  ].join('');

  return css;
}
