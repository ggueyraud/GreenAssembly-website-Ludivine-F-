import 'router';
import lightbox from '@js/components/lightbox';

const { router } = window;

let categories = null;

router.on('mount', () => {
    lightbox('article img');
    categories = document.querySelectorAll('main > nav > a');
    const global_category = categories[0];

    if (location.pathname.includes('/articles')) {
        global_category.classList.add('hidden');
    } else {
        global_category.classList.remove('hidden');
    }
});