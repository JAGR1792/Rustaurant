// agenda.js — panel de empleado dinámico: navegar por días y meses del
// calendario sin recargar la página completa. Pide el fragmento
// GET /agenda/dia?dia=AAAA-MM-DD, reemplaza los KPIs y el panel (calendario,
// barra de día y tabla), y sincroniza la URL con pushState/popstate. Si el
// fragmento falla, cae a la navegación completa de respaldo. Sin librerias.
(function () {
  "use strict";

  const panelActual = () => document.getElementById("agenda-panel");
  const kpisActual = () => document.getElementById("agenda-kpis");

  function marcarCargando(activo) {
    for (const el of [document.getElementById("agenda-seccion"), kpisActual()]) {
      if (!el) continue;
      if (activo) el.setAttribute("aria-busy", "true");
      else el.removeAttribute("aria-busy");
    }
  }

  async function cargarDia(url, empujar) {
    marcarCargando(true);
    try {
      const r = await fetch(url, { headers: { "X-Peticion": "fragmento" } });
      if (!r.ok) throw new Error("respuesta no válida");
      const doc = new DOMParser().parseFromString(await r.text(), "text/html");
      const kpis = doc.getElementById("agenda-kpis");
      const panel = doc.getElementById("agenda-panel");
      if (!kpis || !panel || !panelActual() || !kpisActual()) throw new Error("fragmento incompleto");
      kpisActual().replaceWith(document.importNode(kpis, true));
      panelActual().replaceWith(document.importNode(panel, true));
      if (empujar) history.pushState({ agenda: url }, "", url);
    } catch {
      window.location.href = url;
      return;
    } finally {
      marcarCargando(false);
    }
  }

  // Delegación en el documento: los enlaces viven dentro del panel, que se
  // reemplaza en cada navegación.
  document.addEventListener("click", (ev) => {
    if (ev.defaultPrevented || ev.button !== 0 || ev.metaKey || ev.ctrlKey || ev.shiftKey || ev.altKey) return;
    const enlace = ev.target.closest("#agenda-panel a[href^='/agenda?dia=']");
    if (!enlace) return;
    ev.preventDefault();
    cargarDia(enlace.getAttribute("href"), true);
  });

  // Atrás/adelante del navegador: se vuelve a pedir el fragmento sin apilar.
  window.addEventListener("popstate", () => {
    if (!panelActual()) return;
    const url = window.location.pathname + window.location.search || "/agenda";
    cargarDia(url, false);
  });
})();
