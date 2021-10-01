const fs = require('fs');
const path = require('path');

module.exports = {
    entry: {
        global: './assets/js/global.js',
        contact: './assets/js/contact.js',
        project: './assets/js/project.js',
        portfolio: './assets/js/portfolio.js',
        login: './assets/js/login.js'
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
    }
}