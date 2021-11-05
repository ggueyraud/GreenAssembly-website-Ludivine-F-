export default class Modal {
    // #modal = null;
    #events = new Map();

    constructor(element) {
        this.modal = element;
        this.modal.addEventListener('mousedown', e => {
            if (this.is_open && e.target === this.modal) {
                this.close();
            }
        });

        window.addEventListener('keydown', e => {
            if (this.is_open && e.key === 'Escape') {
                this.close();
                e.preventDefault();
            }
        }, true);

        const header = this.modal.querySelector('.modal__dialog__header');

        if (header) {
            const close_btn = document.createElement('button');
            close_btn.type = 'button';
            close_btn.innerHTML = `<svg class="icon icon--sm" height="18px">
                <use xlink:href="/icons.svg#close"></use>
            </svg>`;
            close_btn.title = 'Fermer la fenêtre';
            close_btn.classList.add('modal__dialog__header__close');
            close_btn.addEventListener('click', () => this.close());
            header.insertAdjacentElement('beforeend', close_btn);
        }
    }

    get is_open() {
        return this.modal.classList.contains('modal--show');
    }

    on(event_name, callback) {
        if (!this.#events.has(event_name)) {
            this.#events.set(event_name, callback);
        }

        return this
    }

    #fire(event_name, args = []) {
        if (this.#events.has(event_name)) {
            return this.#events.get(event_name)(this, ...args);
        }

        // throw new TypeError(`Event "${event_name}" doesn't exist!`);
    }

    open() {
        this.modal.classList.add('modal--show');

        this.#fire('open');
    }

    close() {
        this.#fire('close');

        this.modal.classList.remove('modal--show');
    }
}