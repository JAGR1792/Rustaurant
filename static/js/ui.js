// ui.js original Brindis: toasts y modal propios, sin librerias.
// Patron visto en Camilo (efectos.js) y Emp_Transporte (hooks de UI),
// reescrito minimo y con prefers-reduced-motion respetado en CSS.
function toast(msg) {
  const c = document.getElementById("toasts");
  const el = document.createElement("p");
  el.textContent = msg;
  c.appendChild(el);
  setTimeout(() => el.remove(), 4000);
}
