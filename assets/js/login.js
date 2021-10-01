import Form, { Required, Regex } from 'formvalidation';
import { post } from './utils/http';

const on_mount = () => {
    const required_validator = new Required();

    const login_form = document.querySelector('[name=login]');
    const lost_password_form = document.querySelector('[name=lost_password]')
    
    new Form(login_form, {
        fields: {
            password: {
                validators: [required_validator]
            },
            email: {
                validators: [
                    required_validator,
                    new Regex(/^(([^<>()[\]\\.,;:\s@"]+(\.[^<>()[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$/, `L'email saisit n'a pas un format valide`)
                ]
            }
        }
    })
        .on('send', e => {
            e.preventDefault();
            console.log(e)

            post('/user/login', {
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                },
                validate_status: status => status === 200,
                body: new URLSearchParams(e.detail)
            })
                .then(e => {
                    console.log(e)
                    window.router.handle_url('/admin')

                })
                .catch(() => {
                    document.querySelector('#login_error').classList.remove('hidden');
                })
        });

    document
        .querySelectorAll('.change_state_btn')
        .forEach(btn => {
            btn.addEventListener('click', () => {
                if (login_form.classList.contains('hidden')) {
                    login_form.classList.remove('hidden');
                    lost_password_form.classList.add('hidden');
                } else {
                    login_form.classList.add('hidden');
                    lost_password_form.classList.remove('hidden');
                }
            })
        })
}

window.addEventListener('onMount', on_mount)