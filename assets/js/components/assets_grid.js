const is_filled_class = 'assets__item--is-filled';
const hover_class = 'assets__item--hover';

class DropZone {
    constructor(container) {
        this.container = container;
        this.image = container.querySelector('img');
        this.input = container.querySelector('input');
        this.blob = null;

        // const observer = new MutationObserver(mutations => {
        //     mutations.forEach(mutation => {
        //         if (mutation.type === 'attributes' && mutation.attributeName === 'src') {
        //             if (mutation.target.getAttribute('src') === '') {
        //                 console.log(this.input.value);
        //                 this.input.value = null;
        //                 console.log(this.input.value);
        //                 console.log('clear input value and set is_filled to false')
        //             }
        //         }
        //     });
        // });

        // observer.observe(this.image, {
        //     attributes: true,
        //     childList: false,
        //     CharacterData: false
        // });
    }

    get is_filled() {
        return this.input.value !== '';
    }

    clear() {
        this.input.value = '';

        this.container.classList.remove(is_filled_class);

        // Create timeout for CSS animation
        // setTimeout(() => {
            Object.assign(this.image, {
                src: '',
                draggable: false
            });
        // }, 250);
    }
}

export default class AssetsGrid {
    #events = new Map();
    #dragged_element = null;
    #items = [];
    #limit = 0;

    constructor(container) {
        this.#items = [...container.querySelectorAll('.assets__item')].map((item, index) => {
            const drop_zone = new DropZone(item);

            // Remove button
            const btn = item.querySelector('button');
            btn.addEventListener('click', () => {
                if (index > 0) {
                    this.#items[this.#limit].input.disabled = true;
                    this.#limit--;

                }

                // drop_zone.clear();
                this.#update(index, true);

                // // Create timeout for CSS animation
                // setTimeout(() => {
                //     Object.assign(drop_zone.image, {
                //         src: '',
                //         draggable: false
                //     });
                // }, 250);
            });
            
            // Input handling
            drop_zone.input.addEventListener('change', () => {
                const reader = new FileReader();

                reader.onload = e => {
                    this.#fire('select', [e.target.result, drop_zone]);
                    this.#limit++;
                    this.#items[this.#limit].input.disabled = false;
                }

                reader.readAsDataURL(drop_zone.input.files[0]);
            });

            // Events initialization
            drop_zone.container.addEventListener('dragstart', e => this.#dragged_element = e.target, false);
            drop_zone.container.addEventListener(
                'drop',
                e => {
                    e.preventDefault();

                    // Can only move asset to a dropzone which is filled, prevent drop an element
                    // from which is not a drop_zone
                    if (drop_zone.is_filled && this.#dragged_element) {
                        const new_src = this.#dragged_element.getAttribute('src');
    
                        drop_zone.container.classList.remove(hover_class);
                        drop_zone.image.setAttribute('draggable', true);
    
                        const src = drop_zone.image.getAttribute('src');
                        if (src) {
                            this.#dragged_element.setAttribute('src', src);
    
                            // Move image to new location
                            drop_zone.image.setAttribute('src', new_src);
                            this.#dragged_element = null;
                        } else {
                            this.#dragged_element.parentElement.classList.remove(is_filled_class);
    
                            setTimeout(() => {
                                this.#dragged_element.setAttribute('src', '');
                                
                                // Move image to new location
                                drop_zone.image.setAttribute('src', new_src);
                                this.#dragged_element = null;
                            }, 250);
                        }
    
                        if (!drop_zone.container.classList.contains(is_filled_class)) {
                            drop_zone.container.classList.add(is_filled_class);
                        }
                    }
                },
                false
            );
            drop_zone.container.addEventListener(
                'dragover',
                e => {
                    e.preventDefault();

                    drop_zone.container.classList.add(hover_class);
                },
                false
            );
            drop_zone.container.addEventListener(
                'dragleave',
                () => {
                    drop_zone.container.classList.remove(hover_class);
                },
                false
            );

            return drop_zone
        });
    }

    #update(updated_index, recalculate_each_position = false) {
        if (recalculate_each_position) {
            this.#items.forEach((item, index) => {
                if (index >= updated_index && item.is_filled) {
                    const prev = this.#items[index - 1];

                    if (prev) {
                        prev.image.setAttribute('src', item.image.getAttribute('src'));
                        prev.container.classList.add(is_filled_class);
                        item.clear();
                    }
                    
                }

                if (index === 0) {
                    item.clear();
                }
            });
        }
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

    clear() {
        this.#items.forEach(item => {
            item.clear();
        });
    }

    get value() {
        const value = [];

        this
            .#items
            .filter(item => item.image.getAttribute('src'))
            .forEach(item => value.push(item.blob))
        // this.#items.forEach(item => {
        //     value.push(item.image.getAttribute);
        // });

        return value;
    }
}