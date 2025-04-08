import {transformCSS} from '../'

import {describe, expect, it} from 'vitest';


describe('transformCSS', () => {
  it('works correctly', () => {
    const result = transformCSS({
      input: `
      @import 'b.css';
      .c {
        color: green;
        backdrop-filter: blur(2px);
        background-image: url(//abc.ttt.com/abc?adfsd%3F=1231);
      }
      `,
      minify: false
    })

    expect(result).toMatchInlineSnapshot(`
      {
        "code": "@import "b.css";

      .c {
        color: green;
        -webkit-backdrop-filter: blur(2px);
        backdrop-filter: blur(2px);
        background-image: url("//abc.ttt.com/abc?adfsd%3F=1231");
      }
      ",
      }
    `)
  })

  it('works correctly minify', () => {
    const result = transformCSS({
      input: `
      @import 'b.css';
      .c {
        color: green;
        backdrop-filter: blur(2px);
        background-image: url(//abc.ttt.com/abc?adfsd%3F=1231);
      }
      `,
      minify: true
    })

    expect(result).toMatchInlineSnapshot(`
      {
        "code": "@import "b.css";.c{color:green;-webkit-backdrop-filter:blur(2px);backdrop-filter:blur(2px);background-image:url(//abc.ttt.com/abc?adfsd%3F=1231)}",
      }
    `)
  })

  it('invalid css throw error', () => {
    try {
      transformCSS({
        input: `
        @import 'b.css';
        .c 
          color: green;
          backdrop-filter: blur(2px);
          background-image: url(//abc.ttt.com/abc?adfsd%3F=1231);
        }
        `,
        minify: true,
      })
    } catch (e) {
      expect(e).toMatchInlineSnapshot(`[Error: Unexpected end of input at :7:9]`)
    }
  });

  it('should import rule first', () => {
    try {
      transformCSS({
        input: `
        .c {
          color: green;
        }
        
        @import 'b.css';
        `,
        minify: true,
      })
    } catch (e) {
      expect(e).toMatchInlineSnapshot(`[Error: @import rules must precede all rules aside from @charset and @layer statements at :5:16]`)
    }
  });
})