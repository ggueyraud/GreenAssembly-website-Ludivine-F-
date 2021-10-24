const fs = require('fs');
const path = require('path');

module.exports = {
    resolve: {
        alias: {
            '@js': path.resolve(__dirname, 'assets/js/')
        }
    },
    entry: {
        global: './assets/js/global.js',
        contact: './assets/js/contact.js',
        project: './assets/js/project.js',
        portfolio: './assets/js/portfolio.js',
        login: './assets/js/login.js',
        'dashboard/portfolio': './assets/js/pages/dashboard/portfolio.js'
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
}