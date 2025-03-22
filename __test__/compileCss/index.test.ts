import { describe, it, expect } from 'vitest';
import { compileCSS } from '../../index';
import path from 'node:path';

describe('compileCss', () => {
  const entry = path.resolve(__dirname, './a.css');
  it('should compile css', () => {
    const result = compileCSS(entry);
    expect(result.css).toMatchInlineSnapshot(
      `".c{color:#5c91f6;background-image:url(./img.png)}.b{color:#00f}.a{color:red}"`,
    );
    const basenames = result.dependencies.map((file) => path.basename(file));
    expect(basenames).toMatchInlineSnapshot(`
      [
        "a.css",
        "b.css",
        "c.css",
      ]
    `);
  });
});
