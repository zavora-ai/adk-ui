import { spawn, spawnSync } from 'node:child_process';
import { mkdir, rm } from 'node:fs/promises';
import { createRequire } from 'node:module';

import { build, context } from 'esbuild';

const require = createRequire(import.meta.url);
const watch = process.argv.includes('--watch');
const shared = {
    entryPoints: ['src/index.ts'],
    bundle: true,
    external: [
        'clsx',
        'lucide-react',
        'react',
        'react/*',
        'react-dom',
        'react-dom/*',
        'react-markdown',
        'recharts',
    ],
    logLevel: 'info',
    platform: 'browser',
    target: 'es2020',
};
const builds = [
    { ...shared, format: 'cjs', outfile: 'dist/index.js' },
    { ...shared, format: 'esm', outfile: 'dist/index.mjs' },
];
const tscPath = require.resolve('typescript/bin/tsc');
const tscArgs = [tscPath, '--emitDeclarationOnly', '--noEmit', 'false'];

await rm('dist', { recursive: true, force: true });
await mkdir('dist', { recursive: true });

if (watch) {
    const contexts = await Promise.all(builds.map((options) => context(options)));
    await Promise.all(contexts.map((buildContext) => buildContext.watch()));
    spawn(process.execPath, [...tscArgs, '--watch'], { stdio: 'inherit' });
} else {
    await Promise.all(builds.map((options) => build(options)));
    const declarations = spawnSync(process.execPath, tscArgs, { stdio: 'inherit' });
    if (declarations.status !== 0) {
        process.exit(declarations.status ?? 1);
    }
}
