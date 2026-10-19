// chroma.js — efecto "ChromaGrid" adaptado a JavaScript vanilla (sin React).
// Idea y diseno originales: ChromaGrid de React Bits (https://reactbits.dev),
// licencia MIT. Este port es propio de Brindis: misma mecanica — un spotlight
// radial que sigue al cursor con amortiguacion (lerp en requestAnimationFrame)
// y enmascara la capa a color — escrito para el stack Askama + vanilla del repo.
//
// Uso: un contenedor con [data-chroma-grid] y tarjetas que incluyan
// [data-chroma-area] (la tarjeta y su .chroma-media). Opciones por atributos:
//   data-damping (0.45), data-fade-out (0.6), data-softness (0.6),
//   data-reveal ("spotlight" | "card").
(function () {
  "use strict";

  const clamp = (v, min, max) => Math.min(max, Math.max(min, v));

  // Gradiente de caida del spotlight (mismo polinomio que el original).
  function stopsDe(softness) {
    const core = clamp(1 - softness, 0, 0.95);
    const stops = [];
    for (let i = 0; i <= 12; i++) {
      const t = i / 12;
      const u = clamp((t - core) / Math.max(0.0001, 1 - core), 0, 1);
      const alpha = 1 - u * u * u * (u * (u * 6 - 15) + 10);
      stops.push(`rgba(0, 0, 0, ${alpha.toFixed(3)}) ${(t * 100).toFixed(1)}%`);
    }
    return stops.join(", ");
  }

  function iniciarGrid(root) {
    const reduce =
      window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const damping = Number(root.dataset.damping || 0.45);
    const fadeOut = Number(root.dataset.fadeOut || 0.6);
    const spotlight = root.dataset.reveal !== "card";
    const softness = clamp(Number(root.dataset.softness || 0.6), 0, 1);

    root.style.setProperty("--chroma-stops", stopsDe(softness));

    const estado = { x: 0, y: 0, tx: 0, ty: 0, power: 0, target: 0, primed: false };
    let raf = 0;
    let ultimo = 0;

    // Posiciona cada area del grid en coordenadas del contenedor: las
    // mascaras de cada pieza restan su propio offset (--chroma-ox/oy).
    function colocar() {
      const caja = root.getBoundingClientRect();
      for (const nodo of root.querySelectorAll("[data-chroma-area]")) {
        const r = nodo.getBoundingClientRect();
        nodo.style.setProperty("--chroma-ox", `${r.left - caja.left}px`);
        nodo.style.setProperty("--chroma-oy", `${r.top - caja.top}px`);
      }
    }

    function paso(ahora) {
      raf = 0;
      const dt = ultimo ? Math.min(0.05, (ahora - ultimo) / 1000) : 1 / 60;
      ultimo = ahora;
      const seguir = reduce ? 1 : 1 - Math.exp(-dt / Math.max(0.01, damping * 0.32));
      estado.x += (estado.tx - estado.x) * seguir;
      estado.y += (estado.ty - estado.y) * seguir;
      const span = estado.target > estado.power ? 0.1 : Math.max(0.02, fadeOut * 0.35);
      estado.power += (estado.target - estado.power) * (reduce ? 1 : 1 - Math.exp(-dt / span));
      root.style.setProperty("--chroma-x", `${estado.x.toFixed(2)}px`);
      root.style.setProperty("--chroma-y", `${estado.y.toFixed(2)}px`);
      root.style.setProperty("--chroma-power", estado.power.toFixed(4));
      const asentado =
        Math.abs(estado.tx - estado.x) < 0.1 &&
        Math.abs(estado.ty - estado.y) < 0.1 &&
        Math.abs(estado.target - estado.power) < 0.002;
      if (asentado) {
        estado.power = estado.target;
        root.style.setProperty("--chroma-power", estado.power.toFixed(4));
        ultimo = 0;
      } else {
        raf = requestAnimationFrame(paso);
      }
    }

    function despertar() {
      if (!raf) raf = requestAnimationFrame(paso);
    }

    function alMover(evento) {
      if (!spotlight) return;
      const caja = root.getBoundingClientRect();
      estado.tx = evento.clientX - caja.left;
      estado.ty = evento.clientY - caja.top;
      if (!estado.primed || estado.power < 0.02) {
        estado.x = estado.tx;
        estado.y = estado.ty;
        estado.primed = true;
      }
      estado.target = 1;
      despertar();
    }

    function alSalir() {
      estado.target = 0;
      despertar();
    }

    const observador = new ResizeObserver(colocar);
    observador.observe(root);
    root.addEventListener("pointermove", alMover);
    root.addEventListener("pointerdown", alMover);
    root.addEventListener("pointerleave", alSalir);
    root.addEventListener("pointercancel", alSalir);
    colocar();
  }

  function iniciar() {
    for (const grid of document.querySelectorAll("[data-chroma-grid]")) {
      iniciarGrid(grid);
    }
  }

  document.addEventListener("DOMContentLoaded", iniciar);
})();
