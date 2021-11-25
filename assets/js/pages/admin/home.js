import {  patch } from '@js/utils/http';
import 'router';

const edit_image = (formdata) => {
    patch('/api/home/image', {
        body: formdata
    })
    .then(res => {
        console.log(res)
    })
    .catch(() => {
        console.log('patch error')
    });
}

window.router.on('mount', () => {
    const form = document.querySelector('#home_form');
    const image_input = document.querySelector('#image');
    const image_input_label = document.querySelector('#image_input_label');
    const image_edit_button = document.querySelector('#image_edit_button');

    image_input.addEventListener('change', () => {
        const files_count = image_input.files.length;
        if(files_count > 0) {
            image_input_label.innerText = 'Images sélectionnés: ' + files_count;
            image_edit_button.disabled = false;
        } else {
            image_input_label.innerText = 'Choisir une image';
            image_edit_button.disabled = true;
        }
    });

    form.addEventListener('submit', e => {
        e.preventDefault();

        const formdata = new FormData(form);
        edit_image(formdata);
    });
});