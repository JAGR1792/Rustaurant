// ui.js original Brindis: toasts y modal de confirmacion propios, sin librerias.
// Patron visto en Camilo (efectos.js) y Emp_Transporte (hooks de UI),
// reescrito minimo y con prefers-reduced-motion respetado en CSS.
function toast(msg) {
  const c = document.getElementById("toasts");
  const el = document.createElement("p");
  el.textContent = msg;
  c.appendChild(el);
  setTimeout(() => el.remove(), 4000);
}

// Dialogo de confirmacion reutilizable para acciones destructivas.
// Devuelve una promesa: true si la persona confirma, false si vuelve.
function confirmar(mensaje) {
  return new Promise((resolver) => {
    const dlg = document.getElementById("modal");
    if (!dlg || typeof dlg.showModal !== "function") {
      resolver(window.confirm(mensaje));
      return;
    }
    dlg.innerHTML = "";
    const titulo = document.createElement("h2");
    titulo.textContent = "Confirmar acción";
    const texto = document.createElement("p");
    texto.textContent = mensaje;
    const acciones = document.createElement("div");
    acciones.className = "acciones";
    const si = document.createElement("button");
    si.type = "button";
    si.textContent = "Sí, continuar";
    const no = document.createElement("button");
    no.type = "button";
    no.className = "secundario";
    no.textContent = "Volver";
    acciones.append(si, no);
    dlg.append(titulo, texto, acciones);

    let resuelto = false;
    const cerrar = (valor) => {
      if (resuelto) return;
      resuelto = true;
      if (dlg.open) dlg.close();
      resolver(valor);
    };
    si.addEventListener("click", () => cerrar(true));
    no.addEventListener("click", () => cerrar(false));
    dlg.addEventListener("close", () => cerrar(false), { once: true });
    dlg.showModal();
    no.focus();
  });
}

// Formularios con data-confirmar piden confirmacion antes de enviarse.
document.addEventListener("submit", (ev) => {
  const form = ev.target;
  if (!(form instanceof HTMLFormElement) || !form.dataset.confirmar || form.dataset.confirmado) return;
  ev.preventDefault();
  confirmar(form.dataset.confirmar).then((ok) => {
    if (!ok) return;
    form.dataset.confirmado = "1";
    form.submit();
  });
});
