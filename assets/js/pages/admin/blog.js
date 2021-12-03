import 'router';
import Modal from '@js/components/modal';
import Form, { Required, StringLength } from 'formvalidation';
import { post, del } from '@js/utils/http';
import Swal from 'sweetalert2';

const { router } = window;

router.on('mount', () => {
    let category_modal_container = document.querySelector('#category_modal');
    const category_form = new Form(category_modal_container.querySelector('form'), {
        fields: {
            name: {
                validators: [new Required(), new StringLength(1, 60)]
            },
            description: {
                validators: [new StringLength(0, 255)]
            },
            // is_published: {},
            // is_seo: {}
        }
    })
        .on('send', e => {
            e.preventDefault();

            post('/api/blog/categories', {
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                },
                body: new URLSearchParams(e.detail)
            })
                .then(res => res.json())
                .then(id => {
                    const new_category = Object.assign({}, e.detail);
                    new_category.id = id;
                    categories.push(new_category);

                    add_category(new_category);

                    category_modal.close();
                })
        });
    let category_modal = new Modal(category_modal_container)
        .on('open', () => {
            category_modal_container
                .querySelector('.modal__dialog__header')
                .innerText = category_to_modify
                    ? `Modifier la catégorie : ${category_to_modify.name}`
                    : 'Créer une catégorie';
            category_modal_container
                .querySelector('button[type=submit]')
                .innerText = category_to_modify ? 'Modifier' : 'Créer';

            if (category_to_modify) {
                category_form.fill(category_to_modify);
            }
        })
        .on('close', () => {
            category_to_modify = null;
            category_form.clear();
        });
    let category_to_modify = null;

    const add_category = (category) => {
        const category_container = document.createElement('li');
        category_container.dataset.id = category_container.id;
        category_container.classList.add('categories__item');

        const category_name = document.createElement('span');
        category_name.innerText = category.name;

        const edit_btn = document.createElement('button');
        edit_btn.innerHTML = ``;
        edit_btn.classList.add('text_blue');
        edit_btn.addEventListener('click', () => edit_category(category));

        const delete_btn = document.createElement('button');
        delete_btn.innerHTML = `<svg class="icon" height="20px">
            <use xlink:href="/dashboard_icons.svg#delete"></use>
        </svg>`;
        delete_btn.classList.add('text_error');
        delete_btn.addEventListener('click', () => delete_category(category_container));

        category_container.appendChild(category_name);
        category_container.appendChild(edit_btn);
        category_container.appendChild(delete_btn);
    }

    const edit_category = category => {
        category_to_modify = category;
        category_modal.open();
    }

    const delete_category = async (category_el, index = null) => {
        const id = category_el.dataset.id;

        let res = await Swal.fire({
            title: 'Suppression',
            text: 'Êtes-vous certain.e de vouloir supprimer cette catégorie ?',
            icon: 'warning',
            showCancelButton: true,
            confirmButtonColor: '#3085d6',
            cancelButtonColor: '#d33',
            confirmButtonText: 'Oui, supprimer',
            cancelButtonText: 'Annuler',
            reverseButtons: true
        });

        if (res.isConfirmed) {
            res = await del(`/api/blog/categories/${id}`);

            if (res.ok) {
                if (index === null) {
                    index = categories.findIndex(category => category.id == id);
                }

                categories.splice(index, 1);
                category_el.remove();
            } else {
                // TODO : erreur survenur
            }
        }
    }

    document
        .querySelector('#btn_add_category')
        .addEventListener('click', () => category_modal.open());

    document
        .querySelectorAll('.categories__item')
        .forEach((item, index) => {
            const category = categories[index];

            item
                .querySelector('.btn_edit_category')
                .addEventListener('click', () => edit_category(category));
            item
                .querySelector('.btn_delete_category')
                .addEventListener('click', () => delete_category(item));
        });
});