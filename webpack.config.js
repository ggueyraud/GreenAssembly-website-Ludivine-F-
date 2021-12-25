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
        index: `${entry_path}/index.js`,
        global: `${entry_path}/global.js`,
        contact: `${entry_path}/contact.js`,
        blog: `${entry_path}/blog.js`,
        project: `${entry_path}/project.js`,
        portfolio: `${entry_path}/portfolio.js`,
        'admin/login': `${entry_path}/admin/login.js`,
        'admin/home': `${entry_path}/admin/home.js`,
        'admin/portfolio': `${entry_path}/admin/portfolio.js`,
        'admin/my_little_plus': `${entry_path}/admin/my_little_plus.js`,
        'admin/settings': `${entry_path}/admin/settings.js`,
        'admin/blog': `${entry_path}/admin/blog.js`,
        'admin/motion_design': `${entry_path}/admin/motion_design.js`
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
        syncWebAssembly: true
    },
    plugins
}