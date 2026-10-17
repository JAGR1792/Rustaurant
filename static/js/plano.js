// plano.js original Brindis: asistente de reserva en 4 pasos + plano por zonas.
// Consulta GET /api/disponibilidad?fecha=&hora=&zona= y arma el POST /reserva
// llenando los campos ocultos `mesas` y `extras`. Sin librerias.
(function () {
  "use strict";

  const seleccion = new Set();
  let mesasVisibles = [];

  const $ = (id) => document.getElementById(id);

  function aviso(msg) {
    if (typeof toast === "function") toast(msg);
    else alert(msg);
  }

  function mostrarPaso(n) {
    for (const s of document.querySelectorAll("section[id^='paso-']")) {
      s.hidden = s.id !== `paso-${n}`;
    }
    for (const li of document.querySelectorAll(".progreso li")) {
      if (Number(li.dataset.paso) === n) li.setAttribute("aria-current", "step");
      else li.removeAttribute("aria-current");
    }
    const h = document.querySelector(`#paso-${n} h2`);
    if (h && typeof h.focus === "function") h.focus();
  }

  function sincronizarOcultos() {
    $("mesas").value = [...seleccion].join(",");
    $("extras").value = [...document.querySelectorAll("input.extra:checked")]
      .map((i) => i.value)
      .join(",");
  }

  function resumenSeleccion() {
    const texto = $("bloqueo-aviso");
    const comensales = Number($("comensales").value || 0);
    let capacidad = 0;
    for (const b of document.querySelectorAll(".mesa[data-estado='seleccionada']")) {
      capacidad += Number(b.dataset.capacidad);
    }
    if (seleccion.size === 0) {
      texto.textContent = "Ninguna mesa seleccionada todavía.";
    } else {
      texto.textContent =
        `Mesas ${[...seleccion].join(", ")} — capacidad ${capacidad} ` +
        `para ${comensales} comensales.`;
    }
    $("btn-paso3").disabled = !(seleccion.size > 0 && capacidad >= comensales);
  }

  function pintarPlano(mesas) {
    const plano = $("plano");
    plano.innerHTML = "";
    for (const m of mesas) {
      const b = document.createElement("button");
      b.type = "button";
      b.className = "mesa";
      b.dataset.capacidad = m.capacidad;
      const estado = seleccion.has(m.id) ? "seleccionada" : m.estado;
      b.dataset.estado = estado;
      b.textContent = `M${m.id} · ${m.capacidad}`;
      b.setAttribute("aria-label", `Mesa ${m.id} en zona ${m.zona}, ${estado}`);
      b.disabled = m.estado === "reservada" || m.estado === "bloqueada";
      b.addEventListener("click", () => {
        if (seleccion.has(m.id)) seleccion.delete(m.id);
        else seleccion.add(m.id);
        pintarPlano(mesasVisibles);
      });
      plano.appendChild(b);
    }
    resumenSeleccion();
  }

  async function cargarPlano() {
    const fecha = $("fecha").value;
    const hora = $("hora").value;
    const zona = $("zona").value;
    if (!fecha || !hora) {
      aviso("Elige fecha y hora antes de ver el plano.");
      return;
    }
    try {
      const r = await fetch(
        `/api/disponibilidad?fecha=${fecha}&hora=${hora}&zona=${encodeURIComponent(zona)}`
      );
      if (!r.ok) throw new Error("respuesta no válida");
      mesasVisibles = await r.json();
      // Una mesa que dejó de estar libre se quita de la selección.
      for (const sel of [...seleccion]) {
        const m = mesasVisibles.find((x) => x.id === sel);
        if (!m || m.estado !== "libre") seleccion.delete(sel);
      }
      pintarPlano(mesasVisibles);
    } catch {
      aviso("No se pudo cargar el plano de mesas.");
    }
  }

  function estimarTotal() {
    let total = 0;
    for (const i of document.querySelectorAll("input[data-precio]")) {
      total += Number(i.dataset.precio) * Number(i.value || 0);
    }
    const comensales = Number($("comensales").value || 0);
    for (const i of document.querySelectorAll("input.extra:checked")) {
      const tarifa = Number(i.dataset.tarifa);
      total += i.dataset.porpersona === "1" ? tarifa * comensales : tarifa;
    }
    return total;
  }

  function pintarEstimado() {
    const el = $("total-estimado");
    if (!el) return;
    const cop = new Intl.NumberFormat("es-CO", {
      style: "currency",
      currency: "COP",
      maximumFractionDigits: 0,
    });
    el.textContent =
      `Total estimado de pedido y extras: ${cop.format(estimarTotal())}. ` +
      "El valor final se recalcula en el servidor.";
  }

  function iniciar() {
    if (!$("form-reserva")) return;

    for (const b of document.querySelectorAll("[data-hacia]")) {
      b.addEventListener("click", () => {
        const destino = Number(b.dataset.hacia);
        if (destino === 2 && b.closest("#paso-1")) {
          if (!document.querySelector("input[name=ocasion]:checked")) {
            aviso("Elige la ocasión de tu reserva.");
            return;
          }
          mostrarPaso(2);
          cargarPlano();
          return;
        }
        if (destino === 3) {
          if (seleccion.size === 0) {
            aviso("Selecciona al menos una mesa.");
            return;
          }
          sincronizarOcultos();
          pintarEstimado();
        }
        if (destino === 2 && !b.closest("#paso-1")) cargarPlano();
        mostrarPaso(destino);
      });
    }

    $("btn-ver-plano").addEventListener("click", cargarPlano);
    $("comensales").addEventListener("change", resumenSeleccion);
    for (const i of document.querySelectorAll("input.extra")) {
      i.addEventListener("change", pintarEstimado);
    }
    for (const i of document.querySelectorAll("input[data-precio]")) {
      i.addEventListener("input", pintarEstimado);
    }
    $("form-reserva").addEventListener("submit", sincronizarOcultos);
    mostrarPaso(1);
  }

  document.addEventListener("DOMContentLoaded", iniciar);
})();
