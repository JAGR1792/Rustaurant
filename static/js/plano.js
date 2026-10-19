// plano.js — asistente de reserva en 4 pasos: plano SVG del salón por zonas,
// ficha lateral de la mesa elegida, pedido guiado por porciones (steppers,
// «toda la mesa» y cobertura) y revisión viva del paso 4. Consulta
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
    { nombre: "VIP", etiqueta: "ZONA ÍNTIMA · VIP", etiquetaX: 54, etiquetaY: 464, ancla: "start", slots: [[150, 536], [300, 536]] },
  ];

  const FOTO_ZONA = {
    Terraza: {
      jpg: "/static/img/fotos/mesa-terraza.jpg",
      webp: "/static/img/fotos/mesa-terraza.webp",
    },
    General: {
      jpg: "/static/img/fotos/mesa-general.jpg",
      webp: "/static/img/fotos/mesa-general.webp",
    },
    VIP: {
      jpg: "/static/img/fotos/mesa-vip.jpg",
      webp: "/static/img/fotos/mesa-vip.webp",
    },
  };

  const ESTADO_TEXTO = { libre: "libre", reservada: "reservada", bloqueada: "bloqueada", seleccionada: "seleccionada" };

  const COLORES = {
    libre: { cuerpo: "#e7d9bf", borde: "#c9b795", texto: "#26231f", silla: "#9c8a6d" },
    seleccionada: { cuerpo: "#702c3b", borde: "#a88a59", texto: "#fdf9f1", silla: "#cbb48e", grosor: 3.5 },
    reservada: { cuerpo: "#ddd5c9", borde: "#c4bbac", texto: "#7a7264", silla: "#b5ac9d" },
    bloqueada: { cuerpo: "#cfc7b8", borde: "#9b9282", texto: "#6d675e", silla: "#a89f90", guion: "5 4" },
  };

  function avisoSeleccion() {
    const t = $("bloqueo-aviso");
    if (!t) return;
    t.textContent = `La selección no cubre a los ${comensales()} comensales: agrega otra mesa libre o ajusta el número.`;
    t.classList.add("texto-alerta");
  }

  /// Errores de validación en línea (sin toasts): texto junto al campo.
  function mostrarErrorPaso1(msg) {
    const p = $("error-paso1");
    if (!p) return;
    p.textContent = msg;
    p.hidden = false;
  }

  function limpiarErrorPaso1() {
    const p = $("error-paso1");
    if (!p) return;
    p.textContent = "";
    p.hidden = true;
  }

  let pasoActual = 1;

  function mostrarPaso(n) {
    pasoActual = n;
    for (const s of document.querySelectorAll("section[id^='paso-']")) {
      s.hidden = s.id !== `paso-${n}`;
    }
    for (const li of document.querySelectorAll(".progreso li")) {
      const paso = Number(li.dataset.paso);
      if (paso === n) li.setAttribute("aria-current", "step");
      else li.removeAttribute("aria-current");
      li.classList.toggle("completado", paso < n);
    }
    actualizarStepper();
    const h = document.querySelector(`#paso-${n} h2`);
    if (h && typeof h.focus === "function") h.focus();
  }

  // Stepper navegable: volver a lo visitado siempre; avanzar un paso
  // solo cuando el paso actual quedó completo.
  function actualizarStepper() {
    const ocasion = !!document.querySelector("input[name=ocasion]:checked");
    const listo = seleccion.size > 0 && capacidadSeleccionada() >= comensales();
    const completo = { 2: ocasion, 3: listo, 4: true };
    for (const b of document.querySelectorAll(".progreso button[data-ir]")) {
      const p = Number(b.dataset.ir);
      const volver = p <= pasoActual;
      const avanzar = p === pasoActual + 1 && completo[p];
      b.disabled = !(volver || avanzar);
    }
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

  function ventanal(svg, x0, y0, ancho) {
    svg.appendChild(el("rect", { x: x0, y: y0, width: ancho, height: 13, rx: 6.5, fill: "#f6eedd", stroke: "#c9b795", "stroke-width": 1.8 }));
    for (let x = x0 + ancho / 4; x < x0 + ancho - 8; x += ancho / 4) {
      svg.appendChild(el("line", { x1: x, y1: y0 + 1.5, x2: x, y2: y0 + 11.5, stroke: "#dccdaa", "stroke-width": 1.4 }));
    }
  }

  function planta(svg, x, y) {
    svg.appendChild(el("rect", { x: x - 7, y: y + 3, width: 14, height: 10, rx: 3, fill: "#d9c9a8", stroke: "#b9a37a", "stroke-width": 1.4 }));
    for (const [dx, dy, r] of [[-6, -7, 8], [6, -7, 8], [0, -13, 9]]) {
      svg.appendChild(el("circle", { cx: x + dx, cy: y + dy, r, fill: "#93ab89", stroke: "#7e9874", "stroke-width": 1 }));
    }
  }

  function dibujarSalon(svg) {
    // Piso y muro perimetral (trazo doble suave, estilo plano de salón).
    svg.appendChild(el("rect", { x: 20, y: 20, width: 860, height: 580, rx: 22, fill: "#fbf7f0", stroke: "#c9b795", "stroke-width": 3.5 }));
    svg.appendChild(el("rect", { x: 29, y: 29, width: 842, height: 562, rx: 15, fill: "none", stroke: "#e9ddc7", "stroke-width": 1.5 }));
    // Lavados de zona: terraza, rincón VIP y barra.
    svg.appendChild(el("rect", { x: 34, y: 34, width: 832, height: 178, rx: 14, fill: "#f8f1e2" }));
    svg.appendChild(el("rect", { x: 34, y: 440, width: 372, height: 152, rx: 12, fill: "#f4ecdc" }));
    svg.appendChild(el("rect", { x: 628, y: 484, width: 244, height: 112, rx: 14, fill: "#f6edda" }));
    // Ventanales del muro norte.
    ventanal(svg, 150, 13, 230);
    ventanal(svg, 400, 13, 230);
    svg.appendChild(texto("VENTANALES", 836, 24, "plano-nota", "end"));
    // Plantas de interior junto a los ventanales.
    planta(svg, 108, 58);
    planta(svg, 356, 58);
    planta(svg, 654, 58);
    // Pasillo central punteado.
    muro(svg, 36, 214, 368, 214, true);
    muro(svg, 532, 214, 864, 214, true);
    svg.appendChild(texto("PASILLO", 450, 219, "plano-texto", "middle"));
    // Rincón VIP: media pared superior y borde tenue a la derecha.
    muro(svg, 34, 440, 406, 440, false);
    svg.appendChild(el("line", { x1: 406, y1: 440, x2: 406, y2: 592, stroke: "#e6dac2", "stroke-width": 1.5, "stroke-dasharray": "4 6" }));
    // Barra: frente, superficie y banquetas.
    svg.appendChild(el("rect", { x: 640, y: 496, width: 220, height: 88, rx: 12, fill: "#efe4d1", stroke: "#c9b795", "stroke-width": 1.8 }));
    svg.appendChild(el("line", { x1: 648, y1: 510, x2: 852, y2: 510, stroke: "#e3d5b9", "stroke-width": 1.4 }));
    for (const x of [682, 732, 782]) {
      svg.appendChild(el("circle", { cx: x, cy: 492, r: 8, fill: "#dcccaa", stroke: "#c9b795", "stroke-width": 1.6 }));
    }
    svg.appendChild(texto("BAR", 750, 552, "plano-texto", "middle"));
    // Entrada: vano en el muro sur, hoja abierta y arco de barrido.
    svg.appendChild(el("rect", { x: 436, y: 594, width: 108, height: 16, fill: "#fbf7f0" }));
    svg.appendChild(el("line", { x1: 436, y1: 598, x2: 436, y2: 490, stroke: "#c9b795", "stroke-width": 2.5 }));
    svg.appendChild(el("path", { d: "M 436 490 A 108 108 0 0 1 544 598", fill: "none", stroke: "#d9c9a8", "stroke-width": 1.4, "stroke-dasharray": "3 5" }));
    svg.appendChild(texto("ENTRADA", 490, 588, "plano-nota", "middle"));
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
    // Anillo exterior cuando la mesa está elegida.
    if (elegida) {
      g.appendChild(el("circle", { cx, cy, r: 41, fill: "none", stroke: "#a88a59", "stroke-width": 1.5, opacity: 0.6 }));
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
    // Sombra suave asentando la mesa en el piso.
    g.appendChild(el("circle", { cx: cx + 2, cy: cy + 3.5, r: 34, fill: "rgba(38, 35, 31, 0.10)" }));
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
    const foto = FOTO_ZONA[m.zona] || FOTO_ZONA.General;
    const img = $("ficha-foto");
    img.src = foto.jpg;
    const source = $("ficha-webp");
    if (source) source.srcset = foto.webp;
    img.alt = `Mesa ${m.id} preparada en la zona ${m.zona}`;
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
    const listo = seleccion.size > 0 && capacidad >= comensales;
    if (seleccion.size === 0) {
      texto.textContent = "Ninguna mesa seleccionada todavía.";
      texto.classList.remove("texto-alerta");
    } else {
      texto.textContent =
        `Mesas ${[...seleccion].join(", ")} — capacidad ${capacidad} ` +
        `para ${comensales} comensales.` +
        (listo ? "" : " Falta capacidad: agrega otra mesa libre.");
      texto.classList.toggle("texto-alerta", !listo);
    }
    $("btn-paso3").disabled = !listo;
    const sel = $("ficha-comensales");
    if (sel && sel.options.length > 0 && seleccion.size > 0) {
      sel.value = String(Math.min(Math.max(comensales, 1), sel.options.length));
    }
    pintarPedido();
    actualizarStepper();
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

  let retrasoPlano = null;

  // La recarga se programa con un respiro para no disparar una consulta
  // por cada tecla o clic en los campos de fecha, hora y zona.
  function programarCargaPlano() {
    clearTimeout(retrasoPlano);
    retrasoPlano = setTimeout(cargarPlano, 350);
  }

  async function cargarPlano() {
    const fecha = $("fecha").value;
    const hora = $("hora").value;
    const zona = $("zona").value;
    const estado = $("plano-estado");
    if (!fecha || !hora) {
      if (estado) {
        estado.textContent = "Elige fecha y hora para consultar la disponibilidad.";
        estado.classList.add("plano-estado--error");
      }
      return;
    }
    if (estado) {
      estado.textContent = "Buscando disponibilidad…";
      estado.classList.remove("plano-estado--error");
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
      if (estado) {
        estado.textContent = "Disponibilidad actualizada: elige tus mesas en el plano.";
      }
    } catch {
      if (estado) {
        estado.textContent = "No se pudo cargar el plano: revisa la fecha y la hora e inténtalo de nuevo.";
        estado.classList.add("plano-estado--error");
      }
    }
  }

  // --- paso 3 y 4 --------------------------------------------------------------

  const copFmt = new Intl.NumberFormat("es-CO", {
    style: "currency",
    currency: "COP",
    maximumFractionDigits: 0,
  });

  function comensales() {
    return Math.max(Number($("comensales").value) || 1, 1);
  }

  function porcionesPedidas() {
    let total = 0;
    for (const i of document.querySelectorAll("input[data-precio]")) {
      total += Number(i.value || 0);
    }
    return total;
  }

  function estimarTotal() {
    let total = 0;
    for (const i of document.querySelectorAll("input[data-precio]")) {
      total += Number(i.dataset.precio) * Number(i.value || 0);
    }
    const n = comensales();
    for (const i of document.querySelectorAll("input.extra:checked")) {
      const tarifa = Number(i.dataset.tarifa);
      total += i.dataset.porpersona === "1" ? tarifa * n : tarifa;
    }
    return total;
  }

  // Pedido guiado: cobertura de la mesa, botones «toda la mesa» y costos vivos.
  function pintarPedido() {
    if (!$("cobertura-texto")) return;
    const n = comensales();
    const porciones = porcionesPedidas();
    $("cobertura-texto").textContent =
      porciones === 0
        ? `Mesa para ${n}: todavía sin pedido anticipado — puedes reservar y pedir en el salón.`
        : `Pedido de ${porciones} ${porciones === 1 ? "porción" : "porciones"} · cubre ${Math.min(porciones, n)} de ${n} comensales.`;
    const fill = $("cobertura-fill");
    if (fill) fill.style.width = `${Math.min((porciones / n) * 100, 100)}%`;
    for (const b of document.querySelectorAll(".todo-mesa")) {
      b.textContent = `Toda la mesa (${n})`;
    }
    for (const i of document.querySelectorAll("input.extra")) {
      const costo = i.closest(".extra-linea")?.querySelector("[data-costo]");
      if (!costo) continue;
      if (i.checked) {
        const tarifa = Number(i.dataset.tarifa);
        const monto = i.dataset.porpersona === "1" ? tarifa * n : tarifa;
        costo.textContent = i.dataset.porpersona === "1" ? `× ${n} = ${copFmt.format(monto)}` : copFmt.format(monto);
      } else {
        costo.textContent = "";
      }
    }
    pintarEstimado();
  }

  function pintarEstimado() {
    const destino = $("total-estimado");
    if (!destino) return;
    destino.textContent =
      `Total estimado de pedido y extras: ${copFmt.format(estimarTotal())}. ` +
      "El valor final se recalcula en el servidor.";
  }

  // Revisión del paso 4: datos, pedido desglosado y monto según modalidad.
  function pintarResumen() {
    const datos = $("resumen-datos");
    const pedido = $("resumen-pedido");
    if (!datos || !pedido) return;
    const n = comensales();
    const fecha = $("fecha").value;
    let fechaTxt = fecha || "—";
    if (fecha) {
      const d = new Date(`${fecha}T12:00:00`);
      if (!Number.isNaN(d.getTime())) {
        fechaTxt = d.toLocaleDateString("es-CO", { weekday: "long", day: "numeric", month: "long" });
      }
    }
    const mesa = mesaActual();
    const mesasTxt = seleccion.size
      ? [...seleccion].map((id) => `M${id}`).join(", ") + (mesa ? ` · ${mesa.zona}` : "")
      : "—";
    const ocasion =
      document.querySelector("input[name=ocasion]:checked")?.closest("label")?.querySelector("strong")?.textContent || "—";
    datos.innerHTML = "";
    for (const [clave, valor] of [
      ["Ocasión", ocasion.trim()],
      ["Fecha", fechaTxt],
      ["Hora", $("hora").value || "—"],
      ["Comensales", String(n)],
      ["Mesas", mesasTxt],
    ]) {
      const dt = document.createElement("dt");
      dt.textContent = clave;
      const dd = document.createElement("dd");
      dd.textContent = valor;
      datos.append(dt, dd);
    }
    const restricciones = document.querySelector("input[name=restricciones]")?.value.trim();
    if (restricciones) {
      const dt = document.createElement("dt");
      dt.textContent = "Restricciones";
      const dd = document.createElement("dd");
      dd.textContent = restricciones;
      datos.append(dt, dd);
    }

    const lineas = [];
    for (const i of document.querySelectorAll("input[data-precio]")) {
      const cantidad = Number(i.value || 0);
      if (cantidad > 0) {
        lineas.push(`${cantidad} × ${i.dataset.nombre} — ${copFmt.format(cantidad * Number(i.dataset.precio))}`);
      }
    }
    for (const i of document.querySelectorAll("input.extra:checked")) {
      const tarifa = Number(i.dataset.tarifa);
      const monto = i.dataset.porpersona === "1" ? tarifa * n : tarifa;
      const nombre = i.closest(".extra-linea")?.querySelector("strong")?.textContent || `Extra ${i.value}`;
      lineas.push(`${nombre} — ${copFmt.format(monto)}`);
    }
    pedido.innerHTML = "";
    if (lineas.length === 0) {
      const p = document.createElement("p");
      p.textContent = "Sin pedido anticipado: la mesa podrá ordenar en el salón.";
      pedido.appendChild(p);
    } else {
      const ul = document.createElement("ul");
      for (const linea of lineas) {
        const li = document.createElement("li");
        li.textContent = linea;
        ul.appendChild(li);
      }
      pedido.appendChild(ul);
    }

    const pago = $("resumen-pago");
    if (pago) {
      const modalidad = document.querySelector("input[name=modalidad]:checked")?.value;
      const total = estimarTotal();
      if (modalidad === "completo") {
        pago.textContent = `Pago completo en línea: ${copFmt.format(total)}.`;
      } else if (modalidad === "restaurante") {
        pago.textContent = `Pago en el restaurante: ${copFmt.format(total)} al finalizar.`;
      } else {
        // Misma cuenta que el dominio: total * porcentaje / 100 (entero).
        pago.textContent = `Adelanto del 30 %: ${copFmt.format(Math.floor((total * 30) / 100))} (total ${copFmt.format(total)}).`;
      }
    }
  }

  function iniciar() {
    if (!$("form-reserva")) return;

    // Stepper: se puede volver a cualquier paso ya recorrido.
    for (const b of document.querySelectorAll(".progreso button[data-ir]")) {
      b.addEventListener("click", () => {
        if (b.disabled) return;
        const ir = Number(b.dataset.ir);
        if (ir === 2) cargarPlano();
        if (ir === 3) {
          sincronizarOcultos();
          pintarPedido();
        }
        if (ir === 4) pintarResumen();
        mostrarPaso(ir);
      });
    }

    for (const b of document.querySelectorAll("[data-hacia]")) {
      b.addEventListener("click", () => {
        const destino = Number(b.dataset.hacia);
        if (destino === 2 && b.closest("#paso-1")) {
          if (!document.querySelector("input[name=ocasion]:checked")) {
            mostrarErrorPaso1("Elige la ocasión de tu reserva para continuar.");
            document.querySelector("input[name=ocasion]")?.focus();
            return;
          }
          limpiarErrorPaso1();
          mostrarPaso(2);
          cargarPlano();
          return;
        }
        if (destino === 3) {
          if (!(seleccion.size > 0 && capacidadSeleccionada() >= comensales())) {
            avisoSeleccion();
            return;
          }
          sincronizarOcultos();
          pintarPedido();
        }
        if (destino === 4) pintarResumen();
        if (destino === 2 && !b.closest("#paso-1")) cargarPlano();
        mostrarPaso(destino);
      });
    }

    // Ocasión: al cambiarla se limpia el error en línea y se recalcula el stepper.
    for (const r of document.querySelectorAll("input[name=ocasion]")) {
      r.addEventListener("change", () => {
        limpiarErrorPaso1();
        actualizarStepper();
      });
    }

    // El plano se recarga solo (con un respiro) al cambiar fecha, hora o zona.
    for (const id of ["fecha", "hora", "zona"]) {
      $(id).addEventListener("change", programarCargaPlano);
    }
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
          avisoSeleccion();
          return;
        }
        btn.click();
      });
    }
    for (const plato of document.querySelectorAll(".plato")) {
      const input = plato.querySelector("input[data-precio]");
      if (!input) continue;
      const fijar = (v) => {
        input.value = String(Math.max(0, Math.min(20, v)));
        pintarPedido();
      };
      plato.querySelector(".paso-menos")?.addEventListener("click", () => fijar(Number(input.value || 0) - 1));
      plato.querySelector(".paso-mas")?.addEventListener("click", () => fijar(Number(input.value || 0) + 1));
      input.addEventListener("input", pintarPedido);
      plato.querySelector(".todo-mesa")?.addEventListener("click", () => fijar(comensales()));
    }
    for (const i of document.querySelectorAll("input.extra")) {
      i.addEventListener("change", pintarPedido);
    }
    for (const r of document.querySelectorAll("input[name=modalidad]")) {
      r.addEventListener("change", pintarResumen);
    }
    const restr = document.querySelector("input[name=restricciones]");
    if (restr) restr.addEventListener("input", pintarResumen);
    $("form-reserva").addEventListener("submit", sincronizarOcultos);
    pintarPlano();
    pintarPedido();
    mostrarPaso(1);
  }

  document.addEventListener("DOMContentLoaded", iniciar);
})();
