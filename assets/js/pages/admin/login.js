import Form, { Required, Regex } from 'formvalidation';
import { post } from '../../utils/http';
import 'router';

window.router.on('mount', () => {
    const required_validator = new Required();
    const email_validator = new Regex(
        /^(([^<>()[\]\\.,;:\s@"]+(\.[^<>()[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$/,
        `L'email saisit n'a pas un format valide`
    );
    const login_form = document.querySelector('[name=login]');
    const lost_password_form = document.querySelector('[name=lost_password]')
    
    new Form(login_form, {
        fields: {
            password: {
                validators: [required_validator]
            },
            email: {
                validators: [required_validator, email_validator]
            }
        }
    })
        .on('send', e => {
            e.preventDefault();

            post('/user/login', {
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                },
                validate_status: status => status === 200,
                body: new URLSearchParams(e.detail)
            })
                .then(() => window.router.push('admin', false))
                .catch(() => {
                    document.querySelector('#login_error').classList.remove('hidden');
                })
        });

    const lost_password_error = document.querySelector('#lost_password_error');
    new Form(lost_password_form, {
        fields: {
            recovery_email: {
                validators: [required_validator, email_validator]
            }
        }
    })
        .on('send', e => {
            e.preventDefault();

            const body = Object.assign({}, e.detail);
            body.email = body.recovery_email;
            delete body.recovery_email;

            post('/user/lost-password', {
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                },
                validate_status: status => status === 200,
                body: new URLSearchParams(body)
            })
            .then(async res => {
                res = await res.json();
                
                if (!res.is_valid) {
                    lost_password_error.innerHTML = `L'email saisit n'existe pas ou n'est pas valide`;

                    if (lost_password_error.classList.contains('hidden')) {
                        lost_password_error.classList.remove('hidden')
                    }
                }
            })
            .catch(e => {
                let error_msg = null;

                // Too many attempts
                if (e.status === 429) {
                    error_msg = `Limite de tentatives de récupération d'email atteinte, veuillez réessayer d'ici une heure`;
                } else if (e.status === 400) {
                    error_msg = `L'email saisit n'a pas un format valide`;
                } else {
                    error_msg = 'Une erreur est survenue';
                }

                lost_password_error.innerHTML = error_msg;

                if (lost_password_error.classList.contains('hidden')) {
                    lost_password_error.classList.remove('hidden')
                }
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
});