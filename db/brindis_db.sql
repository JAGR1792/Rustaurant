IF DB_ID(N'brindis_db') IS NULL CREATE DATABASE brindis_db;
GO
USE brindis_db;
GO

-- ==========================================
-- MÓDULO 1: USUARIOS Y SEGURIDAD
-- ==========================================
CREATE TABLE usuario (
    id_usuario INT IDENTITY PRIMARY KEY,
    tipo_documento VARCHAR(10) NOT NULL,
    documento VARCHAR(30) NOT NULL,
    nombre VARCHAR(150) NOT NULL,
    telefono VARCHAR(20) NOT NULL,
    correo VARCHAR(120) NOT NULL UNIQUE,
    hash_pw VARCHAR(255) NOT NULL,
    activo BIT NOT NULL DEFAULT 1,
    debe_cambiar_pw BIT NOT NULL DEFAULT 0,
    rol VARCHAR(20) NOT NULL,
    tipo_cliente VARCHAR(20) NULL,
    strikes INT NOT NULL DEFAULT 0,
    fecha_ultima_actividad DATETIME NULL,
    fecha_contratacion DATE NULL,
    CONSTRAINT UQ_usuario_documento UNIQUE (tipo_documento, documento),
    CONSTRAINT CK_usuario_rol CHECK (rol IN ('CLIENTE', 'EMPLEADO', 'ADMINISTRADOR')),
    CONSTRAINT CK_usuario_tipo_cliente CHECK (tipo_cliente IS NULL OR tipo_cliente IN ('GENERAL', 'VIP')),
    CONSTRAINT CK_usuario_strikes CHECK (strikes >= 0)
);
GO

CREATE TABLE token_reactivacion (
    id_token INT IDENTITY PRIMARY KEY,
    id_cliente INT NOT NULL,
    token VARCHAR(64) NOT NULL UNIQUE,
    fecha_expiracion DATETIME NOT NULL,
    usado BIT NOT NULL DEFAULT 0
);
GO

-- ==========================================
-- MÓDULO 2: INFRAESTRUCTURA, ZONAS Y MESAS
-- ==========================================
CREATE TABLE zona (
    id_zona INT IDENTITY PRIMARY KEY,
    nombre VARCHAR(40) NOT NULL UNIQUE,
    descripcion VARCHAR(200) NOT NULL,
    CONSTRAINT CK_zona_nombre CHECK (nombre IN ('VIP', 'General', 'Terraza'))
);
GO

CREATE TABLE horario_atencion (
    id_horario INT IDENTITY PRIMARY KEY,
    dia_semana VARCHAR(20) NOT NULL UNIQUE,
    hora_apertura TIME NOT NULL,
    hora_cierre TIME NOT NULL,
    CONSTRAINT CK_horario_horas CHECK (hora_cierre > hora_apertura)
);
GO

CREATE TABLE mesa (
    id_mesa INT IDENTITY PRIMARY KEY,
    numero_mesa INT NOT NULL UNIQUE,
    zona_id INT NOT NULL,
    capacidad INT NOT NULL,
    estado_fisico VARCHAR(20) NOT NULL DEFAULT 'DISPONIBLE',
    disponible_fisicamente BIT NOT NULL DEFAULT 1,
    CONSTRAINT CK_mesa_capacidad CHECK (capacidad BETWEEN 1 AND 20),
    CONSTRAINT CK_mesa_estado_fisico CHECK (estado_fisico IN ('DISPONIBLE', 'FUERA_DE_SERVICIO'))
);
GO

CREATE TABLE bloqueo_mesa (
    id_bloqueo INT IDENTITY PRIMARY KEY,
    mesa_id INT NOT NULL,
    inicio DATETIME NOT NULL,
    fin DATETIME NOT NULL,
    expira DATETIME NOT NULL,
    CONSTRAINT CK_bloqueo_rango CHECK (fin > inicio)
);
GO

-- ==========================================
-- MÓDULO 3: CARTA Y PLANES DE SERVICIO EXTRA
-- ==========================================
CREATE TABLE item_menu (
    id_item INT IDENTITY PRIMARY KEY,
    nombre VARCHAR(120) NOT NULL,
    tipo VARCHAR(20) NOT NULL,
    precio_vigente DECIMAL(12,2) NOT NULL,
    CONSTRAINT CK_item_menu_tipo CHECK (tipo IN ('ENTRADA', 'PLATO_FUERTE', 'POSTRE', 'BEBIDA')),
    CONSTRAINT CK_item_menu_precio CHECK (precio_vigente >= 0)
);
GO

CREATE TABLE plan_servicio_extra (
    id_plan INT IDENTITY PRIMARY KEY,
    nombre VARCHAR(100) NOT NULL,
    tarifa DECIMAL(12,2) NOT NULL,
    descripcion VARCHAR(250) NOT NULL,
    por_persona BIT NOT NULL DEFAULT 0,
    subtipo VARCHAR(20) NOT NULL,
    motivo VARCHAR(30) NULL,
    nivel VARCHAR(20) NULL,
    tipo_barra VARCHAR(30) NULL,
    cobra_por_persona BIT NULL,
    CONSTRAINT CK_plan_tarifa CHECK (tarifa >= 0),
    CONSTRAINT CK_plan_subtipo CHECK (subtipo IN ('DECORACION', 'BARRA_COCTELES')),
    CONSTRAINT CK_plan_motivo CHECK (motivo IS NULL OR motivo IN ('Cumpleaños', 'Aniversario', 'Propuesta', 'Romántica')),
    CONSTRAINT CK_plan_nivel CHECK (nivel IS NULL OR nivel IN ('Básico', 'Medio', 'Elaborado')),
    CONSTRAINT CK_plan_tipo_barra CHECK (tipo_barra IS NULL OR tipo_barra IN ('BASICO', 'CARTA_COMPLETA', 'PREMIUM_SHOW'))
);
GO

-- ==========================================
-- MÓDULO 4: RESERVAS, INTERMEDIAS Y PAGOS
-- ==========================================
CREATE TABLE reserva (
    id_reserva INT IDENTITY PRIMARY KEY,
    codigo VARCHAR(8) NOT NULL UNIQUE,
    id_cliente INT NOT NULL,
    id_creador INT NULL,
    tipo_restaurante_completo BIT NOT NULL DEFAULT 0,
    fecha_hora_inicio DATETIME NOT NULL,
    duracion_minutos INT NOT NULL DEFAULT 120,
    num_comensales INT NOT NULL,
    ocasion VARCHAR(20) NOT NULL,
    restricciones VARCHAR(250) NULL,
    observaciones VARCHAR(250) NULL,
    modalidad_pago VARCHAR(20) NOT NULL,
    porcentaje_pago INT NOT NULL,
    medio_pago VARCHAR(20) NOT NULL,
    monto_total DECIMAL(12,2) NOT NULL DEFAULT 0,
    estado VARCHAR(20) NOT NULL DEFAULT 'VIGENTE',
    CONSTRAINT CK_reserva_codigo_len CHECK (LEN(codigo) = 8),
    CONSTRAINT CK_reserva_duracion CHECK (duracion_minutos > 0),
    CONSTRAINT CK_reserva_comensales CHECK (num_comensales >= 1),
    CONSTRAINT CK_reserva_ocasion CHECK (ocasion IN ('CUMPLEANOS', 'ANIVERSARIO', 'PROPUESTA', 'ROMANTICA', 'NEGOCIOS', 'CASUAL')),
    CONSTRAINT CK_reserva_estado CHECK (estado IN ('VIGENTE', 'CLIENTE_EN_MESA', 'COMPLETADA', 'CANCELADA', 'INASISTIDA')),
    CONSTRAINT CK_reserva_modalidad CHECK (modalidad_pago IN ('ADELANTO', 'COMPLETO', 'EN_RESTAURANTE')),
    CONSTRAINT CK_reserva_porcentaje CHECK (porcentaje_pago BETWEEN 0 AND 100),
    CONSTRAINT CK_reserva_coherencia_pago CHECK (
        (modalidad_pago = 'ADELANTO' AND porcentaje_pago = 30) OR
        (modalidad_pago = 'COMPLETO' AND porcentaje_pago = 100) OR
        (modalidad_pago = 'EN_RESTAURANTE' AND porcentaje_pago = 0)
    ),
    CONSTRAINT CK_reserva_medio_pago CHECK (medio_pago IN ('TARJETA', 'TRANSFERENCIA', 'EFECTIVO')),
    CONSTRAINT CK_reserva_monto CHECK (monto_total >= 0)
);
GO

CREATE TABLE reserva_mesa (
    id_reserva_mesa INT IDENTITY PRIMARY KEY,
    id_reserva INT NOT NULL,
    id_mesa INT NOT NULL,
    precio_congelado DECIMAL(12,2) NOT NULL DEFAULT 0,
    CONSTRAINT UQ_reserva_mesa UNIQUE (id_reserva, id_mesa),
    CONSTRAINT CK_reserva_mesa_precio CHECK (precio_congelado >= 0)
);
GO

CREATE TABLE reserva_detalle_pedido (
    id_detalle_pedido INT IDENTITY PRIMARY KEY,
    id_reserva INT NOT NULL,
    id_item INT NOT NULL,
    cantidad INT NOT NULL,
    precio_unitario DECIMAL(12,2) NOT NULL,
    CONSTRAINT UQ_reserva_detalle_item UNIQUE (id_reserva, id_item),
    CONSTRAINT CK_detalle_pedido_cantidad CHECK (cantidad >= 1),
    CONSTRAINT CK_detalle_pedido_precio CHECK (precio_unitario >= 0)
);
GO

CREATE TABLE reserva_servicio_extra (
    id_reserva_servicio INT IDENTITY PRIMARY KEY,
    id_reserva INT NOT NULL,
    id_plan INT NOT NULL,
    precio DECIMAL(12,2) NOT NULL,
    CONSTRAINT UQ_reserva_servicio_plan UNIQUE (id_reserva, id_plan),
    CONSTRAINT CK_servicio_extra_precio CHECK (precio >= 0)
);
GO

CREATE TABLE comprobante_pago (
    id_comprobante INT IDENTITY PRIMARY KEY,
    id_reserva INT NOT NULL,
    id_empleado INT NULL,
    fecha_emision DATETIME NOT NULL DEFAULT GETDATE(),
    concepto VARCHAR(30) NOT NULL,
    tipo_movimiento VARCHAR(15) NOT NULL,
    medio_pago VARCHAR(20) NOT NULL,
    monto_subtotal DECIMAL(12,2) NOT NULL,
    monto_retencion DECIMAL(12,2) NOT NULL DEFAULT 0,
    monto_total DECIMAL(12,2) NOT NULL,
    CONSTRAINT CK_comprobante_concepto CHECK (concepto IN ('ABONO', 'LIQUIDACION', 'REEMBOLSO_TOTAL', 'REEMBOLSO_PARCIAL')),
    CONSTRAINT CK_comprobante_movimiento CHECK (tipo_movimiento IN ('INGRESO', 'EGRESO')),
    CONSTRAINT CK_comprobante_medio CHECK (medio_pago IN ('TARJETA', 'TRANSFERENCIA', 'EFECTIVO')),
    CONSTRAINT CK_comprobante_montos CHECK (monto_subtotal >= 0 AND monto_retencion >= 0 AND monto_total >= 0)
);
GO

-- ==========================================
-- RELACIONES (FOREIGN KEYS CON ON DELETE EXPLÍCITO)
-- ==========================================
ALTER TABLE token_reactivacion ADD CONSTRAINT FK_token_reactivacion_id_cliente 
    FOREIGN KEY (id_cliente) REFERENCES usuario(id_usuario) ON DELETE CASCADE;
GO

ALTER TABLE mesa ADD CONSTRAINT FK_mesa_zona_id 
    FOREIGN KEY (zona_id) REFERENCES zona(id_zona) ON DELETE NO ACTION;
GO

ALTER TABLE bloqueo_mesa ADD CONSTRAINT FK_bloqueo_mesa_mesa_id 
    FOREIGN KEY (mesa_id) REFERENCES mesa(id_mesa) ON DELETE CASCADE;
GO

ALTER TABLE reserva ADD CONSTRAINT FK_reserva_id_cliente 
    FOREIGN KEY (id_cliente) REFERENCES usuario(id_usuario) ON DELETE NO ACTION;
GO

ALTER TABLE reserva ADD CONSTRAINT FK_reserva_id_creador 
    FOREIGN KEY (id_creador) REFERENCES usuario(id_usuario) ON DELETE NO ACTION;
GO

ALTER TABLE reserva_mesa ADD CONSTRAINT FK_reserva_mesa_id_reserva 
    FOREIGN KEY (id_reserva) REFERENCES reserva(id_reserva) ON DELETE CASCADE;
GO

ALTER TABLE reserva_mesa ADD CONSTRAINT FK_reserva_mesa_id_mesa 
    FOREIGN KEY (id_mesa) REFERENCES mesa(id_mesa) ON DELETE NO ACTION;
GO

ALTER TABLE reserva_detalle_pedido ADD CONSTRAINT FK_reserva_detalle_pedido_id_reserva 
    FOREIGN KEY (id_reserva) REFERENCES reserva(id_reserva) ON DELETE CASCADE;
GO

ALTER TABLE reserva_detalle_pedido ADD CONSTRAINT FK_reserva_detalle_pedido_id_item 
    FOREIGN KEY (id_item) REFERENCES item_menu(id_item) ON DELETE NO ACTION;
GO

ALTER TABLE reserva_servicio_extra ADD CONSTRAINT FK_reserva_servicio_extra_id_reserva 
    FOREIGN KEY (id_reserva) REFERENCES reserva(id_reserva) ON DELETE CASCADE;
GO

ALTER TABLE reserva_servicio_extra ADD CONSTRAINT FK_reserva_servicio_extra_id_plan 
    FOREIGN KEY (id_plan) REFERENCES plan_servicio_extra(id_plan) ON DELETE NO ACTION;
GO

ALTER TABLE comprobante_pago ADD CONSTRAINT FK_comprobante_pago_id_reserva 
    FOREIGN KEY (id_reserva) REFERENCES reserva(id_reserva) ON DELETE CASCADE;
GO

ALTER TABLE comprobante_pago ADD CONSTRAINT FK_comprobante_pago_id_empleado 
    FOREIGN KEY (id_empleado) REFERENCES usuario(id_usuario) ON DELETE NO ACTION;
GO

-- ==========================================
-- ÍNDICES DE RENDIMIENTO Y REGLAS DE NEGOCIO
-- ==========================================
CREATE INDEX IX_reserva_mesa_id_mesa ON reserva_mesa(id_mesa);
GO

CREATE INDEX IX_reserva_fecha_hora_inicio ON reserva(fecha_hora_inicio);
GO

CREATE INDEX IX_bloqueo_mesa_rango ON bloqueo_mesa(mesa_id, inicio, fin);
GO

-- ==========================================
-- DATOS SEMILLA (SEED MÍNIMO COHERENTE)
-- ==========================================

-- 1. Usuarios: 1 Admin, 1 Empleado, 2 Clientes (1 General, 1 VIP con 2 strikes para prueba RN10)
INSERT INTO usuario (tipo_documento, documento, nombre, telefono, correo, hash_pw, activo, debe_cambiar_pw, rol, tipo_cliente, strikes, fecha_ultima_actividad, fecha_contratacion)
VALUES 
('CC', '10001001', 'Carlos Mendoza', '3001112233', 'admin@brindis.co', 'hash_admin_123', 1, 0, 'ADMINISTRADOR', NULL, 0, NULL, '2024-01-15'),
('CC', '10002002', 'Laura Gómez', '3002223344', 'empleado@brindis.co', 'hash_emp_123', 1, 0, 'EMPLEADO', NULL, 0, NULL, '2025-03-01'),
('CC', '10003003', 'Andrés Pérez', '3103334455', 'andres.general@correo.co', 'hash_cli_123', 1, 0, 'CLIENTE', 'GENERAL', 0, GETDATE(), NULL),
('CC', '10004004', 'Valeria Soto', '3154445566', 'valeria.vip@correo.co', 'hash_vip_123', 1, 0, 'CLIENTE', 'VIP', 2, GETDATE(), NULL);
GO

-- 2. Zonas: VIP, General, Terraza
INSERT INTO zona (nombre, descripcion)
VALUES 
('VIP', 'Salón privado climatizado con iluminación tenue y atención prioritaria'),
('General', 'Salón principal con ambiente cálido ideal para celebraciones y grupos'),
('Terraza', 'Espacio al aire libre con vista panorámica y ambiente romántico');
GO

-- 3. Horario de Atención (1 horario de ejemplo)
INSERT INTO horario_atencion (dia_semana, hora_apertura, hora_cierre)
VALUES ('VIERNES', '12:00:00', '23:00:00');
GO

-- 4. Mesas (6 mesas distribuidas en las 3 zonas)
INSERT INTO mesa (numero_mesa, zona_id, capacidad, estado_fisico, disponible_fisicamente)
VALUES 
(1, 1, 4, 'DISPONIBLE', 1),
(2, 1, 6, 'DISPONIBLE', 1),
(3, 2, 4, 'DISPONIBLE', 1),
(4, 2, 8, 'DISPONIBLE', 1),
(5, 3, 2, 'DISPONIBLE', 1),
(6, 3, 4, 'DISPONIBLE', 1);
GO

-- 5. Planes de Servicio Extra (1 decoración + 1 barra)
INSERT INTO plan_servicio_extra (nombre, tarifa, descripcion, por_persona, subtipo, motivo, nivel, tipo_barra, cobra_por_persona)
VALUES 
('Decoración Aniversario Especial', 120000.00, 'Arreglo floral en mesa, velas aromáticas y pétalos', 0, 'DECORACION', 'Aniversario', 'Elaborado', NULL, NULL),
('Barra Libre Coctelería Autor', 85000.00, 'Selección ilimitada de cocteles de la casa durante la reserva', 1, 'BARRA_COCTELES', NULL, NULL, 'CARTA_COMPLETA', 1);
GO

-- 6. Ítems del Menú (3 ítems de la carta)
INSERT INTO item_menu (nombre, tipo, precio_vigente)
VALUES 
('Carpaccio de Res al Parmesano', 'ENTRADA', 42000.00),
('Lomo al Trapo en Reducción de Vino', 'PLATO_FUERTE', 78000.00),
('Volcán de Chocolate con Helado Artesanal', 'POSTRE', 28000.00);
GO

-- 7. Reserva vigente de ejemplo con su mesa, pedido, servicio extra, pago y comprobante
-- Subtotal menú: (2 * 42000) + (2 * 78000) + (2 * 28000) = 296000
-- Subtotal extra: 120000 (decoración) = 120000 -> Total Reserva = 416000. Adelanto 30% = 124800
INSERT INTO reserva (codigo, id_cliente, id_creador, tipo_restaurante_completo, fecha_hora_inicio, duracion_minutos, num_comensales, ocasion, restricciones, observaciones, modalidad_pago, porcentaje_pago, medio_pago, monto_total, estado)
VALUES ('BRND2026', 4, 2, 0, '2026-10-23 19:00:00', 120, 4, 'ANIVERSARIO', 'Sin frutos secos', 'Mesa apartada, luz tenue', 'ADELANTO', 30, 'TARJETA', 416000.00, 'VIGENTE');
GO

INSERT INTO reserva_mesa (id_reserva, id_mesa, precio_congelado)
VALUES (1, 1, 0.00);
GO

INSERT INTO reserva_detalle_pedido (id_reserva, id_item, cantidad, precio_unitario)
VALUES 
(1, 1, 2, 42000.00),
(1, 2, 2, 78000.00),
(1, 3, 2, 28000.00);
GO

INSERT INTO reserva_servicio_extra (id_reserva, id_plan, precio)
VALUES (1, 1, 120000.00);
GO

INSERT INTO comprobante_pago (id_reserva, id_empleado, fecha_emision, concepto, tipo_movimiento, medio_pago, monto_subtotal, monto_retencion, monto_total)
VALUES (1, 2, GETDATE(), 'ABONO', 'INGRESO', 'TARJETA', 124800.00, 0.00, 124800.00);
GO

-- Bloqueo temporal de ejemplo para probar RN03
INSERT INTO bloqueo_mesa (mesa_id, inicio, fin, expira)
VALUES (2, '2026-10-23 19:00:00', '2026-10-23 21:00:00', DATEADD(MINUTE, 5, GETDATE()));
GO

-- ==========================================
-- CONSULTAS DE VERIFICACIÓN (COMENTADAS)
-- ==========================================

/*
-- 1. Verificación RN01: Detectar cruce de horario en la misma mesa para reservas activas
-- Evalúa si una nueva franja (@nuevo_inicio a @nuevo_fin) se solapa con reservas VIGENTE o CLIENTE_EN_MESA.
DECLARE @id_mesa_consulta INT = 1;
DECLARE @nuevo_inicio DATETIME = '2026-10-23 20:00:00';
DECLARE @nuevo_fin DATETIME = DATEADD(MINUTE, 120, @nuevo_inicio);

SELECT 
    r.id_reserva,
    r.codigo,
    rm.id_mesa,
    r.fecha_hora_inicio AS inicio_existente,
    DATEADD(MINUTE, r.duracion_minutos, r.fecha_hora_inicio) AS fin_existente,
    r.estado
FROM reserva r
INNER JOIN reserva_mesa rm ON r.id_reserva = rm.id_reserva
WHERE rm.id_mesa = @id_mesa_consulta
  AND r.estado IN ('VIGENTE', 'CLIENTE_EN_MESA')
  AND r.fecha_hora_inicio < @nuevo_fin
  AND @nuevo_inicio < DATEADD(MINUTE, r.duracion_minutos, r.fecha_hora_inicio);

-- 2. Verificación RN02: Comprobar que suma(capacidades de mesas asignadas) >= num_comensales
SELECT 
    r.id_reserva,
    r.codigo,
    r.num_comensales,
    SUM(m.capacidad) AS capacidad_total_asignada,
    CASE 
        WHEN SUM(m.capacidad) >= r.num_comensales THEN 'CUMPLE RN02'
        ELSE 'INCUMPLE RN02'
    END AS validacion_capacidad
FROM reserva r
INNER JOIN reserva_mesa rm ON r.id_reserva = rm.id_reserva
INNER JOIN mesa m ON rm.id_mesa = m.id_mesa
GROUP BY r.id_reserva, r.codigo, r.num_comensales;

-- 3. Verificación RN03: Consultar bloqueos temporales vigentes (no expirados) que se solapan con una franja
DECLARE @ahora DATETIME = GETDATE();
DECLARE @franja_ini DATETIME = '2026-10-23 19:30:00';
DECLARE @franja_fin DATETIME = '2026-10-23 21:30:00';

SELECT 
    b.id_bloqueo,
    b.mesa_id,
    m.numero_mesa,
    b.inicio,
    b.fin,
    b.expira,
    DATEDIFF(SECOND, @ahora, b.expira) AS segundos_restantes
FROM bloqueo_mesa b
INNER JOIN mesa m ON b.mesa_id = m.id_mesa
WHERE b.expira > @ahora
  AND b.inicio < @franja_fin
  AND @franja_ini < b.fin;

-- 4. Verificación RN08 y RN09: Cálculo de reembolso y retención (10%) según tipo de cliente y horas de anticipación
-- General >= 4h (240 min) -> 100%, sino retiene 10%. VIP >= 1h (60 min) -> 100%, sino retiene 10% + 1 strike.
DECLARE @fecha_cancelacion DATETIME = '2026-10-23 17:30:00'; -- 1.5 horas antes de las 19:00

SELECT 
    r.codigo,
    u.nombre AS cliente,
    u.tipo_cliente,
    r.fecha_hora_inicio,
    DATEDIFF(MINUTE, @fecha_cancelacion, r.fecha_hora_inicio) AS minutos_anticipacion,
    cp.monto_total AS monto_abonado,
    CASE 
        WHEN u.tipo_cliente = 'GENERAL' AND DATEDIFF(MINUTE, @fecha_cancelacion, r.fecha_hora_inicio) >= 240 THEN 0.00
        WHEN u.tipo_cliente = 'VIP' AND DATEDIFF(MINUTE, @fecha_cancelacion, r.fecha_hora_inicio) >= 60 THEN 0.00
        ELSE ROUND(cp.monto_total * 0.10, 2)
    END AS retencion_10_pct,
    CASE 
        WHEN u.tipo_cliente = 'GENERAL' AND DATEDIFF(MINUTE, @fecha_cancelacion, r.fecha_hora_inicio) >= 240 THEN cp.monto_total
        WHEN u.tipo_cliente = 'VIP' AND DATEDIFF(MINUTE, @fecha_cancelacion, r.fecha_hora_inicio) >= 60 THEN cp.monto_total
        ELSE ROUND(cp.monto_total * 0.90, 2)
    END AS monto_a_reembolsar,
    CASE 
        WHEN u.tipo_cliente = 'VIP' AND DATEDIFF(MINUTE, @fecha_cancelacion, r.fecha_hora_inicio) < 60 THEN 'APLICA STRIKE (RN09)'
        ELSE 'SIN STRIKE'
    END AS penalidad_strike
FROM reserva r
INNER JOIN usuario u ON r.id_cliente = u.id_usuario
INNER JOIN comprobante_pago cp ON r.id_reserva = cp.id_reserva AND cp.tipo_movimiento = 'INGRESO'
WHERE r.id_reserva = 1;

-- 5. Verificación RN10: Simulación/consulta de degradación de cliente VIP a GENERAL al alcanzar el 3er strike
SELECT 
    id_usuario,
    nombre,
    correo,
    tipo_cliente AS tipo_actual,
    strikes AS strikes_actuales,
    strikes + 1 AS strikes_tras_incumplimiento,
    CASE 
        WHEN (strikes + 1) >= 3 THEN 'GENERAL'
        ELSE tipo_cliente
    END AS nuevo_tipo_cliente_rn10
FROM usuario
WHERE rol = 'CLIENTE' AND tipo_cliente = 'VIP';
*/