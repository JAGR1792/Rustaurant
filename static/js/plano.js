// plano.js — asistente de reserva en 4 pasos: plano SVG del salón por zonas
// y ficha lateral de la mesa elegida. Consulta
// GET /api/disponibilidad?fecha=&hora=&zona= y arma el POST /reserva
// llenando los campos ocultos `mesas` y `extras`. Sin librerias.
(function () {
  "use strict";

  const NS = "http://www.w3.org/2000/svg";

  const seleccion = new Set();
  let mesasVisibles = [];
  let ultimaMesa = null;

  const $ = (id) => document.getElementById(id);

  // Dibujo del salón: cada zona tiene sus posiciones fijas para las mesas.
  // Si la API devuelve más mesas que posiciones, las extra no se dibujan.
  const ZONAS = [
    { nombre: "Terraza", etiqueta: "TERRAZA", etiquetaX: 445, etiquetaY: 52, ancla: "middle", slots: [[320, 112], [570, 112]] },
    { nombre: "General", etiqueta: "SALÓN PRINCIPAL", etiquetaX: 450, etiquetaY: 274, ancla: "middle", slots: [[250, 352], [430, 352], [610, 352], [790, 352]] },
    { nombre: "VIP", etiqueta: "ZONA ÍNTIMA · VIP", etiquetaX: 54, etiquetaY: 508, ancla: "start", slots: [[150, 540], [300, 540]] },
  ];

  const FOTO_ZONA = {
    Terraza: "/static/img/fotos/mesa-terraza.jpg",
    General: "/static/img/fotos/mesa-general.jpg",
    VIP: "/static/img/fotos/mesa-vip.jpg",
  };

  const ESTADO_TEXTO = { libre: "libre", reservada: "reservada", bloqueada: "bloqueada", seleccionada: "seleccionada" };

  const COLORES = {
    libre: { cuerpo: "#e7d9bf", borde: "#c9b795", texto: "#26231f", silla: "#9c8a6d" },
    seleccionada: { cuerpo: "#702c3b", borde: "#a88a59", texto: "#fdf9f1", silla: "#cbb48e", grosor: 3.5 },
    reservada: { cuerpo: "#ddd5c9", borde: "#c4bbac", texto: "#7a7264", silla: "#b5ac9d" },
    bloqueada: { cuerpo: "#cfc7b8", borde: "#9b9282", texto: "#6d675e", silla: "#a89f90", guion: "5 4" },
  };

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

  // --- ayudas SVG -----------------------------------------------------------

  function el(tag, atrs = {}) {
    const nodo = document.createElementNS(NS, tag);
    for (const [k, v] of Object.entries(atrs)) nodo.setAttribute(k, String(v));
    return nodo;
  }

  function texto(txt, x, y, clase, ancla, opciones = {}) {
    const t = el("text", { x, y, "text-anchor": ancla || "start" });
    if (clase) t.setAttribute("class", clase);
    if (opciones.size) t.setAttribute("font-size", opciones.size);
    if (opciones.peso) t.setAttribute("font-weight", opciones.peso);
    if (opciones.fill) t.setAttribute("fill", opciones.fill);
    if (opciones.opacidad) t.setAttribute("fill-opacity", opciones.opacidad);
    t.textContent = txt;
    return t;
  }

  function muro(svg, x1, y1, x2, y2, punteado) {
    const l = el("line", { x1, y1, x2, y2, stroke: "#e0d5c1", "stroke-width": 2 });
    if (punteado) l.setAttribute("stroke-dasharray", "2 6");
    svg.appendChild(l);
  }

  function dibujarSalon(svg) {
    svg.appendChild(el("rect", { x: 16, y: 16, width: 868, height: 588, rx: 18, fill: "#fbf7f0", stroke: "#e0d5c1", "stroke-width": 2 }));
    // Ventanales del muro superior (dan a la terraza).
    for (const x0 of [150, 400]) {
      svg.appendChild(el("rect", { x: x0, y: 9, width: 230, height: 15, rx: 7, fill: "#f2e9d7", stroke: "#e0d5c1" }));
    }
    svg.appendChild(texto("VENTANALES", 836, 22, "plano-nota", "end"));
    // Pasillo central punteado.
    muro(svg, 36, 216, 368, 216, true);
    muro(svg, 532, 216, 864, 216, true);
    svg.appendChild(texto("PASILLO", 450, 221, "plano-texto", "middle"));
    // Rincón VIP separado por media pared.
    muro(svg, 36, 472, 402, 472, false);
    muro(svg, 402, 472, 402, 604, false);
    // Barra con banquetas.
    svg.appendChild(el("rect", { x: 640, y: 496, width: 220, height: 88, rx: 12, fill: "#efe4d1", stroke: "#e0d5c1" }));
    svg.appendChild(texto("BAR", 750, 549, "plano-texto", "middle"));
    for (const x of [680, 730, 780]) {
      svg.appendChild(el("circle", { cx: x, cy: 486, r: 9, fill: "#dcccaa", stroke: "#c9b795", "stroke-width": 2 }));
    }
    // Entrada: hueco en el muro inferior con su umbral.
    svg.appendChild(el("rect", { x: 428, y: 598, width: 124, height: 12, fill: "#fbf7f0" }));
    svg.appendChild(texto("ENTRADA", 490, 592, "plano-nota", "middle"));
  }

  function dibujarMesa(m, cx, cy) {
    const elegida = seleccion.has(m.id);
    const estado = elegida ? "seleccionada" : m.estado;
    const clickable = m.estado === "libre" || elegida;
    const c = COLORES[estado] || COLORES.libre;
    const g = el("g", {
      class: "mesa-svg",
      "data-id": m.id,
      "data-estado": estado,
      "data-clickable": clickable ? "true" : "false",
      "aria-label": `Mesa ${m.id}, zona ${m.zona}, ${m.capacidad} puestos, ${ESTADO_TEXTO[estado] || estado}`,
    });
    if (clickable) {
      g.setAttribute("role", "button");
      g.setAttribute("tabindex", "0");
      g.setAttribute("aria-pressed", String(elegida));
    } else {
      g.setAttribute("aria-disabled", "true");
    }
    // Sillas alrededor de la mesa, una por puesto.
    const r = 50;
    for (let i = 0; i < m.capacidad; i++) {
      const ang = -90 + (i * 360) / m.capacidad;
      const rad = (ang * Math.PI) / 180;
      const x = cx + r * Math.cos(rad);
      const y = cy + r * Math.sin(rad);
      g.appendChild(el("rect", {
        x: x - 9, y: y - 4.5, width: 18, height: 9, rx: 3.5,
        fill: c.silla,
        transform: `rotate(${ang + 90} ${x} ${y})`,
      }));
    }
    const cuerpo = el("circle", { class: "mesa-cuerpo", cx, cy, r: 34, fill: c.cuerpo, stroke: c.borde, "stroke-width": c.grosor || 2 });
    if (c.guion) cuerpo.setAttribute("stroke-dasharray", c.guion);
    g.appendChild(cuerpo);
    g.appendChild(texto(`M${m.id}`, cx, cy - 1, null, "middle", { size: 13, peso: 700, fill: c.texto }));
    g.appendChild(texto(String(m.capacidad), cx, cy + 14, null, "middle", { size: 10, peso: 500, fill: c.texto, opacidad: 0.8 }));
    if (clickable) {
      g.addEventListener("click", () => alternarMesa(m));
      g.addEventListener("keydown", (ev) => {
        if (ev.key === "Enter" || ev.key === " ") {
          ev.preventDefault();
          alternarMesa(m);
        }
      });
    }
    return g;
  }

  // --- estado y ficha -------------------------------------------------------

  function alternarMesa(m) {
    if (seleccion.has(m.id)) {
      seleccion.delete(m.id);
      if (ultimaMesa === m.id) ultimaMesa = null;
    } else {
      seleccion.add(m.id);
      ultimaMesa = m.id;
    }
    pintarPlano();
    // El re-pintado recrea el SVG: devolvemos el foco a la misma mesa
    // para no perder el hilo al navegar con teclado.
    const nuevo = document.querySelector(`.mesa-svg[data-id="${m.id}"]`);
    if (nuevo && typeof nuevo.focus === "function") nuevo.focus({ preventScroll: true });
  }

  function capacidadSeleccionada() {
    let total = 0;
    for (const id of seleccion) {
      const m = mesasVisibles.find((x) => x.id === id);
      if (m) total += m.capacidad;
    }
    return total;
  }

  function mesaActual() {
    if (seleccion.size === 0) return null;
    const id = ultimaMesa !== null && seleccion.has(ultimaMesa) ? ultimaMesa : [...seleccion][0];
    return mesasVisibles.find((m) => m.id === id) || null;
  }

  function actualizarFicha() {
    const vacia = $("ficha-vacia");
    const contenido = $("ficha-contenido");
    if (!vacia || !contenido) return;
    const m = mesaActual();
    if (!m) {
      vacia.hidden = false;
      contenido.hidden = true;
      return;
    }
    vacia.hidden = true;
    contenido.hidden = false;
    const foto = $("ficha-foto");
    foto.src = FOTO_ZONA[m.zona] || FOTO_ZONA.General;
    foto.alt = `Mesa ${m.id} preparada en la zona ${m.zona}`;
    $("ficha-titulo").textContent = `M${m.id}`;
    $("ficha-zona").textContent = `Zona ${m.zona}`;
    $("ficha-capacidad").textContent = `Capacidad de la mesa: ${m.capacidad} puestos.`;
    const sel = $("ficha-comensales");
    const total = Math.max(capacidadSeleccionada(), 1);
    const comensales = Number($("comensales").value) || 2;
    sel.innerHTML = "";
    for (let n = 1; n <= total; n++) {
      const o = document.createElement("option");
      o.value = String(n);
      o.textContent = n === 1 ? "1 persona" : `${n} personas`;
      sel.appendChild(o);
    }
    sel.value = String(Math.min(Math.max(comensales, 1), total));
  }

  function resumenSeleccion() {
    const texto = $("bloqueo-aviso");
    if (!texto) return;
    const comensales = Number($("comensales").value || 0);
    const capacidad = capacidadSeleccionada();
    if (seleccion.size === 0) {
      texto.textContent = "Ninguna mesa seleccionada todavía.";
    } else {
      texto.textContent =
        `Mesas ${[...seleccion].join(", ")} — capacidad ${capacidad} ` +
        `para ${comensales} comensales.`;
    }
    $("btn-paso3").disabled = !(seleccion.size > 0 && capacidad >= comensales);
    const sel = $("ficha-comensales");
    if (sel && sel.options.length > 0 && seleccion.size > 0) {
      sel.value = String(Math.min(Math.max(comensales, 1), sel.options.length));
    }
  }

  // --- plano -----------------------------------------------------------------

  function pintarPlano() {
    const plano = $("plano");
    if (!plano) return;
    plano.innerHTML = "";
    const svg = el("svg", {
      viewBox: "0 0 900 620",
      role: "group",
      "aria-label": "Plano del salón con las mesas disponibles por zona",
    });
    dibujarSalon(svg);
    for (const z of ZONAS) {
      svg.appendChild(texto(z.etiqueta, z.etiquetaX, z.etiquetaY, "plano-texto", z.ancla));
      const deZona = mesasVisibles.filter((m) => m.zona === z.nombre);
      deZona.forEach((m, i) => {
        const slot = z.slots[i];
        if (slot) svg.appendChild(dibujarMesa(m, slot[0], slot[1]));
      });
    }
    plano.appendChild(svg);
    resumenSeleccion();
    actualizarFicha();
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
      if (!seleccion.has(ultimaMesa)) ultimaMesa = null;
      pintarPlano();
    } catch {
      aviso("No se pudo cargar el plano de mesas.");
    }
  }

  // --- paso 3 y 4 --------------------------------------------------------------

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
    const el2 = $("total-estimado");
    if (!el2) return;
    const cop = new Intl.NumberFormat("es-CO", {
      style: "currency",
      currency: "COP",
      maximumFractionDigits: 0,
    });
    el2.textContent =
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
    const selFicha = $("ficha-comensales");
    if (selFicha) {
      selFicha.addEventListener("change", () => {
        $("comensales").value = selFicha.value;
        resumenSeleccion();
      });
    }
    const btnFicha = $("ficha-continuar");
    if (btnFicha) {
      btnFicha.addEventListener("click", () => {
        const btn = $("btn-paso3");
        if (btn.disabled) {
          aviso("La selección aún no cubre a todos los comensales.");
          return;
        }
        btn.click();
      });
    }
    for (const i of document.querySelectorAll("input.extra")) {
      i.addEventListener("change", pintarEstimado);
    }
    for (const i of document.querySelectorAll("input[data-precio]")) {
      i.addEventListener("input", pintarEstimado);
    }
    $("form-reserva").addEventListener("submit", sincronizarOcultos);
    pintarPlano();
    mostrarPaso(1);
  }

  document.addEventListener("DOMContentLoaded", iniciar);
})();
