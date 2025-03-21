import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';
import { runCompare } from './runCompare';
import pkg from 'style-factory/package.json';
vi.setConfig({
  testTimeout: 20_000,
});

const fixtures = path.join(__dirname, '../__fixtures__');

const cases = fs
  .readdirSync(fixtures)
  .filter((f) => {
    return fs.statSync(path.join(fixtures, f)).isDirectory();
  })
  .filter((f) => !f.startsWith('.'))
  .filter((f) => {
    if (process.env.filter) {
      return f.includes(process.env.filter);
    }
    return true;
  });

const ignore = ['import', 'import2'];

for (const f of cases) {
  if (ignore.includes(f)) {
    continue;
  }
  const cssFile = path.join(fixtures, f, 'index.css');
  describe(`style-factory ${pkg.version} vs rust : ${f}`, async () => {
    it('style-factory', async () => {
      await runCompare(cssFile);
    });
  });
}
