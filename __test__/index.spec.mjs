import { describe, it, expect } from 'vitest';

import { styleFactory } from '../index.js';

describe('styleFactory', () => {
  it('should return a string', () => {
    const css = styleFactory(`body { color: #ff0000; }`);
    expect(css).toMatchInlineSnapshot(`"[meta\\:tag=body]{color:red}"`);
  });
});
