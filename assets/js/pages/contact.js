const on_mount = () => {
    document
        .querySelectorAll('.input')
        .forEach(input_container => {
            const input = input_container.querySelector('input, textarea');

            // Set autoheight
            if (input instanceof HTMLTextAreaElement) {
                input
                    .addEventListener('input', e => {
                        e.target.style.height = "5px";
                        e.target.style.height = (e.target.scrollHeight)+"px";
                    })
            }

            input
                .addEventListener('blur', e => {
                    if (e.target.value.length > 0) {
                        input_container.classList.add('valid')
                    }
                })
        })
}

const on_destroy = () => {
    window.removeEventListener('onMount', on_mount)
}

window.addEventListener('onMount', on_mount)