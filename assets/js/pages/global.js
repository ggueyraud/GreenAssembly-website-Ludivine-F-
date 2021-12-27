import 'router';

document.addEventListener('readystatechange', e => {
    if (e.target.readyState === 'complete') {
        const navbar = document.querySelector('#topbar nav');
        const open_menu_btn = document.querySelector('#open_mobile_menu');
        const close_mobile_menu = () => {
            open_menu_btn.querySelector('svg').innerHTML = '<use xlink:href="/icons.svg#burger"></use>';
            navbar.classList.remove('show');
            document.documentElement.style.overflow = null;
            
        }

        navbar?.querySelectorAll('nav a:not(.socials a)')
            .forEach(link => link.addEventListener('click', () => {
                if (navbar.classList.contains('show')) {
                    close_mobile_menu();
                }
            }));

        open_menu_btn
            ?.addEventListener('click', e => {
                e.preventDefault();
                
                if (navbar.classList.contains('show')) {
                    close_mobile_menu();
                } else {
                    open_menu_btn.querySelector('svg').innerHTML = '<use xlink:href="/icons.svg#close"></use>';
                    navbar.classList.add('show');
                    document.documentElement.style.overflow = 'hidden';
                }
            });
    }
});

window.addEventListener('unload', () => {
    if (!navigator.sendBeacon) return;
    
    if (document.visibilityState === 'hidden') {
        const METRIC_TOKEN = localStorage.getItem("METRIC_TOKEN");
    
        if (METRIC_TOKEN !==  null) {
            navigator.sendBeacon('/metrics/log', new URLSearchParams({
                token: METRIC_TOKEN
            }));
        }
    }
}, false);

// Check webp support
(() => {
    const img = new Image();
    img.onload = () => document.documentElement.classList.add('webp_supported');
    img.onerror = () => document.documentElement.classList.add('no_webp');

    img.src = 'data:image/webp;base64,UklGRjoAAABXRUJQVlA4IC4AAACyAgCdASoCAAIALmk0mk0iIiIiIgBoSygABc6WWgAA/veff/0PP8bA//LwYAAA';
})()