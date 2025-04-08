import { transform } from 'lightningcss';

export function minifyCss(css: string) {
  return new Promise<string>((resolve) => {
    resolve(
      transform({
        filename: 'a.css',
        code: Buffer.from(css, 'utf8'),
        minify: true,
        targets: {
          safari: 11 << 16,
          chrome: 55 << 16,
        },
      }).code.toString(),
    );
  });
}
