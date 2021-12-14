import Swal from "sweetalert2";

export default () => Swal.fire({
    title: 'Une erreur est survenue',
    text: 'Si le problème persiste veuillez contacter la personne en charge de la maintenance de votre site-web.',
    icon: 'error',
    footer: "<a href=\"https://greenassembly.fr/contact\" target=\"_blank\">Contacter l'agence GreenAssembly</a>"
});

export const data_removed = (name) => Swal.fire({
    title: "L'élément n'existe plus",
    text: `${name} a déjà été supprimé`,
    icon: 'warning'
})