import 'router';
import Form, { Required, Regex, StringLength } from 'formvalidation';
import { post } from '../utils/http';

window.router.on('mount', () => {
    const required_validator = new Required();
    // const comm
    new Form(document.querySelector('form'), {
        fields: {
            lastname: {
                validators: [required_validator, new StringLength(2, 120)],
                container: document.querySelector('[for="lastname"]').nextElementSibling
            },
            firstname: {
                validators: [required_validator, new StringLength(2, 120)],
                container: document.querySelector('[for="firstname"]').nextElementSibling
            },
            phone_number: {
                validators: [new Regex(/^((\+)33|0|0033)[1-9](\d{2}){4}$/, 'Mauvais format de numéro saisi')],
                container: document.querySelector('[for="phone_number"]').nextElementSibling
            },
            email: {
                validators: [
                    required_validator,
                    new Regex(
                    /^(([^<>()[\]\\.,;:\s@"]+(\.[^<>()[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$/,
                    "L'email saisit n'a pas un format valide"
                    )
                ],
                container: document.querySelector('[for="email"]').nextElementSibling
            },
            content: {
                validators: [
                    required_validator,
                    new StringLength(30, 500)
                ],
                container: document.querySelector('[for="content"]').nextElementSibling
            }
        }
    })
        .on('send', e => {
            const body = {};

            for (const [key, value] of Object.entries(e.detail)) {
                if (value) {
                    body[key] = value;
                }
            }

            post('/contact', {
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                },
                validate_status: status => status === 200,
                body: new URLSearchParams(body)
            })
                .then(() => {
                    e.target.classList.add('hidden');
                    document.querySelector('#success').classList.remove('hidden');
                })
                .catch(() => {
                    document.querySelector('#error').classList.remove('hidden');
                });
        });

    document
        .querySelectorAll('.input')
        .forEach(input_container => {
            const input = input_container.querySelector('input, textarea');

            // Set autoheight
            if (input instanceof HTMLTextAreaElement) {
                input
                    .addEventListener('input', e => {
                        e.target.style.height = "100px";

                        if (e.target.scrollHeight > 100) {
                            e.target.style.height = `${e.target.scrollHeight}px`;
                        }
                    })
            }

            input
                .addEventListener('focus', () => {
                    input_container.classList.remove('valid');
                });

            input
                .addEventListener('blur', e => {
                    if (e.target.value.length > 0 && !e.target.classList.contains('fv_border_error')) {
                        input_container.classList.add('valid')
                    }
                })
        });
});