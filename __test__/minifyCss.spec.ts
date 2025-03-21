import { describe, expect, it } from 'vitest';
import { minifyCss } from './minifyCss';

describe('minifyCss()', () => {
  it('should minify css', async () => {
    const css = await minifyCss(`
      [is='component-666'] {
        color: red;
        width: 50px;
      }
      [is='component-666'] {
        font-size: 10px;
      }
      .a {
        -webkit-align-items: center;
        -ms-flex-align: center;
        align-items: center;
        -webkit-box-align: center;
        
      }
      @-webkit-keyframes anim-show {
        to {
          opacity: 1;
        }
      }
      @keyframes anim-show {
        to {
          opacity: 1;
        }
      }
      @-webkit-keyframes anim-hide {
        to {
          opacity: 0;
        }
      }
      @keyframes anim-hide {
        to {
          opacity: 0;
        }
      }
          `);
    expect(css).toMatchInlineSnapshot(
      `"[is=component-666]{color:red;width:50px;font-size:10px}.a{-webkit-box-align:center;align-items:center}@keyframes anim-show{to{opacity:1}}@keyframes anim-hide{to{opacity:0}}"`,
    );
  });
  it('should merge prop', async () => {
    const css = await minifyCss(`
      .b {
        -webkit-box-align: center;
        -webkit-box-pack: center;
        margin-top: 1em;
        margin-bottom: 1em;
        margin-left: 0;
        margin-right: 0;
      }
    `);
    expect(css).toMatchInlineSnapshot(`".b{-webkit-box-align:center;-webkit-box-pack:center;margin:1em 0}"`);
  });
});
