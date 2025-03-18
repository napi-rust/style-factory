import test from 'ava'

import { sum } from '../index.js'

test('sum from native', (t) => {
  const ab = sum(1, 2);
  t.is(ab, 3)
})
