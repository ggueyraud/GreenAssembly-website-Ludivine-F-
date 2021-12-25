import Form, { Regex } from 'formvalidation';
import { patch } from '@js/utils/http';
import 'router';

const { router } = window;

router.on('mount', () => {
    const check_url = new Regex(
        /^https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$/,
        'L\'url saisie n\'est pas valide'
    );

    // Store values of initial inputs value
    let creations = document.querySelector('#creations_input').value;
    let shootings = document.querySelector('#shootings_input').value;

    new Form(document.querySelector('#links_form'), {
        fields: {
            creations: {
                validators: [check_url]
            },
            shootings: {
                validators: [check_url]
            }
        }
    })
    .on('send', e => {
        e.preventDefault();

        let body = {};

        // Compare with previous value
        if(e.detail.creations !== creations) {
            body.creations = e.detail.creations;
        }

        if(e.detail.shootings !== shootings) {
            body.shootings = e.detail.shootings;
        }

        if (Object.entries(body).length) {
            patch('/api/my_little_plus/links', {
                headers: {
                    'Content-Type': 'application/json'
                },
                body
            })
            .then(() => {
                creations = e.detail.creations;
                shootings = e.detail.shootings;
    
                success_message.classList.add('show_message');
                error_message.classList.remove('show_message');
            })
            .catch(() => {
                success_message.classList.remove('show_message');
                error_message.classList.add('show_message');
            });
        }
    });
});