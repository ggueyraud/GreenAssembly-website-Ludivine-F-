export default selector => {
    let lightbox = document.querySelector('.lightbox');
    let img;
    let body = document.querySelector('body');

    const hide = el => {
        el.classList.remove('lightbox--active');
        body.style.overflow = 'auto';
    }

    if (!lightbox) {
        lightbox = document.createElement('div');
        lightbox.classList.add('lightbox');

        img = document.createElement('img');
        lightbox.appendChild(img);

        document.querySelector('body').insertAdjacentElement('beforeend', lightbox);

        lightbox.addEventListener('click', e => hide(e.target))
    }

    window.addEventListener('keydown', e => {
        const active_box = document.querySelector('.lightbox--active');

        if (active_box && e.key === 'Escape') {
            hide(active_box)
        }
    });

    document
        .querySelectorAll(selector)
        .forEach(element => {
            element.addEventListener('click', e => {
                e.preventDefault();

                lightbox.classList.add('lightbox--active');
                img.src = element.src;
                body.style.overflow = 'hidden';
            })
        });
}