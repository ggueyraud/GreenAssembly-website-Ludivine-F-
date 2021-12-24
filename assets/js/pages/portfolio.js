import 'router';

let all_projects;
const { router } = window;

const sort = id => {
    const container = document.querySelector('#aaa');
    container.animate([
        { opacity: 100 },
        { opacity: 0 }
    ], { duration: 250 });

    setTimeout(() => {
        let projects = all_projects; //.map(project => project.outerHTML);
    
        if (id) {
            projects = projects.filter(project => {
                let categories = project.dataset.categories.split(';');
    
                return categories.includes(id)
            });
        }
    
        projects = projects.map(project => project.outerHTML);
    
        let grid = '';
    
        projects.forEach((project, index) => {
            switch (index) {
                case 0:
                    grid += `<div class="col_span_2 row_span_2">${project}</div>`;
                break;
                case 1:
                case 2:
                    grid += `<div>${project}</div>`;
                break;
                case 3:
                    grid += `<div class="col_span_2>${project}</div>`;
                break;
                default:
                    grid += `<div>${project}</div>`;
                break;
            }
        })
    
        container.innerHTML = grid
        container.animate([
            { opacity: 0 },
            { opacity: 100 }
        ], { duration: 250 });
    }, 250)
}

router.on('mount', () => {
    all_projects = [...document.querySelectorAll('.project')];
    
    document
        .querySelectorAll('#projects nav button')
        .forEach(btn => {
            
            btn.addEventListener('click', e => {
                const active_btn_filter = document.querySelector('#projects nav button.active');
                
                if (e.target != active_btn_filter) {
                    active_btn_filter.classList.remove('active');
                    btn.classList.add('active');
    
                    sort(e.target.dataset.id)
                }
            })
        })
});