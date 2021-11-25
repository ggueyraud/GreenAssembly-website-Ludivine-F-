import Form, { Regex } from 'formvalidation';
import { get, patch } from '@js/utils/http';
import 'router';

let links = undefined;
const check_url = new Regex(
    /^https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$/,
    'L\'url saisie n\'est pas valide'
);

const validate_form = () => {
    const creations_input = document.querySelector('#creations_input');
    const shootings_input = document.querySelector('#shootings_input');

    const success_message = document.querySelector('#success_message');
    const error_message = document.querySelector('#error_message');

    const creations = creations_input?.value.trim();
    const shootings = shootings_input?.value.trim();

    let data = {};

    if((creations || creations === '') && creations != links.creations) {
        data['creations'] = creations;
    }

    if((shootings || shootings === '') && shootings != links.shootings) {
        data['shootings'] = shootings;
    }
    
    patch('/api/my_little_plus/links', {
        headers: {
            'Content-Type': 'application/json'
        },
        body: data
    })
    .then(res => {
        links = {
            creations: creations ?? links.creations,
            shootings: shootings ?? links.shootings
        }

        success_message.classList.add('show_message');
    })
    .catch(() => {
        success_message.classList.remove('show_message');
        error_message.classList.add('show_message');
    });
}

window.router.on('mount', () => {
    const form = document.querySelector('#links_form');
    const creations_input = document.querySelector('#creations_input');
    const shootings_input = document.querySelector('#shootings_input');

    const success_message = document.querySelector('#success_message');
    const error_message = document.querySelector('#error_message');

    get('/api/my_little_plus/links')
    .then(res => {
        return res.json()
    })
    .then(res => {
        links = res;

        creations_input.value = links.creations;
        shootings_input.value = links.shootings;

        error_message.classList.remove('show_message');
    })
    .catch(() => {
        success_message.classList.remove('show_message');
        error_message.classList.add('show_message');
    });

    new Form(form, {
        fields: {
            creations_input: {
                validators: [check_url]
            },
            shootings_input: {
                validators: [check_url]
            }
        }
    })
    .on('send', e => {
        e.preventDefault();
        validate_form();
    });
});