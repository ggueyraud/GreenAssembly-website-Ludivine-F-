import 'router';
import Modal from '@js/components/modal';
import Form, { Required, StringLength } from 'formvalidation';
import { get, post, patch, del } from '@js/utils/http';
import Swal from 'sweetalert2';
import Sortable from 'sortablejs';
import Quill from 'quill';
import { formatDistance } from 'date-fns';
import { fr } from 'date-fns/locale';
import { base64_to_blob } from '../../utils/base642blob';
import { DropZone } from '../../components/assets_grid';
import swal_error, { data_removed } from '@js/utils/swal_error';

const { router } = window;

let category_to_modify = null;
let article_to_modify = null;

router.on('mount', () => {
    let category_modal_container = document.querySelector('#category_modal');
    let category_modal_submit_btn = category_modal_container.querySelector('[type="submit"]');
    const category_modal_delete_btn = category_modal_container
        .querySelector('.modal__dialog__footer > :first-child');
    let article_modal_container = document.querySelector('#article_modal');
    const article_modal_delete_btn = article_modal_container
        .querySelector('.modal__dialog__footer > :first-child');
    let article_form_submit_btn = article_modal_container.querySelector('[type="submit"]');
    let categories_container = document.querySelector('.categories .card__body');
    let articles_container = document.querySelector('.articles .card__body');
    const article_cover_dropzone = new DropZone(article_modal_container.querySelector('.drop_zone'));
    
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

            const submit_btn = e.target.querySelector('[type="submit"]');
            const submit_btn_value_before_send = submit_btn.innerHTML;
            submit_btn.setAttribute('disabled', true);
            submit_btn.innerHTML = `<svg class="icon icon--rotate icon--sm mr_2">
                <use xlink:href="/dashboard_icons.svg#redo"></use>
            </svg> Envoi en cours..`;

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

            // If we want to modify a category but no fields has changed so we don't make an update
            if (category_to_modify && Object.entries(body).length === 0) {
                category_modal.close();
                submit_btn.removeAttribute('disabled');
                return;
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

                    if (e.detail.name != category_to_modify.name) {
                        document.querySelector(`[data-id="${category_to_modify.id}"] span`).innerText = e.detail.name;
                        document
                            .querySelectorAll(`.category[data-id="${category_to_modify.id}"]`)
                            .forEach(label => label.innerText = e.detail.name);
                    }
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
                submit_btn.innerHTML = submit_btn_value_before_send;
                swal_error()
            }

            submit_btn.removeAttribute('disabled');
        });
    const article_form = new Form(document.querySelector('[name="article_form"]'), {
        fields: {
            cover: {
                validators: [new Required()],
                container: document.querySelector('#cover_container')
            },
            title: {
                validators: [new Required(), new StringLength(1, 255)]
            },
            category_id: {},
            description: {
                validators: [new StringLength(0, 320)]
            },
            content: {
                validators: [new Required()]
            },
            is_published: {},
            is_seo: {}
        }
    })
        .on('send', async e => {
            let body = new FormData();
            const submit_btn = e.target.querySelector('[type="submit"]');
            const submit_btn_value_before_send = submit_btn.innerHTML;
            submit_btn.setAttribute('disabled', true);
            submit_btn.innerHTML = `<svg class="icon icon--rotate icon--sm mr_2">
                <use xlink:href="/dashboard_icons.svg#redo"></use>
            </svg> Envoi en cours..`;

            e.detail.cover = document.querySelector('[name="cover"]').files[0];
            console.log(e.detail.category_id, e.detail.category_id === '')
            e.detail.category_id = e.detail.category_id === '' ? '' : parseInt(e.detail.category_id);
            
            for (const [key, value] of Object.entries(e.detail)) {
                if (article_to_modify) {
                    if (article_to_modify[key] !== value && value && key !== 'content') {
                        body.append(key, value);
                    }
                } else {
                    if (value && key !== 'content') {
                        body.append(key, value);
                    }
                }
            }

            if (article_to_modify) {
                let i = 0;
                const images = [];

                let old_content = content_quill.getContents();
                let content = JSON.parse(JSON.stringify(old_content));

                const a = article_to_modify.images.map(image => `/uploads/${image.path}`);

                for (const ops of content.ops) {
                    if (
                        ops.insert !== undefined
                        && ops.insert.image !== undefined
                        && !a.includes(ops.insert.image)
                    ) {
                        images.push(ops.insert.image);
                        ops.insert = `[[${i}]]`;
                        i += 1;
                        console.log(ops, 'is a new image');
                    }
                }
                content_quill.setContents(content);
                let formatted_content = content_quill.root.innerHTML
                for (const image of article_to_modify.images) {
                    formatted_content = formatted_content.replace(`<img src="/uploads/${image.path}">`, `[[${image.id}]]`)
                }
                content_quill.setContents(old_content);


                body.append('content', formatted_content);
                console.log(formatted_content);
    
                for (const image of images) {
                    body.append('pictures[]', base64_to_blob(image));
                }
            } else {
                let i = 0;
                const images = [];

                let old_content = content_quill.getContents();
                let content = JSON.parse(JSON.stringify(old_content));

                for (const ops of content.ops) {
                    if (ops.insert !== undefined && ops.insert.image !== undefined) {
                        images.push(ops.insert.image);
                        ops.insert = `[[${i}]]`;
                        i += 1;
                    }
                }
                content_quill.setContents(content);
                const formatted_content = content_quill.root.innerHTML
                content_quill.setContents(old_content);

                body.append('content', formatted_content);
    
                for (const image of images) {
                    body.append('pictures[]', base64_to_blob(image));
                }
            }

            let i  = 0;
            for (const _ of body.entries()) {
                i += 1;
            }

            // If article edit mod and no data has been updated, close modal
            if (article_to_modify && i == 0) {
                article_modal.close();
                submit_btn.removeAttribute('disabled');
                return;
            }

            const endpoint = `/api/blog/articles${article_to_modify ? `/${article_to_modify.id}` : ''}`;
            const options = {
                body
            };

            try {
                if (article_to_modify) {
                    await patch(endpoint, options);                    

                    if (e.detail !== article_to_modify.title) {
                        document
                            .querySelector(`.articles [data-id="${article_to_modify.id}"] h3`)
                            .innerText = e.detail.title;
                    }

                    if (e.detail.category_id && e.detail.category_id != article_to_modify.category_id) {
                        let category_el = document.querySelector(`.articles [data-id="${article_to_modify.id}"] span[data-id="${article_to_modify.category_id}"]`);
                        const category = categories.find(category => category.id == e.detail.category_id);

                        if (category_el) {
                            category_el.innerText = category.name;
                            category_el.dataset.id = category.id;
                        } else {
                            category_el = document.createElement('span');
                            category_el.dataset.id = category.id;
                            category_el.innerText = category.name;
                            category_el.classList.add('category');
                            document
                                .querySelector(`.articles [data-id="${article_to_modify.id}"] header`)
                                .insertAdjacentElement('beforeend', category_el);
                        }
                    }

                    if (e.detail.description !== article_to_modify.description) {
                        let description_el = document.querySelector(`.articles [data-id="${article_to_modify.id}"] p`);

                        if (description_el) {
                            description_el.innerText = e.detail.description;
                        } else {
                            description_el = document.createElement('p');
                            description_el.innerText = e.detail.description;
                            document
                                .querySelector(`.articles [data-id="${article_to_modify.id}"] header`)
                                .insertAdjacentElement('afterend', description_el);
                        }
                    }
                } else {
                    const res = await post(endpoint, options);
                    const id = await res.json();

                    const new_article = Object.assign({}, e.detail);
                    new_article.id = id;
                    new_article.date = new Date();
                    articles.push(new_article);

                    add_article(new_article);
                }

                article_modal.close();
            } catch(e) {
                console.log(e)
                submit_btn.innerHTML = submit_btn_value_before_send;
                swal_error()
            }

            submit_btn.innerHTML = submit_btn_value_before_send;
            submit_btn.removeAttribute('disabled');
        });
    let category_modal = new Modal(category_modal_container)
        .on('beforeOpen', async () => {
            if (category_to_modify) {
                try {
                    let res = await get(`/api/blog/categories/${category_to_modify.id}`);
                    Object.assign(category_to_modify, await res.json());
                } catch (e) {
                    // Already removed from another user or in another tab
                    if (e.status === 404) {
                        data_removed(category_to_modify.name);

                        const index = categories.findIndex(category => category.id === category_to_modify.id);
                        if (index !== -1) {
                            categories.splice(index, 1);
                        }

                        document
                            .querySelector(`.categories [data-id="${category_to_modify.id}"]`)
                            .remove();
    
                        // Prevent the modal to be opened
                        return false;
                    }
                }
            }

            if (category_to_modify) {
                category_modal_delete_btn.style.removeProperty('display');
            } else {
                category_modal_delete_btn.style.setProperty('display', 'none', 'important');
            }

            category_modal_container
                .querySelector('.modal__dialog__header__title')
                .innerText = category_to_modify
                    ? `Modifier la catégorie : ${category_to_modify.name}`
                    : 'Créer une catégorie';
            
            category_modal_submit_btn.innerText = category_to_modify ? 'Modifier' : 'Créer';
            category_modal_submit_btn.classList.add(`btn__${category_to_modify ? 'blue' : 'green'}`);
            
            if (category_to_modify) {
                category_form.fill(category_to_modify);
            }
        })
        .on('open', () => document.querySelector('[name="name"]').focus())
        .on('close', () => {
            category_modal_submit_btn.classList.remove(`btn__${category_to_modify ? 'blue' : 'green'}`);
            category_to_modify = null;
            category_form.clear();
        });
    category_modal_delete_btn
        .addEventListener('click', () => {
            delete_category(document.querySelector(`.categories li[data-id="${category_to_modify.id}"]`));
        });

    const content_label = document.querySelector('[for="content"]');
    const content_quill = new Quill(document.querySelector('#content_editor'), {
        modules: {
            toolbar: [
                [{ list: 'ordered' }, { list: 'bullet' }],
                ['bold', 'link', 'image', 'clean']
            ]
        },
        theme: 'snow'
    });
    const content_input = document.querySelector('[name="content"]');
    content_quill.on('text-change', () => {
        content_input.value = content_quill.getText().length === 1
            ? ''
            : content_quill.root.innerHTML;

        content_input.dispatchEvent(new Event('input'));
    });
    content_label.addEventListener('click', () => content_quill.focus());

    let article_modal = new Modal(article_modal_container)
        .on('beforeOpen', async () => {
            if (article_to_modify) {
                article_form.remove_field('cover');

                try {
                    let res = await get(`/api/blog/articles/${article_to_modify.id}`);
                    Object.assign(article_to_modify, await res.json());
                } catch (e) {
                    // Already removed from another user or in another tab
                    if (e.status === 404) {
                        data_removed(article_to_modify.title);

                        const index = articles.findIndex(article => article.id === article_to_modify.id);

                        if (index !== -1) {
                            articles.splice(index, 1);
                        }

                        document
                            .querySelector(`.articles [data-id="${article_to_modify.id}"]`)
                            .remove();
    
                        // Prevent the modal to be opened
                        return false;
                    }
                }

                // TODO : set cover
                article_cover_dropzone.setImage(`/uploads/${article_to_modify.cover}`);

                console.log(article_to_modify)
                for (const image of article_to_modify.images) {
                    article_to_modify.content = article_to_modify.content.replace(`[[${image.id}]]`, `<img src="/uploads/${image.path}" />`);
                }

                content_quill.root.innerHTML = article_to_modify.content;
                console.log(content_quill.getContents());
            }

            if (article_to_modify) {
                article_modal_delete_btn.style.removeProperty('display');
            } else {
                article_modal_delete_btn.style.setProperty('display', 'none', 'important');
            }

            article_modal_container
                .querySelector('.modal__dialog__header__title')
                .innerText = article_to_modify
                    ? `Modifier l'article : ${article_to_modify.title}`
                    : 'Créer un article';
    
            article_form_submit_btn.innerText = article_to_modify ? 'Modifier' : 'Créer';
            article_form_submit_btn.classList.add(`btn__${article_to_modify ? 'blue' : 'green'}`);
    
            if (article_to_modify) {
                article_form.fill(article_to_modify);
            }
        })
        .on('open', () => document.querySelector('[name="title"]').focus())
        .on('close', () => {
            content_quill.root.innerHTML = '';
            article_form_submit_btn.classList.remove(`btn__${article_to_modify ? 'blue' : 'green'}`);
            article_to_modify = null;

            article_cover_dropzone.clear();
            // article_modal_container.querySelector('#left').innerHTML = '';
            // article_modal_container.querySelector('#right').innerHTML = '';

            if (article_to_modify) {
                article_form.add_field('cover', {
                    validators: [new Required()],
                    container: document.querySelector('#cover_container')
                });
            }
            article_form.clear();
        });
    article_modal_delete_btn
        .addEventListener('click', () => {
            delete_article(document.querySelector(`.articles li[data-id="${article_to_modify.id}"]`));
        });

    const add_category = (category) => {
        const category_container = document.createElement('li');
        category_container.dataset.id = category.id;
        category_container.classList.add('categories__item');

        const category_name = document.createElement('span');
        category_name.innerText = category.name;

        const edit_btn = document.createElement('button');
        edit_btn.innerHTML = `<svg class="icon icon--sm">
            <use xlink:href="/dashboard_icons.svg#edit"></use>
        </svg>`;
        edit_btn.classList.add('text_blue');
        edit_btn.addEventListener('click', () => edit_category(category));

        const delete_btn = document.createElement('button');
        delete_btn.innerHTML = `<svg class="icon icon--sm">
            <use xlink:href="/dashboard_icons.svg#delete"></use>
        </svg>`;
        delete_btn.classList.add('text_error');
        delete_btn.addEventListener('click', () => delete_category(category_container));

        category_container.appendChild(category_name);
        category_container.appendChild(edit_btn);
        category_container.appendChild(delete_btn);

        // Add category to select
        const option = document.createElement('option');
        option.setAttribute('value', category.id);
        option.innerText = category.name;
        document.querySelector('#category_id').appendChild(option);

        categories_container.appendChild(category_container);
    }
    const edit_category = category => {
        category_to_modify = Object.assign({}, category);
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

                // Remove the category on the select of the article form
                document
                    .querySelector(`#category_id [value="${id}"]`)
                    .remove();
                
                // Remove all tag on articles
                document
                    .querySelectorAll(`.category[data-id="${id}"]`)
                    .forEach(category_tag => category_tag.remove());

                category_el.remove();

                // If removed from category modal, close the modal
                if (category_to_modify) {
                    category_modal.close();
                }
            } else {
                swal_error()
            }
        }
    }

    const add_article = article => {
        const container_el = document.createElement('li');
        container_el.dataset.id = article.id;
        
        const header_el = document.createElement('header');
        const title_id = document.createElement('h3');
        title_id.innerText = article.title;
        header_el.appendChild(title_id);
        
        if (article.category_id) {
            const category = categories.find(category => category.id == article.category_id);

            if (category) {
                const category_el = document.createElement('span');
                category_el.dataset.id = article.category_id;
                category_el.innerText = category.name;
                category_el.classList.add('category');
                header_el.appendChild(category_el);
            }
        }
        container_el.appendChild(header_el);

        if (article.description) {
            const description_el = document.createElement('p');
            description_el.innerText = article.description;
            container_el.appendChild(description_el);
        }

        const actions_el = document.createElement('div');
        const edit_btn = document.createElement('button');
        edit_btn.classList.add('text_blue');
        edit_btn.innerHTML = `<svg class="icon">
            <use xlink:href="/dashboard_icons.svg#edit"></use>
        </svg>`;
        edit_btn.addEventListener('click', () => edit_article(article));
        const delete_btn = document.createElement('button');
        delete_btn.classList.add('text_error');
        delete_btn.innerHTML = `<svg class="icon">
            <use xlink:href="/dashboard_icons.svg#delete"></use>
        </svg>`;
        delete_btn.addEventListener('click', () => delete_article(container_el));
        actions_el.appendChild(edit_btn);
        actions_el.appendChild(delete_btn);
        container_el.appendChild(actions_el);

        const time = document.createElement('time');
        time.innerText = formatDistance(new Date(article.date), new Date(), { addSuffix: true, locale: fr });
        container_el.appendChild(time);

        articles_container.prepend(container_el);
    }
    const edit_article = article => {
        article_to_modify = Object.assign({}, article);
        article_modal.open();
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

                if (article_to_modify) {
                    article_modal.close();
                }
            } else {
                swal_error()
            }
        }
    }

    // new Sortable(categories_container, {
    //     animation: 150,
    //     onEnd: e => {
    //         if (e.newIndex !== e.oldIndex) {
    //             patch(`/api/blog/categories/${e.item.dataset.id}`, {
    //                 headers: {
    //                     'Content-Type': 'application/json'
    //                 },
    //                 body: { order: parseInt(e.newIndex + 1) }
    //             })
    //                 .catch(swal_error)
    //         }
    //     }
    // });

    document
        .querySelector('.categories .card__header button')
        .addEventListener('click', () => category_modal.open());
    document
        .querySelector('.articles .card__header button')
        .addEventListener('click', () => article_modal.open());

    document
        .querySelectorAll('.categories li')
        .forEach((item, index) => {
            const category = categories[index];

            item
                .querySelector('div > :first-child')
                .addEventListener('click', () => edit_category(category));
            item
                .querySelector('div > :last-child')
                .addEventListener('click', () => delete_category(item, index));
        });

    document
        .querySelectorAll('.articles li')
        .forEach((item, index) => {
            const article = articles[index];

            item
                .querySelector('div > :first-child')
                .addEventListener('click', () => edit_article(article));
            item
                .querySelector('div > :last-child')
                .addEventListener('click', () => delete_article(item, index));

            // Fill the article time tag with formatted date
            const time = item.querySelector('time');
            time.innerText = formatDistance(new Date(time.getAttribute('datetime')), new Date(), { addSuffix: true, locale: fr });

            if (article.category_id) {
                const category = categories.find(category => category.id === article.category_id);
                
                if (category) {
                    item
                        .querySelector('header > .category')
                        .innerText = category.name;
                }
            }
        });
});