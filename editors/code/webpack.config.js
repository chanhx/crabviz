//@ts-check

'use strict';

const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const WebpackShellPluginNext = require('webpack-shell-plugin-next');

//@ts-check
/** @typedef {import('webpack').Configuration} WebpackConfig **/

/** @type WebpackConfig */
const baseConfig = {
  mode: 'none', // this leaves the source code as close as possible to the original (when packaging we set this to 'production')
  externals: {
    vscode: 'commonjs vscode', // the vscode-module is created on-the-fly and must be excluded. Add other modules that cannot be webpack'ed, 📖 -> https://webpack.js.org/configuration/externals/
    // modules added here also need to be added in the .vscodeignore file
  },
  resolve: {
    // support reading TypeScript and JavaScript files, 📖 -> https://github.com/TypeStrong/ts-loader
    extensions: ['.ts', '.js']
  },
  devtool: "nosources-source-map",
  infrastructureLogging: {
    level: "log", // enables logging required for problem matchers
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: /node_modules/,
        use: [{ loader: "ts-loader" }],
      },
    ],
  },
};

/** @type WebpackConfig */
const extensionConfig = {
  ...baseConfig,
  target: 'node', // VS Code extensions run in a Node.js-context 📖 -> https://webpack.js.org/configuration/node/

  entry: './src/extension.ts', // the entry point of this extension, 📖 -> https://webpack.js.org/configuration/entry-context/
  output: {
    // the bundle is stored in the 'dist' folder (check package.json), 📖 -> https://webpack.js.org/configuration/output/
    path: path.resolve(__dirname, 'dist'),
    filename: 'extension.js',
    libraryTarget: 'commonjs2'
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, '../../core'),
      outDir: path.resolve(__dirname, 'out', 'crabviz'),
      extraArgs: '--target=web --features vscode',
      forceMode: 'production',
    }),
    // @ts-ignore
    new WebpackShellPluginNext({
      onBuildStart: {
        scripts: ['pnpm --dir ../../webview-ui build'],
        blocking: true,
        parallel: false
      }
    }),
  ],
  experiments: {
    futureDefaults: true,
    topLevelAwait: true,
  }
};

module.exports = [ extensionConfig ];
