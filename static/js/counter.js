// counter.js — port vanilla del componente Counter de React Bits
// (https://reactbits.dev, licencia MIT): cifras que ruedan con amortiguación
// de muelle, escalonado desde la derecha, desenfoque de movimiento y giro de
// entrada al aparecer en pantalla. Adaptado a Brindis:
//  - solo enteros (la cuenta regresiva no usa decimales ni moneda);
//  - los separadores (miles) son estáticos, sin animación de entrada/salida;
//  - cada cifra usa un medidor fijo ("0") con `tabular-nums` en CSS.
//
// Uso por atributos en un elemento [data-contador]:
//   data-valor, data-min-digitos, data-duracion, data-rebote, data-escalonado,
//   data-direccion (auto | up | down | shortest), data-intro (false lo apaga),
//   data-locale (por defecto es-CO).
//
// Cuenta regresiva: un elemento [data-cuenta] con data-objetivo (ISO 8601 con
// desfase) y campos [data-cuenta-unidad="dias|horas|min|seg"] que a su vez son
// [data-contador]. Al llegar a cero el panel recibe la clase `cuenta-llego`.
(function () {
  "use strict";

  const TAU = Math.PI * 2;
  const clamp = (v, min, max) => Math.min(max, Math.max(min, v));
  const mod = (v, base) => ((v % base) + base) % base;
  const envolver = (offset) => mod(offset + 5, 10) - 5;
  const SVG_NS = "http://www.w3.org/2000/svg";

  const prefiereReducir = () =>
    !!(window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches);

  const formatear = (valor, locale, minDigitos) =>
    new Intl.NumberFormat(locale, {
      useGrouping: true,
      minimumIntegerDigits: clamp(Math.round(minDigitos), 1, 21),
    }).formatToParts(Math.round(valor));

  const numeroParaLectores = (valor, locale) =>
    new Intl.NumberFormat(locale).format(Math.round(valor));

  const cifrasDelLocale = (locale) => {
    const formato = new Intl.NumberFormat(locale, { useGrouping: false });
    return Array.from({ length: 10 }, (_, n) => formato.format(n));
  };

  // Cuánto avanza una cifra al cambiar de valor según la dirección.
  const pasoEntre = (desde, hasta, direccion) => {
    const arriba = mod(hasta - desde, 10);
    if (direccion === "up") return arriba;
    if (direccion === "down") return arriba === 0 ? 0 : arriba - 10;
    return arriba > 5 ? arriba - 10 : arriba;
  };

  // Partes del número -> ranuras (cifras con su lugar y separadores).
  // El lugar cuenta desde la derecha: 0 unidades, 1 decenas, ...
  const slotsDe = (partes, cifras) => {
    let lugar = 0;
    for (const parte of partes) {
      if (parte.type === "integer") lugar += [...parte.value].length;
    }
    const slots = [];
    for (const parte of partes) {
      if (parte.type === "integer") {
        for (const char of parte.value) {
          lugar -= 1;
          slots.push({
            tipo: "digito",
            digito: Math.max(0, cifras.indexOf(char)),
            rank: -2 * lugar,
          });
        }
      } else if (parte.type === "group") {
        slots.push({ tipo: "simbolo", texto: parte.value, rank: 0 });
      }
    }
    return slots;
  };

  let secuencia = 0; // ids únicos para los filtros SVG de desenfoque
  const registrados = new Map();

  function crearContador(nodo) {
    const locale = nodo.dataset.locale || "es-CO";
    const minDigitos = Number(nodo.dataset.minDigitos || 1);
    const duracion = Math.max(0.05, Number(nodo.dataset.duracion || 0.7));
    const rebote = clamp(Number(nodo.dataset.rebote || 0.2), 0, 0.9);
    const escalonado = Math.max(0, Number(nodo.dataset.escalonado || 0.04));
    const direccion = ["auto", "up", "down", "shortest"].includes(nodo.dataset.direccion)
      ? nodo.dataset.direccion
      : "auto";
    const conIntro = nodo.dataset.intro !== "false";
    const cifras = cifrasDelLocale(locale);

    nodo.classList.add("contador");
    const sr = document.createElement("span");
    sr.className = "contador__sr";
    const fila = document.createElement("span");
    fila.className = "contador__fila";
    nodo.appendChild(sr);
    nodo.appendChild(fila);

    const items = []; // una por cifra visible
    let em = 16;
    let raf = 0;
    let ultimo = 0;
    let valorActual = Number(nodo.dataset.valor || 0);

    const medir = () => {
      em = parseFloat(window.getComputedStyle(nodo).fontSize) || 16;
    };

    function construirItems(slots) {
      items.length = 0;
      fila.textContent = "";
      for (const slot of slots) {
        if (slot.tipo === "simbolo") {
          const simbolo = document.createElement("span");
          simbolo.className = "contador__simbolo";
          simbolo.textContent = slot.texto;
          fila.appendChild(simbolo);
          continue;
        }
        const caja = document.createElement("span");
        caja.className = "contador__digito";
        const medida = document.createElement("span");
        medida.className = "contador__medida";
        medida.textContent = cifras[0];
        const ventana = document.createElement("span");
        ventana.className = "contador__ventana";
        const tira = document.createElement("span");
        tira.className = "contador__tira";
        const nodosCifra = cifras.map((char, n) => {
          const cifra = document.createElement("span");
          cifra.className = "contador__cifra";
          cifra.textContent = char;
          cifra.style.transform = `translateY(${envolver(n - slot.digito) * 100}%)`;
          tira.appendChild(cifra);
          return cifra;
        });
        ventana.appendChild(tira);
        const svg = document.createElementNS(SVG_NS, "svg");
        svg.setAttribute("class", "contador__desenfoque");
        svg.setAttribute("aria-hidden", "true");
        const filtro = document.createElementNS(SVG_NS, "filter");
        const filtroId = `contador-desenfoque-${secuencia++}`;
        filtro.setAttribute("id", filtroId);
        filtro.setAttribute("x", "-10%");
        filtro.setAttribute("y", "-50%");
        filtro.setAttribute("width", "120%");
        filtro.setAttribute("height", "200%");
        filtro.setAttribute("color-interpolation-filters", "sRGB");
        const blur = document.createElementNS(SVG_NS, "feGaussianBlur");
        blur.setAttribute("stdDeviation", "0 0");
        filtro.appendChild(blur);
        svg.appendChild(filtro);
        caja.appendChild(medida);
        caja.appendChild(ventana);
        caja.appendChild(svg);
        fila.appendChild(caja);
        items.push({
          pos: slot.digito,
          vel: 0,
          target: slot.digito,
          rank: slot.rank,
          espera: 0,
          cifras: nodosCifra,
          tira,
          blur,
          filtroId,
          desenfocado: false,
        });
      }
    }

    function pintar(item) {
      for (let n = 0; n < item.cifras.length; n++) {
        item.cifras[n].style.transform = `translateY(${(envolver(n - item.pos) * 100).toFixed(3)}%)`;
      }
      const radio = clamp((Math.abs(item.vel) - 3) * 0.006, 0, 0.09) * em;
      if (radio > 0.25) {
        item.blur.setAttribute("stdDeviation", `0 ${radio.toFixed(2)}`);
        if (!item.desenfocado) {
          item.tira.style.filter = `url(#${item.filtroId})`;
          item.desenfocado = true;
        }
      } else if (item.desenfocado) {
        item.tira.style.filter = "";
        item.desenfocado = false;
      }
    }

    function paso(ahora) {
      raf = 0;
      const transcurrido = clamp((ahora - ultimo) / 1000, 0, 1 / 20);
      ultimo = ahora;
      const reducido = prefiereReducir();
      const omega = TAU / duracion;
      const zeta = 1 - rebote;
      let ocupado = false;
      for (const item of items) {
        if (reducido) {
          item.espera = 0;
          item.pos = item.target;
          item.vel = 0;
        } else {
          let restante = transcurrido;
          if (item.espera > 0) {
            const usado = Math.min(item.espera, restante);
            item.espera -= usado;
            restante -= usado;
          }
          if (restante > 0) {
            const subpasos = Math.ceil(restante * 240);
            const dt = restante / subpasos;
            for (let i = 0; i < subpasos; i++) {
              item.vel += (-omega * omega * (item.pos - item.target) - 2 * zeta * omega * item.vel) * dt;
              item.pos += item.vel * dt;
            }
          }
        }
        const asentado =
          item.espera <= 0 &&
          Math.abs(item.pos - item.target) < 0.002 &&
          Math.abs(item.vel) < 0.02;
        if (asentado) {
          item.target = mod(Math.round(item.target), 10);
          item.pos = item.target;
          item.vel = 0;
        } else {
          ocupado = true;
        }
        pintar(item);
      }
      if (ocupado) raf = window.requestAnimationFrame(paso);
    }

    function despertar() {
      if (!raf) {
        ultimo = performance.now();
        raf = window.requestAnimationFrame(paso);
      }
    }

    function actualizar(nuevo) {
      const anterior = valorActual;
      valorActual = Math.round(nuevo);
      sr.textContent = numeroParaLectores(valorActual, locale);
      const partes = formatear(valorActual, locale, minDigitos);
      const slots = slotsDe(partes, cifras);
      const digitos = slots.filter((s) => s.tipo === "digito");
      if (digitos.length !== items.length) {
        // Cambió la cantidad de cifras: se reconstruye sin animación.
        construirItems(slots);
        return;
      }
      const tendencia =
        direccion !== "auto"
          ? direccion
          : Math.abs(valorActual) > Math.abs(anterior)
            ? "up"
            : Math.abs(valorActual) < Math.abs(anterior)
              ? "down"
              : "shortest";
      const moviendo = [];
      items.forEach((item, i) => {
        const slot = digitos[i];
        item.rank = slot.rank;
        const avance = pasoEntre(mod(Math.round(item.target), 10), slot.digito, tendencia);
        if (avance !== 0) {
          item.target += avance;
          moviendo.push(item);
        }
      });
      // Escalonado desde la derecha: la cifra de las unidades arranca primero.
      moviendo
        .sort((a, b) => b.rank - a.rank)
        .forEach((item, orden) => {
          item.espera = orden * escalonado;
        });
      despertar();
    }

    medir();
    const partes = formatear(valorActual, locale, minDigitos);
    construirItems(slotsDe(partes, cifras));
    sr.textContent = numeroParaLectores(valorActual, locale);

    // Giro de entrada: cada cifra da una vuelta completa y cae en su lugar
    // la primera vez que el contador aparece en pantalla.
    if (conIntro && !prefiereReducir()) {
      const girar = () => {
        const giro = direccion === "down" ? -10 : 10;
        [...items]
          .sort((a, b) => a.rank - b.rank)
          .forEach((item, orden) => {
            item.target += giro;
            item.espera = orden * escalonado;
          });
        despertar();
      };
      if (typeof IntersectionObserver === "undefined") {
        girar();
      } else {
        const observador = new IntersectionObserver(
          (entradas) => {
            if (!entradas.some((x) => x.isIntersecting)) return;
            observador.disconnect();
            girar();
          },
          { threshold: 0.5 }
        );
        observador.observe(nodo);
      }
    }

    registrados.set(nodo, { actualizar, medir });
  }

  function iniciarCuentas() {
    for (const panel of document.querySelectorAll("[data-cuenta]")) {
      const objetivo = Date.parse(panel.dataset.objetivo || "");
      if (Number.isNaN(objetivo)) continue;
      const campo = (unidad) => panel.querySelector(`[data-cuenta-unidad="${unidad}"]`);
      const campos = {
        dias: campo("dias"),
        horas: campo("horas"),
        min: campo("min"),
        seg: campo("seg"),
      };
      const poner = (nodo, valor) => {
        const registrado = nodo && registrados.get(nodo);
        if (registrado) registrado.actualizar(valor);
      };
      let reloj = 0;
      const latido = () => {
        const restante = Math.max(0, objetivo - Date.now());
        const totalSeg = Math.floor(restante / 1000);
        poner(campos.dias, Math.floor(totalSeg / 86400));
        poner(campos.horas, Math.floor(totalSeg / 3600) % 24);
        poner(campos.min, Math.floor(totalSeg / 60) % 60);
        poner(campos.seg, totalSeg % 60);
        if (restante <= 0) {
          panel.classList.add("cuenta-llego");
          if (reloj) {
            window.clearInterval(reloj);
            reloj = 0;
          }
        }
      };
      latido();
      reloj = window.setInterval(latido, 1000);
    }
  }

  function iniciar() {
    for (const nodo of document.querySelectorAll("[data-contador]")) {
      if (!registrados.has(nodo)) crearContador(nodo);
    }
    iniciarCuentas();
    if (document.fonts && document.fonts.ready) {
      document.fonts.ready
        .then(() => {
          for (const registrado of registrados.values()) registrado.medir();
        })
        .catch(() => {});
    }
    if (typeof ResizeObserver !== "undefined") {
      const observador = new ResizeObserver(() => {
        for (const registrado of registrados.values()) registrado.medir();
      });
      for (const nodo of registrados.keys()) observador.observe(nodo);
    }
  }

  document.addEventListener("DOMContentLoaded", iniciar);
})();
