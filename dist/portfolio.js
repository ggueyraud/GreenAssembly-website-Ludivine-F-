!function(){let e;const t=()=>{e=[...document.querySelectorAll(".project")],document.querySelectorAll("#projects nav button").forEach((e=>{e.addEventListener("click",(t=>{const a=document.querySelector("#projects nav button.active");t.target!=a&&(a.classList.remove("active"),e.classList.add("active"),o(t.target.dataset.id))}))}))},o=t=>{const o=document.querySelector("#aaa");o.animate([{opacity:100},{opacity:0}],{duration:250}),console.log(e),setTimeout((()=>{let a=e;t&&(a=a.filter((e=>e.dataset.categories.split(";").includes(t)))),a=a.map((e=>e.outerHTML));let n="";a.forEach(((e,t)=>{switch(t){case 0:n+=`<div class="col_span_2 row_span_2">${e}</div>`;break;case 1:case 2:n+=`<div>${e}</div>`;break;case 3:n+=`<div class="col_span_2>${e}</div>`;break;default:n+=`<div>${e}</div>`}})),o.innerHTML=n,o.animate([{opacity:0},{opacity:100}],{duration:250})}),250)},a=()=>{window.removeEventListener("onMount",t),window.removeEventListener("onDestroy",a)};window.addEventListener("onMount",t),window.addEventListener("onDestroy",a)}();