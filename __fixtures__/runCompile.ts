import { readFileSync } from 'fs-extra';
import styleFactory from 'style-factory';
import { styleFactory as styleFactoryRust } from '../index';
import { bench } from 'vitest';
import pkg from 'style-factory/package.json';

export const runCompile = (file: string) => {
  const context = readFileSync(file, 'utf8');

  {
    bench(`styleFactory ${pkg.version} (${context.length})`, () => {
      styleFactory(context);
    });

    bench(`styleFactory Rust (${context.length})`, () => {
      styleFactoryRust(context);
    });
  }
};
