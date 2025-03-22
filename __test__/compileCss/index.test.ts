import { describe, it, expect } from "vitest";
import { compileCSS } from "../../index";
import path from "node:path";

import * as aaa from '../../index'
console.log(aaa)

describe("compileCss", () => {
  const entry = path.resolve('./a.css')
  it("should compile css", () => {
    const result = compileCSS(entry)
    expect(result.css).toMatchInlineSnapshot()
    expect(result.dependencies).toMatchInlineSnapshot()
  })
})