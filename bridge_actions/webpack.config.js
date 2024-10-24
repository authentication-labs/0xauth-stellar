const path = require('path');
const nodeExternals = require('webpack-node-externals');

module.exports = {
    entry: {
        createIdentityAddKey: './createIdentityAddKey.js',
        addKeyStellar: './addKeyStellar.js',
        addClaimStellar: './addClaimStellar.js',
        removeKeyStellar: './removeKeyStellar.js',
        removeClaimStellar: './removeClaimStellar.js'
    },
    target: 'node',
    externals: [nodeExternals({
        allowlist: [
            '@stellar/stellar-sdk',
            '@stellar/stellar-base',
            'toml',
            '@stellar/js-xdr',
            'base32.js',
            'urijs',
            'urijs/src/URITemplate',
            'eventsource'
        ]
    })],
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: '[name]/index.js',
        libraryTarget: 'commonjs2',
    },
    module: {
        rules: [
            {
                test: /\.js$/,
                exclude: /node_modules/,
                use: {
                    loader: 'babel-loader',
                    options: {
                        presets: ['@babel/preset-env']
                    }
                }
            }
        ]
    },
};
