import I_f3cea1431258782941feb3c71a992799 from './a.css';
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
    I_f3cea1431258782941feb3c71a992799(options),
    '.',
    prefix,
    'foo{color:red;height:',
    rpx(100) + 'px',
    '}',
    '[meta\\:tag=view]',
    '{color:green}',
  ].join('');

  return css;
}
