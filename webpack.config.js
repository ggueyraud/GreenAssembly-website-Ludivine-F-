const path = require('path');
const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;

let plugins = [];

if (process.env.NODE_ENV === 'production') {
    plugins.push(new BundleAnalyzerPlugin());
}

const entry_path = './assets/js/pages';

module.exports = {
    resolve: {
        alias: {
            '@js': path.resolve(__dirname, 'assets/js')
        }
    },
    entry: {
        global: `${entry_path}/global.js`,
        contact: `${entry_path}/contact.js`,
        project: `${entry_path}/project.js`,
        portfolio: `${entry_path}/portfolio.js`,
        'admin/login': `${entry_path}/admin/login.js`,
        'admin/portfolio': `${entry_path}/admin/portfolio.js`,
        'admin/settings': `${entry_path}/admin/settings.js`
    },
    watch: process.env.NODE_ENV === 'development',
    watchOptions: {
        ignored: /node_modules/
    },
    target: "browserslist",
    output: {
        path: path.resolve(__dirname, 'dist')
    },
    experiments: {
        syncWebAssembly: true,
        asset: true
    },
    // module: {
    //     rules: [
    //       { test: /\.js$/, exclude: /node_modules/, loader: "babel-loader" }
    //     ]
    // }
    plugins
}