import Router from 'router';

document.addEventListener('readystatechange', e => {
    if (e.target.readyState === 'complete') {
        window.router = new Router();

        const navbar = document.querySelector('#topbar nav');
        const open_menu_btn = document.querySelector('#open_mobile_menu');

        if (open_menu_btn) {
            const close_mobile_menu = () => {
                
            }
            
            open_menu_btn
            .addEventListener('click', e => {
                e.preventDefault();
                
                if (navbar.classList.contains('show')) {
                    open_menu_btn.querySelector('svg').innerHTML = '<use xlink:href="/icons.svg#burger"></use>';
                    navbar.classList.remove('show');
                    document.documentElement.style.overflow = null;
                    window.history.pushState(null, null, ' ');
                    } else {
                        open_menu_btn.querySelector('svg').innerHTML = '<use xlink:href="/icons.svg#close"></use>';
                        window.history.pushState({ menu_opened: true }, null, '#menu-opened');
                        navbar.classList.add('show');
                        document.documentElement.style.overflow = 'hidden';
    
                    }
                });
    
            // document
            //     .querySelector('#close_mobile_menu')
            //     .addEventListener('click', close_mobile_menu)
        }

    }
});

document.addEventListener('visibilitychange', () => {
    console.log('visibilitychance', document.visibilityState)
    if (!navigator.sendBeacon) return;

    if (document.visibilityState === 'hidden') {
        const METRIC_TOKEN = localStorage.getItem("METRIC_TOKEN");

        if (METRIC_TOKEN !==  null) {
            navigator.sendBeacon('/metrics/log', new URLSearchParams({
                token: METRIC_TOKEN
            }));
        }
    }
});