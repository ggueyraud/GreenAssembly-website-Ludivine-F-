import 'router';

window.router.on('mount', () => {
    if(document.documentElement.classList.contains('no_webp')) {
        document.documentElement.style.setProperty('--header_img_mobile', 'url(/uploads/mobile/index.png)');
        document.documentElement.style.setProperty('--header_img', 'url(/uploads/index.png)');
    } else {
        document.documentElement.style.setProperty('--header_img_mobile', 'url(/uploads/mobile/index.webp)');
        document.documentElement.style.setProperty('--header_img', 'url(/uploads/index.webp)');
    }
});