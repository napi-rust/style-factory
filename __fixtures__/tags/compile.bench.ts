import { join } from 'path';
import { runCompile } from '../runCompile.ts';

const input = join(__dirname, 'index.css');

runCompile(input);
