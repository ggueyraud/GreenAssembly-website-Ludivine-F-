window.addEventListener("onMount",(()=>{document.querySelectorAll(".input").forEach((e=>{const t=e.querySelector("input, textarea");t instanceof HTMLTextAreaElement&&t.addEventListener("input",(e=>{e.target.style.height="5px",e.target.style.height=e.target.scrollHeight+"px"})),t.addEventListener("blur",(t=>{t.target.value.length>0&&e.classList.add("valid")}))}))}));