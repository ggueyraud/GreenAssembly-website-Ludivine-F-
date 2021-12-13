const is_filled_class = 'drop_zone--is-filled';
const hover_class = 'drop_zone--hover';

// TODO : implement max file size
export class DropZone {
    #events = new Map();

    constructor(container) {
        this.container = container;
        this.image = container.querySelector('img');
        this.input = container.querySelector('input');
        this.blob = null;
        this.#events.set('change', (_, image) => {
            this.image.setAttribute('src', image);
        });

        const remove_btn = this.container.querySelector('button');
        remove_btn.addEventListener('click', () => this.#fire('clear'));

        this.input.addEventListener('change', () => {
            const reader = new FileReader();

            reader.onload = e => {
                this.#fire('change', [e.target.result]);
                this.container.classList.add(is_filled_class);
            }

            reader.readAsDataURL(this.input.files[0]);
        });
    }

    get is_filled() {
        return this.input.value !== '';
    }

    setImage(image) {
        this.image.setAttribute('src', image);
        this.container.classList.add(is_filled_class);
    }

    clear() {
        //this.input.value = '';

        this.container.classList.remove(is_filled_class);

        // Create timeout for CSS animation
        // setTimeout(() => {
            Object.assign(this.image, {
                src: '',
                draggable: false
            });
        // }, 250);
    }

    #fire(event_name, args = []) {
        if (this.#events.has(event_name)) {
            return this.#events.get(event_name)(this, ...args);
        }

        // throw new TypeError(`Event "${event_name}" doesn't exist!`);
    }

    on(event_name, callback) {
        this.#events.set(event_name, callback);

        return this;
    }
}

export default class AssetsGrid {
    #events = new Map();
    #dragged_element = null;
    #items = [];
    #limit = 0;

    constructor(container) {
        this.#items = [...container.querySelectorAll('.drop_zone')].map((item, index) => {
            const drop_zone = new DropZone(item);

            drop_zone
                .on('clear', () => {
                    console.log('clear', this.#limit);
                    this.#items[this.#limit].input.setAttribute('disabled', true);
                    this.#limit--;

                    this.#update(index, true);

                    // // Create timeout for CSS animation
                    // setTimeout(() => {
                    //     Object.assign(drop_zone.image, {
                    //         src: '',
                    //         draggable: false
                    //     });
                    // }, 250);
                })
                .on('change', (drop_zone, image) => {
                    this.#fire('select', [image, drop_zone]);
                    // Make available next dropzone
                    this.#limit++;
                    this.#items[this.#limit].input.removeAttribute('disabled');
                });
            
            // Input handling
            // drop_zone.input.addEventListener('change', () => {
            //     const reader = new FileReader();

            //     reader.onload = e => {
            //         this.#fire('select', [e.target.result, drop_zone]);
            //         this.#limit++;
            //         this.#items[this.#limit].input.removeAttribute('disabled');
            //     }

            //     reader.readAsDataURL(drop_zone.input.files[0]);
            // });

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

    /////// HERE
    #update(updated_index, recalculate_each_position = false) {
        if (recalculate_each_position) {
            this.#items.forEach((item, index) => {
                console.log(item, index, updated_index, item.is_filled);
                console.log(index >= updated_index && item.is_filled);

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
        this.#items.forEach((item, index) => {
            item.clear();

            if (index > 0) {
                item.input.setAttribute('disabled', true);
            }
        });

        this.#limit = 0;
    }

    get value() {
        const value = [];

        this
            .#items
            .filter(item => item.image.getAttribute('src'))
            .forEach(item => value.push(item.blob));

        return value;
    }
}