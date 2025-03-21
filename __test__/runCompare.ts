import { readFileSync } from 'fs-extra';
import styleFactory from 'style-factory';
import { styleFactory as styleFactoryRust } from '../index';
import path, { dirname } from 'node:path';
import { writeFileSync } from 'node:fs';
import { requireModuleCode } from 'require-module-exports';
import prettier from 'prettier';
import { expect } from 'vitest';
import { minifyCss } from './minifyCss';

async function buildStyle(options: {
  inputCss: string;
  buildId: string;
  dir: string;
  // biome-ignore lint/complexity/noBannedTypes: <explanation>
  factory: Function;
}) {
  const { buildId, factory, inputCss, dir } = options;
  const outputJsFile = path.join(dir, `${buildId}.js`);
  const outputCssFile = path.join(dir, `${buildId}.css`);

  let hostCss = '';
  const fnOptions = {
    prefix: 'sf-',
    rpx: (value: number) => value / 2,
    tag: (tag: string) => {
      return tag;
    },
    host: 'component-666',
    hostStyle(hostStyle: string) {
      hostCss = hostStyle;
    },
  };
  const jsCode = factory(inputCss, {
    transformTag: (t) => {
      if (t === '\\*') {
        return 'unsupported-star';
      }
      if (t === 'web-view') {
        return 'unsupported-web-view';
      }
      return `[meta\\\\:tag=${t}]`;
    },
  });
  // biome-ignore lint/complexity/noBannedTypes: <explanation>
  const codeFn = requireModuleCode(jsCode).default as Function;
  const outputCss = codeFn(fnOptions);
  await writeFile(outputJsFile, jsCode);
  const css = `/* host-begin */${await minifyCss(hostCss)}/* host-end */\n${await minifyCss(outputCss)}`;
  await writeFile(outputCssFile, css);
  return css;
}

export const runCompare = async (file: string) => {
  const context = readFileSync(file);
  const code = context.toString();
  // const code = await minifyCss(context.toString());
  await writeFile(file.replace('.css', '.min.css'), `/* ${new Date().toLocaleString()} */\n${code}`);
  const dir = dirname(file);
  const left = await buildStyle({ inputCss: code, dir, buildId: 'js', factory: styleFactory });
  const right = await buildStyle({ inputCss: code, dir, buildId: 'rs', factory: styleFactoryRust });
  if (!dir.endsWith('large')) {
    expect(await formatCss(left)).toEqual(await formatCss(right));
  }
};

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
