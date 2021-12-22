import Swal from 'sweetalert2';
import Sortable from 'sortablejs';
import Form, { Required, StringLength } from 'formvalidation';
import Quill from 'quill';
import { get, post, patch, put, del } from '@js/utils/http';
import '@js/components/modal';
import Modal from '../../components/modal';
import DOMPurify from 'dompurify'
import { formatDistance } from 'date-fns'
import { fr } from 'date-fns/locale';
import swal_error, { data_removed } from '@js/utils/swal_error';
import Cropper from 'cropperjs';
import AssetsGrid, { is_filled_class } from '@js/components/assets_grid';
import 'router';

let project_fv = null;
let editor = null;
let add_project_modal = null;
let project_to_modify = null;
const { router } = window;

const delete_project = (el, id) => Swal.fire({
    title: 'Suppression',
    text: 'Êtes-vous certain.e de vouloir supprimer ce projet ?',
    icon: 'warning',
    showCancelButton: true,
    confirmButtonColor: '#3085d6',
    cancelButtonColor: '#d33',
    confirmButtonText: 'Oui, supprimer',
    cancelButtonText: 'Annuler',
    reverseButtons: true
})
    .then(res => {
        if (res.isConfirmed) {
            del(`/api/portfolio/projects/${id}`)
                .then(() => el.remove())
                .catch(swal_error)
        }
    });

router.on('mount', () => {
    let categories_container = null;
    let projects_container = null;
    let sortable_categories = null;

    const swal_error = () => Swal.fire({
        title: 'Une erreur est survenue',
        text: 'Si le problème persiste veuillez contacter la personne en charge de la maintenance de votre site-web.',
        icon: 'error',
        footer: "<a href=\"https://greenassembly.fr/contact\" target=\"_blank\">Contacter l'agence GreenAssembly</a>"
    });

    const init_category_events = (el, span = null, input = null, button = null) => {
        const id = el.dataset.id;

        if (!span) {
            span = el.querySelector('span');
        }

        if (!input) {
            input = el.querySelector('input');
        }

        if (!button) {
            button = el.querySelector('button');
        }

        span.addEventListener('click', () => {
            sortable_categories.option('disabled', true);
            input.value = span.innerText;
            el.classList.add('categories__item--edition');
            input.focus();
            input.setSelectionRange(0, input.value.length);
        });

        input.addEventListener('blur', () => {
            sortable_categories.option('disabled', false);
            el.classList.remove('categories__item--edition');
        });
        input.addEventListener('keydown', e => {
            const value = e.target.value;
            const { name } = categories.find(category => category.id == id);

            if (e.key === 'Enter' && value) {
                // Don't update if value hasn't changed
                if (name != value) {
                    put(`/api/portfolio/categories/${id}`, {
                        headers: {
                            'Content-Type': 'application/x-www-form-urlencoded'
                        },
                        body: new URLSearchParams({ name: value })
                    })
                        .then(() => {
                            el.classList.remove('categories__item--edition');
    
                            const checkbox_container = document.querySelector(`[name="categories"][value="${id}"]`).parentElement;
                            checkbox_container.innerHTML = '';
                            checkbox_container.textContent = value;
                            const checkbox_appearence = document.createElement('span');
                            checkbox_container.prepend(checkbox_appearence);
                            const checkbox = document.createElement('input');
                            checkbox.type = 'checkbox';
                            checkbox.value = id;
                            checkbox.name = 'categories';
                            checkbox_container.prepend(checkbox);
                            // document.querySelector(`[name="categories"][value="${id}"]`).parentElement.innerText = value;
    
                            span.innerText = value;
                        })
                        .catch(swal_error)
                } else {
                    el.classList.remove('categories__item--edition');
                }
            }
        });

        button.addEventListener('click', () => {
            Swal.fire({
                title: 'Suppression',
                text: 'Êtes-vous certain.e de vouloir supprimer cette catégorie ?',
                icon: 'warning',
                showCancelButton: true,
                confirmButtonColor: '#3085d6',
                cancelButtonColor: '#d33',
                confirmButtonText: 'Oui, supprimer',
                cancelButtonText: 'Annuler',
                reverseButtons: true
            })
                .then(res => {
                    if (res.isConfirmed) {
                        del(`/api/portfolio/categories/${id}`)
                            .then(() => {
                                categories_container.querySelector(`[data-id="${id}"]`).remove();
                                // console.log(document.querySelector(`.categories[type="${id}"]`))
                                console.log(document.querySelector(`[name="categories"][value="${id}"]`));
                                document.querySelector(`[name="categories"][value="${id}"]`).parentElement.remove();
                            })
                            .catch(swal_error)
                    }
                });
        });
    }

    const add_category = (id, name) => {
        const category = document.createElement('li');
        category.dataset.id = id;
        category.classList.add('categories__item');
        
        const input = document.createElement('input');
        input.type = 'text';
        input.maxLength = 30;

        const category_name = document.createElement('span');
        category_name.innerText = name;
        category_name.classList.add('categories__item__name');

        const delete_btn = document.createElement('button');
        delete_btn.innerHTML = `<svg class="icon icon--sm">
            <use xlink:href="/dashboard_icons.svg#delete"></use>
        </svg>`;
        delete_btn.classList.add('text_error');

        
        category.appendChild(category_name);
        category.appendChild(input);
        category.appendChild(delete_btn);
        
        categories_container.appendChild(category);

        init_category_events(category, category_name, input, delete_btn);
    }

    document
        .querySelectorAll('.categories__item')
        .forEach(item => init_category_events(item));

    document
        .querySelectorAll('.projects__item')
        .forEach(item => {
            const project = projects.find(project => project.id == item.dataset.id);

            // const content_el = item.querySelector('.projects__item__content');
            // content_el.innerHTML = DOMPurify.sanitize(content_el.innerHTML, {
            //     ALLOWED_TAGS: []
            // });

            // Update button
            item
                .querySelector('.projects__item__actions .text_blue')
                .addEventListener('click', () => {
                    project_to_modify = project;
                    add_project_modal.open();
                });

            // Delete button
            item.querySelector('.projects__item__actions .text_error').addEventListener('click', () => delete_project());

            const time = item.querySelector('time');
            time.innerText = formatDistance(new Date(time.getAttribute('datetime')), new Date(), { addSuffix: true, locale: fr });
        });

    const add_project = ({id, name, description, content, date}, after = true) => {
        const project = document.createElement('div');
        project.classList.add('projects__item');

        const project_title = document.createElement('div');
        project_title.classList.add('projects__item__title');
        project_title.innerText = name;

        const project_content = document.createElement('div');
        project_content.classList.add('projects__item__content');
        project_content.innerText = DOMPurify.sanitize(content, {
            ALLOWED_TAGS: []
        });

        const project_actions = document.createElement('div');
        project_actions.classList.add('projects__item__actions');

        const update_btn = document.createElement('button');
        update_btn.classList.add('text_blue');
        update_btn.innerHTML = `<svg class="icon" height="20px">
            <use xlink:href="/dashboard_icons.svg#edit"></use>
        </svg>`;
        update_btn.addEventListener('click', () => {
            project_to_modify = project;
            add_project_modal.open();
        });

        const delete_btn = document.createElement('button');
        delete_btn.classList.add('text_error');
        delete_btn.innerHTML = `<svg class="icon" height="20px">
            <use xlink:href="/dashboard_icons.svg#delete"></use>
        </svg>`;
        delete_btn.addEventListener('click', () => delete_project());

        project_actions.appendChild(update_btn);
        project_actions.appendChild(delete_btn);

        const project_footer = document.createElement('div');
        project_footer.classList.add('projects__item__footer');
        // project_footer.innerText = formatRelative(, { locale: fr });
        project_footer.innerText = formatDistance(new Date(date), new Date(), { addSuffix: true, locale: fr });

        project.appendChild(project_title);
        project.appendChild(project_content);
        project.appendChild(project_actions);
        project.appendChild(project_footer);

        if (after) {
            projects_container.appendChild(project);
        } else {
            projects_container.prepend(project)
        }
    }

    // Categories
    categories_container = document.querySelector('#categories .card__body');
    projects_container = document.querySelector('.projects');
    
    const new_category_input = document.querySelector('[name=new_category_name]');
    
    sortable_categories = new Sortable(categories_container, {
        animation: 150,
        onEnd: e => {
            if (e.newIndex !== e.oldIndex) {
                put(`/api/portfolio/categories/${e.item.dataset.id}`, {
                    headers: {
                        'Content-Type': 'application/x-www-form-urlencoded'
                    },
                    body: new URLSearchParams({ order: parseInt(e.newIndex + 1) })
                })
                    .catch(swal_error)
            }
        }
    });
    
    new_category_input
        .addEventListener('keydown', e => {
            const value = e.target.value;
            
            if (e.which === 13 && value) {
                post('/api/portfolio/categories', {
                    headers: {
                        'Content-Type': 'application/x-www-form-urlencoded'
                    },
                    body: new URLSearchParams({ name: value })
                })
                    .then(async res => {
                        const id = await res.json();

                        // Create new checkbox in project modal
                        const checkbox_container = document.createElement('label');
                        checkbox_container.classList.add('checkbox', 'mb_0');
                        checkbox_container.textContent = value;
                        const checkbox_appearence = document.createElement('span');
                        checkbox_container.prepend(checkbox_appearence);
                        const checkbox = document.createElement('input');
                        checkbox.type = 'checkbox';
                        checkbox.value = id;
                        checkbox.name = 'categories';
                        // checkbox_container.prep
                        checkbox_container.prepend(checkbox);
                        document
                            .querySelector('[for="categories"]')
                            .nextElementSibling
                            .appendChild(checkbox_container);

                        project_fv.remove_field('categories[]');
                        project_fv.add_field('categories[]');
    
                        // Add category in categories column
                        add_category(id, value);

                        new_category_input.value = '';
                    })
                    .catch(swal_error)
            }
        });
    
    // Projects
    const project_form = document.querySelector('[name=create_project]');
    const project_form_submit = project_form.querySelector('button[type=submit]');
    const reset_submit_btn = (submit_value) => {
        project_form_submit.removeAttribute('disabled');
        project_form_submit.innerHTML = submit_value;
    }
    
    project_fv = new Form(project_form, {
        fields: {
            name: {
                validators: [new Required(), new StringLength(1, 120)]
            },
            description: {
                validators: [new StringLength(0, 320)]
            },
            content: {
                validators: [new Required(), new StringLength(30, 1000)]
            },
            'categories[]': {}
        }
    })
        .on('send', async e => {
            const submit_value = project_form_submit.innerHTML;
            project_form_submit.setAttribute('disabled', true);
            project_form_submit.innerHTML = `<svg class="icon icon--rotate icon--sm mr_2" height="18px">
                <use xlink:href="/dashboard_icons.svg#redo"></use>
            </svg> Envoi en cours..`;
    
            const body = new FormData();

            for (const [key, value] of Object.entries(e.detail)) {
                if ((project_to_modify && project_to_modify[key] !== value) || !project_to_modify) {
                    if (Array.isArray(value)) {
                        let key_array = `${key}[]`;

                        for (const item of value) {
                            body.append(key_array, item);
                        }
                    } else {
                        body.append(key, value);
                    }
                }

                if (key === 'categories' && project_to_modify && project_to_modify.categories != e.detail.categories) {
                    if (e.detail.categories.length === 0) body.append('categories[]', '');
                }
            }

            if (project_to_modify) {
                // console.log(project_to_modify.assets)
                console.log('--- Grid values ---');
                console.log(assets_grid.value);
                console.log('-------------------');

                for (const asset of project_to_modify.assets) {
                    if (!assets_grid.value.includes(`/uploads/${asset.path}`)) {
                        console.log(asset, 'removed')
                        asset.to_delete = true;
                    }

                    asset.order = assets_grid.value.indexOf(`/uploads/${asset.path}`);
                }



                console.log(project_to_modify.assets)

                // for (const asset of assets_grid.value) {
                //     const existing_asset = project_to_modify.assets.find(existing_asset => existing_asset.path === `/uploads/${asset}`);
                
                //     if ()
                // }

                return;
                let i = 0;
                for (const _ of body.entries()) {
                    i += 1;
                }

                // No changes detected
                if (i === 0) {
                    reset_submit_btn(submit_value);
    
                    add_project_modal.close();

                    return;
                }
            } else {
                assets_grid.value.forEach(img => body.append('files[]', img));
            }

            try {
                if (project_to_modify) {
                    await patch(`/api/portfolio/projects/${project_to_modify.id}`, {
                        body
                    });

                    if (e.detail.name !== project_to_modify.name) {
                        document.querySelector(`#projects [data-id="${project_to_modify.id}"] span`).innerText = e.detail.name;
                    }

                    if (e.detail.description !== project_to_modify.description) {
                        console.log(e.detail.description)
                        let description = document.querySelector(`#projects [data-id="${project_to_modify.id}"] p`);

                        if (e.detail.description) {
                            if (!description) {
                                description = document.createElement('p');
                                document
                                    .querySelector(`#projects [data-id="${project_to_modify.id}"] span`)
                                    .insertAdjacentElement('afterend', description);
                            }

                            description.innerText = e.detail.description;
                        } else {
                            if (description) {
                                description.remove();
                            }
                        }
                    }
                } else {
                    const res = await post('/api/portfolio/projects', {
                        body
                    })

                    const id = await res.json();
                    const { name, content } = e.detail;
                    
                    add_project({ id, name, content, date: new Date() }, false);
                }

                reset_submit_btn(submit_value);
                add_project_modal.close();
            } catch (e) {
                console.log(e)
                reset_submit_btn(submit_value);
                swal_error();
            }
        });
    
    editor = new Quill('#content_editor', {
        modules: {
            toolbar: [
                [{ header: [2, 3, false] }],
                [{ list: 'ordered' }, { list: 'bullet' }],
                ['bold', 'link', 'clean']
            ]
        },
        theme: 'snow'
    });
    editor.on('text-change', () => {
        const value = editor.getText().length === 1 ? '' : editor.root.innerHTML;
        const content = document.querySelector('[name=content]');
    
        content.value = DOMPurify.sanitize(value, {
            ALLOWED_TAGS: ['p', 'b', 'h2', 'h3', 'ul', 'ol', 'li', 'a']
        });
    
        content.dispatchEvent(new Event('input'));
    })
    
    const cropper_el = document.querySelector('#cropper');
    let cropper = null;
    const assets_grid = new AssetsGrid(document.querySelector('.assets'));
    assets_grid
        .on('select', (_, image, img) => {
            img.container.classList.remove(is_filled_class);
            cropper_el.setAttribute('src', image);
            window.img2change = img;
        
            if (cropper) {
                cropper.replace(image);
            }
        
            asset_editor_modal.open();
        });
        // .on('remove', (_, dropzone) => {
        //     const asset = project_to_modify.assets.find(asset => `/uploads/${asset.path}` === dropzone.blob);

        //     if (asset) {
        //         asset.to_delete = true;
        //     }
        // })
        // .on('move', (_, ));
    
    const rotate_selector = document.querySelector('#rotate');
    const rotate_input = rotate_selector.nextElementSibling;
    rotate_input.addEventListener('input', e => {
        let value = parseInt(e.target.value);
        
        if (value > 360) {
            value = 360;
        } else if (value < 0) {
            value = 0;
        }
    
        rotate_input.value = value;
        cropper.rotateTo(value);
    });
    
    const valid_crop_btn = document.querySelector('#valid_crop');
    const asset_editor_modal = new Modal(document.querySelector('#asset_editor_modal'));
    asset_editor_modal.on('open', () => {
        valid_crop_btn.focus();
        if (!cropper) {
            cropper = new Cropper(cropper_el, {
                minCropBoxWidth: 320,
                zoom(e) {
                    if (e.detail.ratio <= 0.5 || e.detail.ratio >= 2.5) {
                        e.preventDefault()
                    }
                }
            });
        }
    });
    asset_editor_modal.on('close', () => {
        rotate_selector.value = 0;
        rotate_input.value = 0;
    });
    
    rotate_selector.addEventListener('input', e => {
        const value = parseInt(e.target.value);
    
        rotate_input.value = value;
        cropper.rotateTo(value);
    });

    add_project_modal = new Modal(document.querySelector('#add_project_modal'))
        .on('beforeOpen', async (modal) => {
            if (project_to_modify) {
                try {
                    let res = await get(`/api/portfolio/projects/${project_to_modify.id}`);
                    Object.assign(project_to_modify, await res.json());
                } catch (e) {
                    if (e.status === 404) {
                        data_remove(project_to_modify.name);

                        // TODO : remove project
                    } else {
                        swal_error()
                    }

                    return false;
                }

                editor.root.innerHTML = project_to_modify.content;
                assets_grid.setImages(project_to_modify.assets.map(asset => `/uploads/${asset.path}`));

                modal.modal.querySelector('.modal__dialog__footer > :first-child').classList.remove('hidden');
                modal.modal.querySelector('.modal__dialog__footer > :last-child').innerText = project_to_modify
                    ? `Modifier`
                    : 'Créer';
            } else {}

            modal.modal.querySelector('.modal__dialog__header__title')
                .innerText = project_to_modify
                    ? `Modifier le project : ${project_to_modify.name}`
                    : 'Créer un projet';

            if (project_to_modify) {
                project_to_modify['categories[]'] = project_to_modify.categories;
                project_fv.fill(project_to_modify);
            }
        })
        .on('open', e => e.modal.querySelector('[name="name"]').focus())
        .on('close', modal => {
            if (project_to_modify) {
                modal.modal.querySelector('.modal__dialog__footer > :first-child').classList.add('hidden');
            }

            project_to_modify = null;
            assets_grid.clear();
            editor.setContents([]);
            project_fv.clear();
        });
    
    valid_crop_btn
        .addEventListener('click', () => {
            cropper.getCroppedCanvas().toBlob(blob => {
                console.log('before', window.img2change)
                window.img2change.blob = blob;
                window.img2change.image.src = URL.createObjectURL(blob);
                window.img2change.image.classList.remove('hidden');
                window.img2change.container.classList.add('drop_zone--is-filled');
                window.img2change.image.setAttribute('draggable', true);
                console.log(window.img2change)
            });
    
            asset_editor_modal.close();
        });
    document.querySelector('#projects .card__header button').addEventListener('click', () => add_project_modal.open());
    
    // Quill auto-height
    document.querySelector('[name=description]').addEventListener('input', e => {
        e.target.style.height = '5px';
        e.target.style.height = `${e.target.scrollHeight}px`;
    });
});