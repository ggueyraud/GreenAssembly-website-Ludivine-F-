import Form from 'formvalidation';
import { patch } from '@js/utils/http';
import swal_error from '@js/utils/swal_error';
import 'router';
import { DropZone } from '@js/components/assets_grid';

const { router } = window;

router.on('mount', () => {
    const logo_input = document.querySelector('[name="logo"]').parentElement;
    const favicon_input = document.querySelector('[name="favicon"]').parentElement;
    new DropZone(logo_input);
    new DropZone(favicon_input);

    // Saved form values
    let background_color = document.querySelector('[name="background_color"]').value;
    let title_color = document.querySelector('[name="title_color"]').value;
    let text_color = document.querySelector('[name="text_color"]').value;
    let logo = logo_input.querySelector('img').getAttribute('src');
    let favicon = favicon_input.querySelector('img').getAttribute('src');

    if (logo) {
        logo_input.classList.add('drop_zone--is-filled');
    }
    if (favicon) {
        favicon_input.classList.add('drop_zone--is-filled');
    }

    new Form(document.querySelector('form'), {
        fields: {
            logo: {},
            favicon: {},
            background_color: {},
            title_color: {},
            text_color: {}
        }
    })
    .on('send', e => {
        e.preventDefault();

        const body = new FormData();
        console.log(e.detail)

        if (e.detail.background_color !== background_color) {
            body.append('background_color', e.detail.background_color);
        }
        if (e.detail.title_color !== title_color) {
            body.append('title_color', e.detail.title_color);
        }
        if (e.detail.text_color !== text_color) {
            body.append('text_color', e.detail.text_color);
        }

        if (e.detail.logo[0]) {
            body.append('logo', e.detail.logo[0]);
        }
        
        if (e.detail.favicon[0]) {
            body.append('favicon', e.detail.favicon[0]);
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