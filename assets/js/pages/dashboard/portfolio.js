import Swal from 'sweetalert2';
import Sortable from 'sortablejs';
import Form, { Required, StringLength } from 'formvalidation';
import Quill from 'quill';
import { post, put, del } from '@js/utils/http';
import '@js/components/modal';
import Modal from '../../components/modal';

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

        if (e.which === 13 && value) {
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

const add_project = (id, name, content, date) => {
    const project = document.createElement('div');
    project.classList.add('projects__item');

    const project_title = document.createElement('div');
    project_title.classList.add('projects__item__title');
    project_title.innerText = name;

    const project_content = document.createElement('div');
    project_content.classList.add('projects__item__content');
    project_content.innerText = content;

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
    project_footer.innerText = date;

    project.appendChild(project_title);
    project.appendChild(project_content);
    project.appendChild(project_actions);
    project.appendChild(project_footer);
    projects_container.appendChild(project);
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
    const m = new Modal(document.querySelector('.modal'));
    m.on('open', e => {
        console.log(e);
        console.log('modal open');
    });

    document.querySelector('#create_project').addEventListener('click', () => m.open())

    new Quill('#content_editor', {
        modules: {
            toolbar: [
                [{ header: [2, 3, false] }],
                [{ list: 'ordered' }, { list: 'bullet' }],
                ['link', 'clean']
            ]
        },
        theme: 'snow'
    });
    document.querySelector('[name=description]').addEventListener('input', e => {
        e.target.style.height = "5px";
        e.target.style.height = (e.target.scrollHeight)+"px";
    });
    new Form(document.querySelector('[name=create_project]'), {
        fields: {
            name: {
                validators: [new Required(), new StringLength(1, 120)]
            },
            description: {
                validators: [new StringLength(0, 320)]
            },
            content: {
                validators: [new Required(), new StringLength(1, 1000)]
            }
        }
    })
        .on('valid', () => console.log('valid'))
}

const on_destroy = () => {
    window.removeEventListener('onMount', on_mount)
    window.removeEventListener('onDestroy', on_destroy)
}

window.addEventListener('onMount', on_mount)
window.addEventListener('onDestroy', on_destroy)