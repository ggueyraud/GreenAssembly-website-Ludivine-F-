import {  patch } from '@js/utils/http';
import 'router';

const edit_image = (formdata) => {
    const loader = document.querySelector('#loader');
    loader.classList.add('loader--shown');

    const success_message = document.querySelector('#success_message');
    const error_message = document.querySelector('#error_message');

    patch('/api/home/image', {
        body: formdata
    })
    .then(res => {
        reset_form();
        success_message.classList.add('show_message');
        error_message.classList.remove('show_message');
    })
    .catch(() => {
        success_message.classList.remove('show_message');
        error_message.classList.add('show_message');
    })
    .finally(() => {
        loader.classList.remove('loader--shown');
    });
}

const update_form = () => {
    const image_input = document.querySelector('#image');
    const image_input_label = document.querySelector('#image_input_label');
    const image_edit_button = document.querySelector('#image_edit_button');
    const files_count = image_input.files.length;

    if(files_count > 0) {
        image_input_label.innerText = 'Images sélectionnés: ' + files_count;
        image_edit_button.disabled = false;
    } else {
        image_input_label.innerText = 'Choisir une image';
        image_edit_button.disabled = true;
    }
}

const reset_form = () => {
    const form = document.querySelector('#home_form');
    form.reset();

    update_form(form);
}

window.router.on('mount', () => {
    const form = document.querySelector('#home_form');
    const image_input = document.querySelector('#image');

    const success_message = document.querySelector('#success_message');
    const error_message = document.querySelector('#error_message');

    image_input.addEventListener('change', update_form);

    form.addEventListener('submit', e => {
        e.preventDefault();

        success_message.classList.remove('show_message');
        error_message.classList.remove('show_message');

        const formdata = new FormData(form);
        edit_image(formdata);

        form.reset();
    });
});
