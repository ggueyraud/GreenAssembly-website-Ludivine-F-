import 'router';

// window.addEventListener('router:mount', () => console.log('router:mount'));
// window.addEventListener('router:destroy', () => console.log('router:destroy'));
// window.addEventListener('router:loading', () => console.log('router:loading'));
// window.addEventListener('router:change', () => console.log('router:change'));
// window.router = new Router();

document.addEventListener('readystatechange', e => {
    if (e.target.readyState === 'interactive') {
        console.log(document.body)

    } else if (e.target.readyState === 'complete') {
        // console.log(Router)

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