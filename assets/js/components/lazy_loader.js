export default () => {
    const lazy_images = [].slice.call(document.querySelectorAll(".lazy"));

    if ("IntersectionObserver" in window) {
        let lazy_image_observe = new IntersectionObserver((entries, observer) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    const lazy_image = entry.target;


                    if (lazy_image instanceof HTMLPictureElement) {
                        lazy_image
                            .querySelectorAll('source')
                            .forEach(source => {
                                source.srcset = source.dataset.srcset;
                            });

                        const img = lazy_image.querySelector('img');
                        img.src = img.dataset.src;
                    } else if (lazy_image instanceof HTMLImageElement) {
                        lazy_image.src = lazy_image.dataset.src;
                    }

                    lazy_image.classList.remove("lazy");
                    lazy_image_observe.unobserve(lazy_image);
                }
            });
        });

        lazy_images.forEach(img => {
            lazy_image_observe.observe(img);
        });
    } else {
      // Possibly fall back to event handlers here
      lazy_images
        .forEach(img => {
            if (img instanceof HTMLPictureElement) {
                img
                    .querySelectorAll('source')
                    .forEach(source => {
                        source.srcset = source.dataset.srcset;
                    })
            } else if (img instanceof HTMLImageElement) {
                img.src = img.dataset.src;
            }
        })
    }
}