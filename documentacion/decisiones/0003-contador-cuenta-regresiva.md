# ADR 0003: Cuenta regresiva de la próxima reserva (port del Counter)

- Estado: aceptado
- Fecha: 2026-10-16

## Contexto

Jorge pidió que después de reservar aparezca un contador de "cuánto falta"
para la reserva, con el componente Counter de React Bits (cifras que ruedan
con animación por dígito). El repo es Askama + JS vanilla y el ADR 0001
prohíbe frameworks y copiar código ajeno.

## Decisión

Portar la mecánica a código propio vanilla: `static/js/counter.js` (por
cada cifra, una tira 0-9 dentro de una ventana con máscara de desvanecido;
integración de muelle con subpasos; escalonado desde la derecha; desenfoque
de movimiento por filtro SVG; giro de entrada con IntersectionObserver) y
`static/css/counter.css` (ventana, máscara, panel de cuenta regresiva con
la paleta de marca). El panel aparece en `mis_reservas.html` solo para
clientes, con la reserva vigente más cercana (`vistas.rs::proxima_reserva`);
el objetivo viaja como ISO 8601 con desfase UTC-5 (`fecha_iso_local`) y el
navegador calcula el resto. El diseño original se acredita a React Bits
(MIT) en el encabezado de ambos archivos.

Simplificaciones del port: solo enteros, separadores estáticos y
reconstrucción sin animación si cambia la cantidad de cifras.

## Consecuencias

- Sin dependencias ni build step; sin JS el panel no se dibuja (mejora
  progresiva, igual que el efecto de la carta del ADR 0002).
- `prefers-reduced-motion` desactiva giros y resortes: los valores se
  pintan directo.
- La selección de "próxima reserva" es lógica de presentación testeada en
  `vistas.rs`; las reglas de negocio siguen en dominio y servicios.
