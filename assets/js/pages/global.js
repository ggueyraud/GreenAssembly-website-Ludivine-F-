import 'router';
import LazyLoader from '@js/components/lazy_loader';
import { get } from '@js/utils/http';

const { router } = window;

const read_cookie = (cookie_name) => {
    return document.cookie.split('; ').find(row => row.startsWith(cookie_name))?.split('=')?.[1]
}

const send_metrics = () => {
    if (!navigator.sendBeacon) return;
    
    const vid = localStorage.getItem('VID');
    const sid = read_cookie('sid');

    if (vid !== null) {
        const { pathname } = location;
        const belongs_to = pathname.includes('/articles/')
            ? 'BlogPost'
            : pathname.includes('/portfolio/')
                ? 'Project'
                : 'Page';

        console.log(belongs_to)
        navigator.sendBeacon('/metrics/log', new URLSearchParams({
            sid: sid ?? null,
            token: vid
        }));

        localStorage.setItem('VID', '');
    }
}

document.addEventListener('readystatechange', e => {
    if (e.target.readyState === 'complete') {
        LazyLoader();
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

router.on('change', async () => {
    send_metrics();

    let sid = read_cookie('sid');
    if(!sid) {
        const res = await get('/metrics/session');
        const data = await res.json();
        sid = data.sid;
        document.cookie = 'sid=' + sid + '; expires=' + new Date(data.vud).toUTCString() + '; SameSite=Strict; Secure';
    }

    const res = await get(`/metrics/token?path=${location.pathname}&sid=${sid}`);
    localStorage.setItem('VID', await res.text());
});

window.addEventListener('unload', send_metrics, false);

// Check webp support
(() => {
    const img = new Image();
    img.onload = () => document.documentElement.classList.add('webp_supported');
    img.onerror = () => document.documentElement.classList.add('no_webp');

    img.src = 'data:image/webp;base64,UklGRjoAAABXRUJQVlA4IC4AAACyAgCdASoCAAIALmk0mk0iIiIiIgBoSygABc6WWgAA/veff/0PP8bA//LwYAAA';
})()