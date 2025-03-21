import { readFileSync } from 'fs-extra';
import styleFactory from 'style-factory';
import { styleFactory as styleFactoryRust } from '../index';
import { describe, it, expect } from 'vitest';
import path from 'node:path';
import { browserslistToTargets, transform } from 'lightningcss';
import { requireModuleCode } from 'require-module-exports';

function minifyCss(css: string) {
  return transform({
    filename: 'a.css',
    code: Buffer.from(css, 'utf8'),
    minify: true,
    targets: browserslistToTargets(['safari 11', 'ios_saf 11', 'Android 6', 'chrome 66']),
  }).code.toString();
}

export const runCompare = (file: string) => {
  const context = readFileSync(file);
  const relative = path.relative(process.cwd(), file);

  const code = minifyCss(context.toString());
  const jsCode = styleFactory(code.toString());
  const rustCode = styleFactoryRust(code.toString());

  const jsFn = requireModuleCode(jsCode).default;
  const rustFn = requireModuleCode(rustCode).default;

  const options = {
    prefix: 'sf-',
    rpx: (value: number) => value / 2,
    tag: (tag: string) => {
      if (tag === '*') {
        return 'unsupport-star';
      }
      return tag;
    },
    host: 'component-666',
  };
  const jsResult = jsFn(options);
  const rustResult = rustFn(options);

  const jsCss = minifyCss(jsResult);
  const rustCss = minifyCss(rustResult);

  expect(jsCss).toMatchSnapshot('js');
  expect(rustCss).toMatchSnapshot('rs');

  expect(jsCss.length).toEqual(rustCss.length);
};
