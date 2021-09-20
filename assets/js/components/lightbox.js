export default selector => {
    let lightbox = document.querySelector('.lightbox');
    let img;
    let body = document.querySelector('body');

    if (!lightbox) {
        lightbox = document.createElement('div');
        lightbox.classList.add('lightbox');

        img = document.createElement('img');
        lightbox.appendChild(img);

        document.querySelector('body').insertAdjacentElement('beforeend', lightbox);

        lightbox.addEventListener('click', () => {
            lightbox.classList.remove('lightbox--active');
            body.style.overflow = 'auto';
        })
    }

    document
        .querySelectorAll(selector)
        .forEach(element => {
            element.addEventListener('click', e => {
                console.log(e, element)
                e.preventDefault();

                lightbox.classList.add('lightbox--active');
                img.src = element.src;
                body.style.overflow = 'hidden';
            })
        });
}