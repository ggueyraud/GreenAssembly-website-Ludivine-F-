import 'router';
import Modal from '@js/components/modal';
import Form, { Required, StringLength } from 'formvalidation';
import { post, patch, del } from '@js/utils/http';
import Swal from 'sweetalert2';
import Sortable from 'sortablejs';
import Quill from 'quill';
import { formatDistance } from 'date-fns';
import { fr } from 'date-fns/locale';
import { base64_to_blob } from '../../utils/base642blob';
import { DropZone } from '../../components/assets_grid';

const { router } = window;

let category_to_modify = null;
let article_to_modify = null;
let blocks = [];

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
            is_visible: {},
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
            
            for (const [key, value] of Object.entries(e.detail)) {
                if (article_to_modify) {
                    if (article_to_modify[key] !== value) {
                        // body[key] = value;
                        body.append(key, value);
                    }
                } else {
                    body.append(key, value);
                }
            }

            let i = 0;
            const images = [];

            for (const block of blocks) {
                let old_content = block.content.getContents();
                let content = JSON.parse(JSON.stringify(old_content));
                // const images = [];

                for (const ops of content.ops) {
                    if (ops.insert !== undefined && ops.insert.image !== undefined) {
                        images.push(ops.insert.image);
                        // ops.insert = `[[${images.length - 1}]]`;
                        ops.insert = `[[${i}]]`;
                        i += 1;
                    }
                }

                block.content.setContents(content);
                const formatted_content = block.content.root.innerHTML
                block.content.setContents(old_content);
                console.log(old_content, content)

                block.content = formatted_content;
                // block.pictures = images.map(image => base64_to_blob(image));
                // block.pictures = images;
                console.log(block, images)

                body.append('blocks[]', JSON.stringify(block));
            }

            for (const image of images) {
                body.append('pictures[]', base64_to_blob(image));
            }

            const endpoint = `/api/blog/articles${article_to_modify ? `/${article_to_modify.id}` : ''}`;
            const options = {
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
                    new_article.date = new Date();
                    articles.push(new_article);

                    console.log(new_article)

                    add_article(new_article);
                }

                article_modal.close();
            } catch(e) {
                submit_btn.innerHTML = submit_btn_value_before_send;
                swal_error()
            }

            submit_btn.innerHTML = submit_btn_value_before_send;
            submit_btn.removeAttribute('disabled');
        });
    let category_modal = new Modal(category_modal_container)
        .on('open', () => {
            document.querySelector('[name="name"]').focus();

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
        .on('close', () => {
            category_modal_submit_btn.classList.remove(`btn__${category_to_modify ? 'blue' : 'green'}`);
            category_to_modify = null;
            category_form.clear();
        });
    let article_modal = new Modal(article_modal_container)
        .on('open', () => {
            document.querySelector('[name="title"]').focus();

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
        .on('close', () => {
            article_form_submit_btn.classList.remove(`btn__${article_to_modify ? 'blue' : 'green'}`);
            article_to_modify = null;
            blocks = [];

            article_cover_dropzone.clear();
            article_modal_container.querySelector('#left').innerHTML = '';
            article_modal_container.querySelector('#right').innerHTML = '';

            article_form.clear();
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

    const add_article = ({ id, title, category_id, description, date }) => {
        const container_el = document.createElement('li');
        container_el.dataset.id = id;
        
        const header_el = document.createElement('header');
        const title_id = document.createElement('h3');
        title_id.innerText = title;
        header_el.appendChild(title_id);
        
        if (category_id) {
            const category = categories.find(category => category.id == category_id);

            if (category) {
                const category_el = document.createElement('span');
                category_el.innerText = category.name;
                category_el.classList.add('category');
                header_el.appendChild(category_el);
            }
        }
        container_el.appendChild(header_el);

        if (description) {
            const description_el = document.createElement('p');
            description_el.innerText = description;
            container_el.appendChild(description_el);
        }

        const actions_el = document.createElement('div');
        const edit_btn = document.createElement('button');
        edit_btn.classList.add('text_blue');
        edit_btn.innerHTML = `<svg class="icon">
            <use xlink:href="/dashboard_icons.svg#edit"></use>
        </svg>`;
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
        time.innerText = formatDistance(new Date(date), new Date(), { addSuffix: true, locale: fr });
        container_el.appendChild(time);

        articles_container.prepend(container_el);
    }
    const edit_article = article => {
        article_to_modify = article;
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
    const block_on_end = e => {
        const block_data = blocks[parseInt(e.item.dataset.id)];

        if ((e.from !== e.to || e.newIndex !== e.oldIndex) && block_data) {
            if (e.from === e.to) {
                const up = e.newIndex > e.oldIndex;
    
                blocks
                    .filter(block => {
                        return ((up && block.order <= e.newIndex && block.order > e.oldIndex) || (!up && block.order < e.oldIndex && block.order >= e.newIndex))
                    })
                    .forEach(block => {
                        if (up) {
                            block.order -= 1;
                        } else {
                            block.order += 1;
                        }
                    });
    
                block_data.order = e.newIndex;
            } else {
                const from_right = e.from.id === 'right';
                block_data.left_column = from_right;
                block_data.order = e.newIndex;

                blocks
                    .filter(block => block.left_column === (e.to.id === 'right') && block.order >= e.oldIndex)
                    .forEach(block => {
                        block.order -= 1;
                    });

                blocks
                    .filter(block => block.left_column === (e.to.id === 'left') && block.order >= e.newIndex && block !== block_data)
                    .forEach(block => {
                        block.order += 1;
                    });
            }
            console.log(blocks)
        }
    };
    new Sortable(document.querySelector('#left'), {
        group: 'shared',
        animation: 150,
        onEnd: block_on_end
    });
    new Sortable(document.querySelector('#right'), {
        group: 'shared',
        animation: 150,
        onEnd: block_on_end
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
            const new_block_data = {
                left_column: true,
                order: blocks.filter(block => block.left_column === true).length
            };
            const new_block = document.createElement('div');
            new_block.classList.add('blocks__item');
            new_block.dataset.id = blocks.length;

            const title_label = document.createElement('label');
            title_label.innerText = 'Titre';

            const input = document.createElement('input');
            input.type = 'text';
            input.addEventListener('input', e => {
                new_block_data.title = e.target.value;
            });

            const content_label = document.createElement('label');
            content_label.innerText = 'Contenu';

            const content_editor = document.createElement('div');

            new_block.appendChild(title_label);
            new_block.appendChild(input);
            new_block.appendChild(content_label);
            new_block.appendChild(content_editor);

            blocks.push(new_block_data);
            console.log(blocks)

            document.querySelector('#left').appendChild(new_block);

            const editor = new Quill(content_editor, {
                modules: {
                    toolbar: [
                        [{ list: 'ordered' }, { list: 'bullet' }],
                        ['bold', 'link', 'image', 'clean']
                    ]
                },
                theme: 'snow'
            });
            new_block_data.content = editor;
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
        .querySelectorAll('.articles li')
        .forEach((item, index) => {
            const article = articles[index];

            item
                .querySelector('div > :first-child')
                .addEventListener('click', () => edit_article(article));
            item
                .querySelector('div > :last-child')
                .addEventListener('click', () => delete_article(item, index));

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