import Carousel, { CarouselPagination, CarouselTouch } from 'carousel';

const on_mount = () => {
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
}

const on_destroy = () => {
    window.removeEventListener('onMount', on_mount)
    window.removeEventListener('onDestroy', on_destroy)
}

window.addEventListener('onMount', on_mount)
window.addEventListener('onDestroy', on_destroy)