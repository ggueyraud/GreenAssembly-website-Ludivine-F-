const on_mount = () => {
    document
        .querySelectorAll('#projects nav button')
        .forEach(btn => {
            
            btn.addEventListener('click', e => {
                const active_btn_filter = document.querySelector('#projects nav button.active');
                
                if (e.target != active_btn_filter) {
                    active_btn_filter.classList.remove('active');
                    btn.classList.add('active');
                }

                console.log(e.target.dataset.id)
            })
        })
}

const on_destroy = () => {
    window.removeEventListener('onMount', on_mount)
    window.removeEventListener('onDestroy', on_destroy)
}

window.addEventListener('onMount', on_mount)
window.addEventListener('onDestroy', on_destroy)