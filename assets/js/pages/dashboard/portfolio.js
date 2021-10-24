import Swal from 'sweetalert2';
import Sortable from 'sortablejs';
import Form, { Required, StringLength } from 'formvalidation';
import Quill from 'quill';
import { post, put, del } from '@js/utils/http';
import '@js/components/modal';
import Modal from '../../components/modal';
import DOMPurify from 'dompurify'
import { formatDistance } from 'date-fns'
import { fr } from 'date-fns/locale';
import Cropper from 'cropperjs';
import AssetsGrid from '@js/components/assets_grid';

let categories_container = null;
let projects_container = null;
let sortable_categories = null;

const swal_error = () => Swal.fire({
    title: 'Une erreur est survenue',
    text: 'Si le problème persiste veuillez contacter la personne en charge de la maintenance de votre site-web.',
    icon: 'error',
    footer: `<a href="https://greenassembly.fr/contact" target="_blank">Contacter l'agence GreenAssembly</a>`
})

const add_category = (id, name) => {
    const category = document.createElement('li');
    category.dataset.id = id;
    category.classList.add('categories__item');
    
    const input = document.createElement('input');
    input.type = 'text';
    input.maxLength = 30;
    input.addEventListener('focus', () => console.log('focus'))
    input.addEventListener('blur', () => {
        sortable_categories.option('disabled', false);
        category.classList.remove('categories__item--edition');
    });

    const category_name = document.createElement('span');
    category_name.innerText = name;
    category_name.classList.add('categories__item__name');
    category_name.addEventListener('click', () => {
        sortable_categories.option('disabled', true);
        input.value = category_name.innerText;
        category.classList.add('categories__item--edition');
        input.focus();
        input.setSelectionRange(0, input.value.length);
    });
    input.addEventListener('keydown', e => {
        const value = e.target.value;

        if (e.key === 'Enter' && value) {
            put(`/portfolio/categories/${category.dataset.id}`, {
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                },
                body: new URLSearchParams({ name: value })
            })
                .then(() => {
                    category.classList.remove('categories__item--edition')
                    category_name.innerText = value;
                })
                .catch(swal_error)
        }
    });

    // TODO : change onto button
    const delete_btn = document.createElement('a');
    delete_btn.href = 'javascript:;';
    delete_btn.innerHTML = `<svg class="icon" height="20px">
        <use xlink:href="/dashboard_icons.svg#delete"></use>
    </svg>`;
    delete_btn.addEventListener('click', () => {
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
                    del(`/portfolio/categories/${id}`)
                        .then(() => categories_container.querySelector(`[data-id="${id}"]`).remove())
                        .catch(swal_error)
                }
            });
    });
    
    category.appendChild(category_name);
    category.appendChild(input);
    category.appendChild(delete_btn);

    categories_container.appendChild(category);
}

const add_project = (id, name, content, date, after = true) => {
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

    // TODO : replace with button
    const update_btn = document.createElement('a');
    update_btn.href = 'javascript:;';
    update_btn.innerHTML = `<svg class="icon" height="20px">
        <use xlink:href="/dashboard_icons.svg#edit"></use>
    </svg>`;
    const delete_btn = document.createElement('a');
    delete_btn.href = 'javascript:;';
    delete_btn.classList.add('text_error');
    delete_btn.innerHTML = `<svg class="icon" height="20px">
        <use xlink:href="/dashboard_icons.svg#delete"></use>
    </svg>`;
    delete_btn.addEventListener('click', e => {
        e.preventDefault();

        Swal.fire({
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
                    del(`/portfolio/projects/${id}`)
                        .then(() => project.remove())
                        .catch(swal_error)
                }
            });
    });

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

const on_mount = () => {
    // Categories
    categories_container = document.querySelector('.categories');
    projects_container = document.querySelector('.projects');

    const new_category_input = document.querySelector('[name=new_category_name]');

    categories.forEach(category => add_category(category.id, category.name));
    projects.forEach(project => add_project(project.id, project.name, project.content, project.date));

    sortable_categories = new Sortable(categories_container, {
        animation: 150,
        onEnd: e => {
            if (e.newIndex !== e.oldIndex) {
                put(`/portfolio/categories/${e.item.dataset.id}`, {
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
                post('/portfolio/categories', {
                    headers: {
                        'Content-Type': 'application/x-www-form-urlencoded'
                    },
                    body: new URLSearchParams({ name: value })
                })
                    .then(async res => {
                        const id = await res.json();

                        add_category(id, value);
                        new_category_input.value = '';
                    })
                    .catch(swal_error)
            }
        });

    // Projects
    const editor = new Quill('#content_editor', {
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
        content.value = value;
        content.dispatchEvent(new Event('input'));
    })

    const add_project_modal = new Modal(document.querySelector('#add_project_modal'));
    add_project_modal.on('open', e => {
        e.modal.querySelector('[name="name"]').focus();
    });
    add_project_modal.on('close', () => {
        assets_grid.clear();
    });

    const cropper_el = document.querySelector('#cropper');
    let cropper = null;
    const assets_grid = new AssetsGrid(document.querySelector('.assets'));
    assets_grid.on('select', (_, image, img) => {
        cropper_el.setAttribute('src', image);
        window.img2change = img;

        if (cropper) {
            cropper.replace(image);
        } else {
            cropper = new Cropper(cropper_el, {
                minCropBoxWidth: 320,
                zoom(e) {
                    if (e.detail.ratio <= 0.5 || e.detail.ratio >= 2.5) {
                        e.preventDefault()
                    }
                    console.log(e)
                }
            });
        }
        asset_editor_modal.open();
    });
    
    const asset_editor_modal = new Modal(document.querySelector('#asset_editor_modal'));
    document.querySelector('#rotate').addEventListener('input', e => {
        const value = parseInt(e.target.value);

        e.target.nextElementSibling.innerText = `${value}%`;
        cropper.rotateTo(value);
    });

    document.querySelector('#valid_crop').addEventListener('click', () => {
        cropper.getCroppedCanvas().toBlob(blob => {
            // const { image, blob } = window.img2change;
            // console.log(blob, window.img2change);
            window.img2change.blob = blob;
            window.img2change.image.src = URL.createObjectURL(blob);
            window.img2change.image.classList.remove('hidden');
            window.img2change.image.parentElement.classList.add('assets__item--is-filled')
            window.img2change.image.setAttribute('draggable', true);
        });

        asset_editor_modal.close();
    });
    document.querySelector('#create_project').addEventListener('click', () => add_project_modal.open());

    // Quill auto-height
    document.querySelector('[name=description]').addEventListener('input', e => {
        e.target.style.height = '5px';
        e.target.style.height = `${e.target.scrollHeight}px`;
    });

    // Form validation
    new Form(document.querySelector('[name=create_project]'), {
        fields: {
            name: {
                validators: [new Required(), new StringLength(1, 120)]
            },
            description: {
                validators: [new StringLength(0, 320)]
            },
            content: {
                validators: [new Required(), new StringLength(30, 1000)]
            }
        }
    })
        .on('valid', () => console.log('valid'))
        .on('send', e => {
            const form_data = new FormData();

            for (const [key, value] of Object.entries(e.detail)) {
                form_data.append(key, value);
            }

            assets_grid.value.forEach(img => form_data.append('files[]', img));
            // form_data.append('files', assets_grid.value[0])

            post('/portfolio/projects', {
                // headers: {
                //     'Content-Type': 'multipart/form-data'
                // },
                // body: new URLSearchParams(e.detail)
                body: form_data
            })
                .then(async res => {
                    const id = await res.json();

                    add_project_modal.close();

                    add_project(id, e.detail.name, e.detail.content, new Date(), false)
                })
                .catch(swal_error)
        });
}

const on_destroy = () => {
    window.removeEventListener('onMount', on_mount)
    window.removeEventListener('onDestroy', on_destroy)
}

window.addEventListener('onMount', on_mount)
window.addEventListener('onDestroy', on_destroy)