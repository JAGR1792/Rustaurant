// plano.js original Brindis: pinta zonas y consulta disponibilidad sin recargar.
// Idea vista en Camilo (reservar.js) y Proyecto_IA (hooks de fetch),
// reescrita: fetch GET /api/disponibilidad?fecha=&hora=&zona= .
async function cargarPlano(fecha, hora, zona) {
  const r = await fetch(
    `/api/disponibilidad?fecha=${fecha}&hora=${hora}&zona=${zona}`);
  const mesas = await r.json(); // [{id, zona, capacidad, estado}]
  const plano = document.getElementById("plano");
  plano.innerHTML = "";
  for (const m of mesas) {
    const b = document.createElement("button");
    b.className = "mesa";
    b.dataset.estado = m.estado; // libre|reservada|bloqueada|seleccionada
    b.textContent = `M${m.id} · ${m.capacidad}`;
    b.setAttribute("aria-label", `Mesa ${m.id}, ${m.estado}`);
    b.disabled = m.estado === "reservada" || m.estado === "bloqueada";
    plano.appendChild(b);
  }
}
