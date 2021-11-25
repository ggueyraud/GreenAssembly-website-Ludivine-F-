import 'router';


window.router.on('mount', () => {
    if(document.documentElement.classList.contains('no_webp')) {
        document.documentElement.style.setProperty('--header_img_mobile', 'url(/img/index_mobile.png)');
        document.documentElement.style.setProperty('--header_img', 'url(/img/index.png)');
    } else {
        document.documentElement.style.setProperty('--header_img_mobile', 'url(/img/index_mobile.webp)');
        document.documentElement.style.setProperty('--header_img', 'url(/img/index.webp)');
    }
});