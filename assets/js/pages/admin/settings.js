import Form from 'formvalidation';
import { patch } from '@js/utils/http';
import swal_error from '@js/utils/swal_error';
import 'router';

const { router } = window;

router.on('mount', () => {
    let background_color = document.querySelector('[name="background_color"]').ariaValueMax;
    let title_color = document.querySelector('[name="title_color"]').value;
    let text_color = document.querySelector('[name="text_color"]').value;

    new Form(document.querySelector('form'), {
        fields: {
            background_color: {},
            title_color: {},
            text_color: {}
        }
    })
    .on('send', e => {
        e.preventDefault();

        const body = new FormData();

        if (e.detail.background_color !== background_color) {
            body.append('background_color', e.detail.background_color);
        }
        if (e.detail.title_color !== title_color) {
            body.append('title_color', e.detail.title_color);
        }
        if (e.detail.text_color !== text_color) {
            body.append('text_color', e.detail.text_color);
        }

        let i = 0;
        for (const _ of body.entries()) {
            i += 1;
        }

        if (i > 0) {
            patch('/api/settings', {
                // headers: {
                //     'Content-Type': 'application/json'
                // },
                body
            })
            .then(() => {
                if (e.detail.background_color !== background_color) {
                    background_color = e.detail.background_color;
                }
                if (e.detail.title_color !== title_color) {
                    title_color = e.detail.title_color;
                }
                if (e.detail.text_color !== text_color) {
                    text_color = e.detail.text_color;
                }
            })
            .catch(swal_error)
        }

        console.log(e);
    });
});