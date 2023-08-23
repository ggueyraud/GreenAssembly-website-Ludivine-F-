import Carousel, { CarouselPagination, CarouselTouch } from 'carousel';
import lightbox from '@js/components/lightbox';
import 'router';

const { router } = window;

router.on('mount', () => {
    lightbox('img');

    const carousel = new Carousel(document.querySelector('.carousel'), {
        breakpoints: {
            768: {
                slides_visible: 2
            },
            1280: {
                slides_visible: 3
            }
        }
    });
    carousel.use([CarouselTouch, CarouselPagination]);
});