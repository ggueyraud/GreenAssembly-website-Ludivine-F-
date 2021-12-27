import lightbox from '@js/components/lightbox';
import 'router';

const { router } = window;

router.on('mount', () => {
    lightbox('main img');
});