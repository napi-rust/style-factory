import { readFileSync } from 'fs-extra';
import styleFactory from 'style-factory';
import { styleFactory as styleFactoryRust } from '../index';
import { expect } from 'vitest';
import path, { dirname } from 'node:path';
import { writeFileSync } from 'node:fs';
import { transform } from 'lightningcss';
import { requireModuleCode } from 'require-module-exports';
import prettier from 'prettier';

function minifyCss(css: string) {
  return new Promise<string>((resolve) => {
    resolve(
      transform({
        filename: 'a.css',
        code: Buffer.from(css, 'utf8'),
        minify: true,
      }).code.toString(),
    );
  });
}

async function formatCss(code: string) {
  try {
    return await prettier.format(code, {
      parser: 'css',
      tabWidth: 2,
      singleQuote: true,
      trailingComma: 'all',
      semi: true,
    });
  } catch (e) {
    return code;
  }
}

async function writeFile(file: string, code: string) {
  const ext = path.extname(file);
  const isCss = ext === '.css';
  const formatCode = await prettier.format(code, {
    parser: isCss ? 'css' : 'babel',
    tabWidth: 2,
    singleQuote: true,
    trailingComma: 'all',
    semi: true,
  });
  writeFileSync(file, formatCode);
}

export const runCompare = async (file: string) => {
  const context = readFileSync(file);
  const dir = dirname(file);

  const code = await minifyCss(context.toString());

  await writeFile(file.replace('.css', '.min.css'), code);

  const jsCode = styleFactory(code, {
    transformTag: (t) => {
      if (t === '\\*') {
        return 'unsupport-star';
      }
      if (t === 'page') {
        return '[meta\\\\:tag=page]';
      }
      return `[meta\\\\:tag=${t}]`;
    },
  });
  const rustCode = styleFactoryRust(code);

  await writeFile(path.join(dir, 'js.js'), jsCode);
  await writeFile(path.join(dir, 'rs.js'), rustCode);

  const jsFn = requireModuleCode(jsCode).default;
  const rustFn = requireModuleCode(rustCode).default;

  const options = {
    prefix: 'sf-',
    rpx: (value: number) => value / 2,
    tag: (tag: string) => {
      return tag;
    },
    host: 'component-666',
  };
  const jsResult = jsFn(options);
  const rustResult = rustFn(options);

  const jsCss = await minifyCss(jsResult);
  const rustCss = await minifyCss(rustResult);

  expect(jsCss).toMatchSnapshot('js');
  expect(rustCss).toMatchSnapshot('rs');

  await writeFile(path.join(dir, 'js.css'), jsCss);
  await writeFile(path.join(dir, 'rust.css'), rustCss);

  expect(jsCss.length).toEqual(rustCss.length);
  expect(jsCss).toEqual(rustCss);
};
