import { describe, it, expect } from 'vitest';

import { styleFactory } from '../index.js';

describe('styleFactory', () => {
  it('should return a string', () => {
    const css = styleFactory(`body { color: #ff0000; }`);
    expect(css).toMatchInlineSnapshot(`"[meta\\:tag=body]{color:red}"`);
  });

  it('keyframe case', () => {
    const css = styleFactory(`@keyframes mymove { from { top: 0px; } to { top: 200px; } }`);
    expect(css).toMatchInlineSnapshot(`"@keyframes mymove{0%{top:0}to{top:200px}}"`);
  });

  it('throw error', () => {
    expect(() => {
      styleFactory(`.a color: #ff0000; }`, { throwOnError: true });
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Transform error: Parse error: Unexpected end of input at :0:21]`);
  });
});
