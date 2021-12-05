import 'router';
import Modal from '@js/components/modal';
import Form, { Required, StringLength } from 'formvalidation';
import { post, patch, del } from '@js/utils/http';
import Swal from 'sweetalert2';
import Sortable from 'sortablejs';
import Quill from 'quill';

const { router } = window;

const swal_error = () => Swal.fire({
    title: 'Une erreur est survenue',
    text: 'Si le problème persiste veuillez contacter la personne en charge de la maintenance de votre site-web.',
    icon: 'error',
    footer: `<a href="https://greenassembly.fr/contact" target="_blank">Contacter l'agence GreenAssembly</a>`
});

router.on('mount', () => {
    let category_modal_container = document.querySelector('#category_modal');
    let category_modal_submit_btn = category_modal_container.querySelector('[type="submit"]');

    let article_modal_container = document.querySelector('#article_modal');
    let categories_container = document.querySelector('.categories');
    let category_to_modify = null;
    let article_to_modify = null;
    const category_form = new Form(document.querySelector('[name="category_form"]'), {
        fields: {
            name: {
                validators: [new Required(), new StringLength(1, 60)]
            },
            description: {
                validators: [new StringLength(0, 255)]
            },
            is_visible: {},
            is_seo: {}
        }
    })
        .on('send', async e => {
            e.preventDefault();

            let body = {};

            if (category_to_modify) {
                for (const [key, value] of Object.entries(e.detail)) {
                    if (category_to_modify[key] !== value) {
                        body[key] = value;
                    }
                }
            } else {
                body = e.detail;
            }

            const endpoint = `/api/blog/categories${category_to_modify ? `/${category_to_modify.id}` : ''}`;
            const options = {
                headers: {
                    'Content-Type': 'application/json',
                },
                body
            };

            try {
                if (category_to_modify) {
                    await patch(endpoint, options);

                    document.querySelector(`[data-id="${category_to_modify.id}"] span`).innerText = e.detail.name;
                    Object.assign(category_to_modify, e.detail);
                } else {
                    const res = await post(endpoint, options);
                    const id = await res.json();

                    const new_category = Object.assign({}, e.detail);
                    new_category.id = id;
                    categories.push(new_category);

                    add_category(new_category);
                }

                category_modal.close();
            } catch {
                swal_error()
            }
        });
    const article_form = new Form(document.querySelector('[name="article_form"]'), {
        fields: {
            title: {
                validators: [new Required(), new StringLength(1, 255)]
            },
            description: {
                validators: [new StringLength(0, 320)]
            },
            is_visible: {},
            is_seo: {}
        }
    })
        .on('send', async e => {
            let body = {};
            
            if (article_to_modify) {
                for (const [key, value] of Object.entries(e.detail)) {
                    if (article_to_modify[key] !== value) {
                        body[key] = value;
                    }
                }
            } else {
                body = e.detail;
            }

            const endpoint = `/api/blog/articles${article_to_modify ? `/${article_to_modify.id}` : ''}`;
            const options = {
                headers: {
                    'Content-Type': 'application/json'
                },
                body
            };

            try {
                if (article_to_modify) {
                    await patch(endpoint, options);

                    // TODO : update article dom
                } else {
                    const res = await post(endpoint, options);
                    const id = await res.json();

                    const new_article = Object.assign({}, e.detail);
                    new_article.id = id;
                    articles.push(new_article);

                    add_article(new_article);
                }

                article_modal.close();
            } catch {
                swal_error()
            }
        });
    let category_modal = new Modal(category_modal_container)
        .on('open', () => {
            category_modal_container
                .querySelector('.modal__dialog__header')
                .innerText = category_to_modify
                    ? `Modifier la catégorie : ${category_to_modify.name}`
                    : 'Créer une catégorie';

            category_modal_submit_btn.innerText = category_to_modify ? 'Modifier' : 'Créer';
            category_modal_submit_btn.classList.add(`btn__${category_to_modify ? 'blue' : 'green'}`);

            if (category_to_modify) {
                category_form.fill(category_to_modify);
            }
        })
        .on('close', () => {
            category_modal_submit_btn.classList.remove(`btn__${category_to_modify ? 'blue' : 'green'}`);
            category_to_modify = null;
            category_form.clear();
        });
    let article_modal = new Modal(article_modal_container)
        .on('open', () => {
            article_modal_container
                .querySelector('.modal__dialog__header')
                .innerText = article_to_modify
                    ? `Modifier l'article : ${article_to_modify.title}`
                    : 'Créer un article';

            if (article_to_modify) {
                article_form.fill(article_to_modify);
            }
        })
        .on('close', () => {
            article_to_modify = null;
            article_form.clear();
        });

    const add_category = (category) => {
        const category_container = document.createElement('li');
        category_container.dataset.id = category.id;
        category_container.classList.add('categories__item');

        const category_name = document.createElement('span');
        category_name.innerText = category.name;

        const edit_btn = document.createElement('button');
        edit_btn.innerHTML = `<svg class="icon" height="20px">
            <use xlink:href="/dashboard_icons.svg#edit"></use>
        </svg>`;
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

        categories_container.appendChild(category_container);
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
                swal_error()
            }
        }
    }

    const delete_article = async (article_el, index = null) => {
        const id = article_el.dataset.id;

        let res = await Swal.fire({
            title: 'Suppression',
            text: 'Êtes-vous certain.e de vouloir supprimer cet article ?',
            icon: 'warning',
            showCancelButton: true,
            confirmButtonColor: '#3085d6',
            cancelButtonColor: '#d33',
            confirmButtonText: 'Oui, supprimer',
            cancelButtonText: 'Annuler',
            reverseButtons: true
        });

        if (res.isConfirmed) {
            res = await del(`/api/blog/articles/${id}`);

            if (res.ok) {
                if (index === null) {
                    index = articles.findIndex(article => article.id == id);
                }

                articles.splice(index, 1);
                article_el.remove();
            } else {
                swal_error()
            }
        }
    }

    new Sortable(categories_container, {
        animation: 150,
        onEnd: e => {
            if (e.newIndex !== e.oldIndex) {
                patch(`/api/blog/categories/${e.item.dataset.id}`, {
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: { order: parseInt(e.newIndex + 1) }
                })
                    .catch(swal_error)
            }
        }
    });
    new Sortable(document.querySelector('#left'), {
        group: 'shared',
        animation: 150
    });
    new Sortable(document.querySelector('#right'), {
        group: 'shared',
        animation: 150
    });

    document
        .querySelector('#btn_add_category')
        .addEventListener('click', () => category_modal.open());
    document
        .querySelector('#btn_add_article')
        .addEventListener('click', () => article_modal.open());
    document
        .querySelector('#add_block button')
        .addEventListener('click', () => {
            const new_block = document.createElement('div');
            new_block.classList.add('blocks__item');

            const title_label = document.createElement('label');
            title_label.innerText = 'Titre';

            const input = document.createElement('input');
            input.type = 'text';

            const content_label = document.createElement('label');
            content_label.innerText = 'Contenu';

            const content_editor = document.createElement('div');

            new_block.appendChild(title_label);
            new_block.appendChild(input);
            new_block.appendChild(content_label);
            new_block.appendChild(content_editor);

            document.querySelector('#left').appendChild(new_block);

            new Quill(content_editor, {
                modules: {
                    toolbar: [
                        [{ header: [2, 3, false] }],
                        [{ list: 'ordered' }, { list: 'bullet' }],
                        ['bold', 'link', 'clean']
                    ]
                },
                theme: 'snow'
            });
        });

    document
        .querySelectorAll('.categories__item')
        .forEach((item, index) => {
            const category = categories[index];

            item
                .querySelector('.btn_edit_category')
                .addEventListener('click', () => edit_category(category));
            item
                .querySelector('.btn_delete_category')
                .addEventListener('click', () => delete_category(item, index));
        });

    document
        .querySelectorAll('.articles__item')
        .forEach((item, index) => {
            const article = articles[index];

            item
                .querySelector('div > :last-child')
                .addEventListener('click', () => delete_article(item, index));
        });
});