import { describe, it, expect } from 'vitest';

import { styleFactory } from '../index.js';

describe('styleFactory', () => {
  it('should return rpx', () => {
    const css = styleFactory(`body { height: 100rpx; }`);
    expect(css).toMatchInlineSnapshot(`
      "export default function styleFactory(options) {
        var prefix = options.prefix || '';
        var tag = options.tag || (tag => tag);
        var rpx = options.rpx;
        var host = options.host || 'host-placeholder';
        var css = "[meta\\\\:tag=body]{height:" + rpx(100) + "px}";
        
        return css;
      }"
    `);
  });

  it('keyframe case', () => {
    const css = styleFactory(`@keyframes mymove { from { top: 0px; } to { top: 200px; } }`);
    expect(css).toMatchInlineSnapshot(`
      "export default function styleFactory(options) {
        var prefix = options.prefix || '';
        var tag = options.tag || (tag => tag);
        var rpx = options.rpx;
        var host = options.host || 'host-placeholder';
        var css = "@keyframes mymove{0%{top:0}to{top:200px}}";
        
        return css;
      }"
    `);
  });

  it('import css', () => {
    const css = styleFactory(`@import url('./style.css');`);
    expect(css).toMatchInlineSnapshot(`
      "import I_1568b90116e4f2a5d70b882f42df82dd from "./style.css";
      export default function styleFactory(options) {
        var prefix = options.prefix || '';
        var tag = options.tag || (tag => tag);
        var rpx = options.rpx;
        var host = options.host || 'host-placeholder';
        var css = "" + I_1568b90116e4f2a5d70b882f42df82dd(options) + "";
        
        return css;
      }"
    `);
  })

  it('throw error', () => {
    expect(() => {
      styleFactory(`.a color: #ff0000; }`, { throwOnError: true });
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Transform error: Parse error: Unexpected end of input at :0:21]`);
  });
});
