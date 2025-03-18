import test from 'ava'

import { styleFactory } from '../index.js'

test('styleFactory', (t) => {
  const css = styleFactory(`body { color: red; }`)
  t.is(css, 'body{color:red}')
})
