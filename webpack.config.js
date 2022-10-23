const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
    target: 'webworker',
    mode: "production",
    entry: {
        index: "./js/index.ts"
    },
    output: {
        path: dist,
        filename: "[name].js",
        libraryTarget: 'commonjs'
    },
    devServer: {
        contentBase: dist,
    },
    resolve: {
        extensions: ['.tsx', '.ts', '.js'],
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules|pkg/,
            },
        ],
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: __dirname,
        }),
    ]
};
