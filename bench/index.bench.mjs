import { styleFactory as rustStyleFactory } from '../index.js';
import styleFactory from 'style-factory';
import { bench } from 'vitest';

import styleFactoryPkg from 'style-factory/package.json';

const css = `body { color: red; height: 100px; width: 100rpx; }`;

bench('rust', () => {
  rustStyleFactory(css);
});

bench('js styleFactory ' + styleFactoryPkg.version, () => {
  styleFactory(css);
});
