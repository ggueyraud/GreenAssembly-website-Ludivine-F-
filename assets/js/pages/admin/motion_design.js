import 'router';
import Form, { Required, Regex } from 'formvalidation';
import { put } from '../../utils/http';
import swal_error from '@js/utils/swal_error';

const { router } = window;

router.on('mount', () => {
    console.log(document.querySelector('form'))
    new Form(document.querySelector('form'), {
        fields: {
            link: {
                validators: [
                    new Required(),
                    new Regex(
                        /^https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$/,
                        "L'url saisie n'est pas valide"
                    )
                ]
            }
        }
    })
        .on('send', e => {
            e.preventDefault();

            const submit_btn = e.target.querySelector('[type="submit"]');
            const submit_btn_before_send = submit_btn.innerHTML;
            submit_btn.setAttribute('disabled', true);
            submit_btn.innerHTML = `<svg class="icon icon--rotate icon--sm mr_2">
                <use xlink:href="/dashboard_icons.svg#redo"></use>
            </svg> Envoi en cours..`;

            put('/api/motion-design', {
                headers: {
                    'Content-Type': 'application/json'
                },
                body: e.detail
            })
            .then(() => {
                submit_btn.removeAttribute('disabled');
                submit_btn.innerHTML = submit_btn_before_send;
            })
            .catch(swal_error)
        })
});