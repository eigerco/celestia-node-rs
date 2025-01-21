const path = require('path');

module.exports = {
  entry: './index.js',
  mode: 'development',
  output: {
    filename: 'main.js',
    path: path.resolve(__dirname, 'dist'),
  },
  experiments: {
    asyncWebAssembly: true,
  },
  resolve: {
    // hack needed for our local setup as it uses 'file:../../node-wasm/...' dependencies in package.json
    // which create symlinks in `node_modules`. Webpack will follow it, but then when it tries
    // to resolve `lumina-node-wasm` from `lumina-node`, it won't jump back from symlink and
    // instead it will try to find `node_modules` in parent directories of '../../node-wasm/js'.
    // Instead we need to guide it to always use `node_modules` we have there locally.
    // https://webpack.js.org/configuration/resolve/#resolvemodules
    modules: [path.resolve(__dirname, 'node_modules')],
    // disable nodejs modules from cosmjs
    fallback: {
      buffer: false,
      crypto: false,
      events: false,
      path: false,
      stream: false,
      string_decoder: false,
    },
  },
};
