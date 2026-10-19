"""Diagramas Brindis (neutros) sin solapes y CON multiplicidad.
Reusa uml.py. Ejecucion: python fig_brindis.py
Valida 0 incidencias (sin cruces ni etiquetas pisadas).
Convenciones: dep = dependencia navegable; numero cerca de cada extremo
es la multiplicidad (1, *, 0..*). Herencia/realizacion no llevan.
"""
import os
import sys

AQUI = os.path.dirname(os.path.abspath(__file__))
SRC = AQUI
sys.path.insert(0, SRC)

from uml import Canvas  # noqa: E402


def fig7_factory(out):
    c = Canvas(9.6, 5.4, fs=8, name="fig7-brindis")
    svc = c.cls("svc", 0.12, 0.12, "UsuarioService",
                attrs=["-fabricas : Map<Rol, UsuarioFactory>"],
                meths=["+fabrica_para(rol : Rol) : UsuarioFactory"])
    fac = c.cls("fac", svc.r + 1.05, 0.12, "UsuarioFactory",
                meths=["+rol() : Rol", "+crear_usuario(base, datos) : Usuario"],
                kind="abstract")
    y1 = 1.75
    cf = c.cls("cf", 0.35, y1, "ClienteFactory",
               meths=["+crear_usuario() : Cliente"])
    ef = c.cls("ef", cf.r + 0.55, y1, "EmpleadoFactory",
               meths=["+crear_usuario() : Empleado"])
    af = c.cls("af", ef.r + 0.55, y1, "AdministradorFactory",
               meths=["+crear_usuario() : Administrador"])
    y2 = 3.15
    cli = c.cls("cli", cf.x, y2, "Cliente",
                attrs=["-tipo : TipoCliente", "-strikes : int"])
    emp = c.cls("emp", ef.x, y2, "Empleado",
                attrs=["-legajo : String"])
    adm = c.cls("adm", af.x, y2, "Administrador",
                attrs=["-empleado : Empleado"])
    # dependencia: sin multiplicidad (solo etiqueta)
    c.edge("dep", [c.pt("svc", "E", at=svc.cy), c.pt("fac", "W", at=svc.cy)],
           "svc", "fac", label=("elige", 0, 0.5, 0, -0.13))
    c.tree_v("gen", "fac", fac.cx, y1 - 0.30, ["cf", "ef", "af"])
    # dependencia <<create>>: sin multiplicidad
    c.edge("dep", [c.pt("cf", "S"), c.pt("cli", "N", at=cf.cx)], "cf", "cli",
           label=("crea", 0, 0.5, 0.14, 0, "left"))
    c.edge("dep", [c.pt("ef", "S"), c.pt("emp", "N", at=ef.cx)], "ef", "emp",
           label=("crea", 0, 0.5, 0.14, 0, "left"))
    c.edge("dep", [c.pt("af", "S"), c.pt("adm", "N", at=af.cx)], "af", "adm",
           label=("crea", 0, 0.5, 0.14, 0, "left"))
    c.edge("gen", [c.pt("adm", "W"), c.pt("emp", "E", at=adm.cy)], "adm", "emp")
    issues = c.validate()
    assert not issues, issues
    c.fit()
    c.render(out)


def fig8_builder(out):
    c = Canvas(9.6, 3.4, fs=8, name="fig8-brindis")
    y = 0.14
    d = c.cls("dir", 0.12, y, "ReservaServiceImpl",
              attrs=["[director]"], meths=["+confirmar() : id"])
    b = c.cls("bld", d.r + 1.00, y, "ReservaBuilder",
              meths=["+cliente() : Builder", "+mesas() : Builder", "+fecha() : Builder",
                     "+pago() : Builder", "+build() : Reserva"])
    r = c.cls("res", b.r + 1.00, y, "Reserva",
              attrs=["-codigo : String", "-estado : EstadoReserva"],
              meths=["+new() : Reserva {paquete}"])
    yy = y + 0.40
    # dependencias: sin multiplicidad
    c.edge("dep", [c.pt("dir", "E", at=yy), c.pt("bld", "W", at=yy)], "dir", "bld",
           label=("dirige", 0, 0.5, 0, -0.13))
    c.edge("dep", [c.pt("bld", "E", at=yy), c.pt("res", "W", at=yy)], "bld", "res",
           label=("construye", 0, 0.5, 0, -0.13))
    n = c.note("nota", b.x, b.b + 0.45,
               ["build() concentra RN02 capacidad,",
                "RN05 horario, RN06 pago. Si falla,",
                "la Reserva jamas existe."])
    c.edge("note", [c.pt("nota", "N", at=b.cx), c.pt("bld", "S", at=b.cx)], "nota", "bld")
    issues = c.validate()
    assert not issues, issues
    c.fit()
    c.render(out)


def fig9_proxy(out):
    c = Canvas(9.6, 4.1, fs=8, name="fig9-brindis")
    ctl = c.cls("ctl", 0.12, 0.12, "ReservaController",
                meths=["+POST /reservas : 201"])
    iface = c.cls("if", ctl.r + 0.70, 0.12, "ReservaService",
                  meths=["+confirmar() : id", "+cancelar() : void",
                         "+agenda() : List<Reserva>"],
                  kind="interface")
    y1 = 1.85
    prx = c.cls("prx", 0.12, y1, "ReservaServiceProxy",
                attrs=["-real : ReservaService"],
                meths=["+verificar_rol() : void"])
    real = c.cls("real", prx.r + 1.10, y1, "ReservaServiceImpl",
                 meths=["+confirmar() : id  [transaccion]"])
    ses = c.cls("ses", real.r + 0.70, y1, "UsuarioSesion",
                attrs=["-rol : Rol", "-id_cliente : id"])
    yy = 0.12 + 0.40
    # dependencia: sin multiplicidad
    c.edge("dep", [c.pt("ctl", "E", at=yy), c.pt("if", "W", at=yy)], "ctl", "if",
           label=("usa", 0, 0.5, 0, -0.10))
    c.tree_v("real", "if", iface.cx, y1 - 0.30, ["prx", "real"])
    _bus = 3.05
    _px = c.boxes["prx"].cx
    _sx = c.boxes["ses"].cx
    # dependencia: sin multiplicidad (rodeo por DEBAJO para no cruzar)
    _bus = 3.05
    _px = c.boxes["prx"].cx
    _sx = c.boxes["ses"].cx
    c.edge("dep", [c.pt("prx", "S", at=_px), (_px, _bus),
                   (_sx, _bus), c.pt("ses", "S", at=_sx)], "prx", "ses",
           label=("verifica rol", 1, 0.5, 0, -0.11))
    # dependencia: sin multiplicidad
    _dy = prx.cy
    c.edge("dep", [c.pt("prx", "E", at=_dy), c.pt("real", "W", at=_dy)],
           "prx", "real", label=("delega", 0, 0.5, 0, -0.11))
    issues = c.validate()
    assert not issues, issues
    c.fit()
    c.render(out)


def fig10_decorator(out):
    c = Canvas(9.6, 5.5, fs=8, name="fig10-brindis")
    comp = c.cls("comp", 3.60, 0.12, "ComponenteReserva",
                 meths=["+costo() : double", "+descripcion() : String",
                        "+indicaciones() : Lista"],
                 kind="interface")
    y1 = 1.70
    base = c.cls("base", 0.30, y1, "ReservaBase",
                 attrs=["-pedido_total : double"],
                 meths=["+costo() : double"])
    dec = c.cls("dec", base.r + 1.10, y1, "ReservaDecorator",
                attrs=["-envuelto : ComponenteReserva"],
                kind="abstract")
    y2 = 3.45
    d1 = c.cls("d1", dec.x, y2, "DecoracionDecorator",
               attrs=["-tarifa : double"],
               meths=["+costo() : envuelto + tarifa"])
    d2 = c.cls("d2", d1.r + 0.60, y2, "BarraCoctelesDecorator",
               attrs=["-tarifa : double", "-por_persona : boolean"],
               meths=["+costo() : envuelto + tarifa x n"])
    c.tree_v("real", "comp", comp.cx, y1 - 0.30, ["base", "dec"])
    # asociacion (agregacion): SI lleva multiplicidad 1 a 1
    c.edge("agg", [c.pt("dec", "W", at=dec.cy), c.pt("base", "E", at=base.cy)],
           "dec", "base", label=("envuelve", 0, 0.5, 0, -0.11),
           mt=("1", 1), mh=("1", 1))
    c.tree_v("gen", "dec", dec.cx, y2 - 0.30, ["d1", "d2"])
    issues = c.validate()
    assert not issues, issues
    c.fit()
    c.render(out)


def fig6_dao(out):
    c = Canvas(9.8, 6.0, fs=8, name="fig6-brindis")
    gen = c.cls("gen", 3.60, 0.12, "GenericDAO<T, Id>",
                meths=["+obtener() +listar() +guardar() +eliminar()"],
                kind="interface")
    y1 = 1.55
    u = c.cls("u", 0.12, y1, "UsuarioDAO", kind="interface",
              meths=["+buscar_por_correo() : Usuario"])
    m = c.cls("m", u.r + 0.40, y1, "MesaDAO", kind="interface",
              meths=["+en_servicio() : Lista"])
    r = c.cls("r", m.r + 0.40, y1, "ReservaDAO", kind="interface",
              meths=["+hay_cruce() : boolean"])
    p = c.cls("p", r.r + 0.40, y1, "PagoDAO", kind="interface",
              meths=["+emitir_comprobante() : id"])
    y2 = 3.00
    ui = c.cls("ui", u.x, y2, "UsuarioDAOImpl", attrs=["-pool : ConnectionPool"])
    mi = c.cls("mi", m.x, y2, "MesaDAOImpl", attrs=["-pool : ConnectionPool"])
    ri = c.cls("ri", r.x, y2, "ReservaDAOImpl", attrs=["-pool : ConnectionPool"])
    pi = c.cls("pi", p.x, y2, "PagoDAOImpl", attrs=["-pool : ConnectionPool"])
    pool = c.cls("pool", 0.12, 4.60, "ConexionBD",
                 attrs=["-url : String"],
                 kind="utility")
    c.tree_v("gen", "gen", gen.cx, y1 - 0.30, ["u", "m", "r", "p"])
    for a, b in [("ui", "u"), ("mi", "m"), ("ri", "r"), ("pi", "p")]:
        A = c.boxes[a]
        c.edge("real", [c.pt(a, "N", at=A.cx), c.pt(b, "S", at=A.cx)], a, b)
    # 1..* impls usan 1 pool: bus limpio (una horizontal + verticales) y una sola flecha con multiplicidad
    bus = 4.35
    _pool_cx = c.boxes["pool"].cx
    _xs = [c.boxes[k].cx for k in ["ui", "mi", "ri", "pi"]]
    _lo, _hi = min(_xs), max(_xs)
    c._plain([(_lo, bus), (_hi, bus)], dashed=True)
    for k in ["ui", "mi", "ri", "pi"]:
        _cx = c.boxes[k].cx
        _b = c.boxes[k].b
        c._plain([(_cx, _b + 0.05), (_cx, bus)], dashed=True, src=k)
    _midbus = (_lo + _hi) / 2
    c.edge("dep", [(_midbus, bus), (_pool_cx, bus), c.pt("pool", "N", at=_pool_cx)],
           "ui", "pool")
    issues = c.validate()
    assert not issues, issues
    c.fit()
    c.render(out)


def fig11_control(out):
    c = Canvas(9.8, 6.5, fs=8, name="fig11-brindis")
    a1 = c.cls("a1", 0.12, 0.12, "AuthController",
               meths=["+login() : Sesion"])
    a2 = c.cls("a2", a1.r + 0.45, 0.12, "ReservaController",
               meths=["+confirmar() : id"])
    a3 = c.cls("a3", a2.r + 0.45, 0.12, "PersonalController",
               meths=["+agenda() : Lista"])
    y1 = 1.35
    iu = c.cls("iu", 0.40, y1, "IUsuarioService",
               meths=["+registrar() : id"], kind="interface")
    ir = c.cls("ir", iu.r + 0.60, y1, "IReservaService",
               meths=["+confirmar() : id", "+cancelar() : void"], kind="interface")
    y2 = 2.70
    pu = c.cls("pu", iu.x, y2, "UsuarioServiceProxy",
               meths=["+registrar() : id"])
    pr = c.cls("pr", ir.x, y2, "ReservaServiceProxy",
               meths=["+confirmar() : id"])
    im = c.cls("im", pr.r + 1.30, y2, "ReservaServiceImpl",
               attrs=["[director]"], meths=["+confirmar() : id"])
    y3 = 4.05
    fb = c.cls("fb", 0.12, y3, "FabricaUsuarios + Builder",
               meths=["+crear() : Usuario", "+build() : Reserva"])
    dc = c.cls("dc", fb.r + 0.50, y3, "Costos + Decoradores",
               meths=["+costo() : double"])
    dao = c.cls("dao", dc.r + 0.50, y3, "DAO + ConexionBD",
                meths=["+guardar() : id"])
    # controladores -> interfaces con codo al centro (dependencias: sin multiplicidad)
    for a, b in [("a1", "iu"), ("a2", "ir")]:
        A, B = c.boxes[a], c.boxes[b]
        _mid = (A.b + B.y) / 2
        c.edge("dep", [c.pt(a, "S", at=A.cx), (A.cx, _mid),
                       (B.cx, _mid), c.pt(b, "N", at=B.cx)], a, b)
    A, B = c.boxes["a3"], c.boxes["ir"]
    _mid = (A.b + B.y) / 2
    c.edge("dep", [c.pt("a3", "S", at=A.cx), (A.cx, _mid),
                   (B.cx, _mid), c.pt("ir", "N", at=B.cx)], "a3", "ir")
    c.edge("real", [c.pt("pu", "N"), c.pt("iu", "S", at=pu.cx)], "pu", "iu")
    c.edge("real", [c.pt("pr", "N"), c.pt("ir", "S", at=pr.cx)], "pr", "ir")
    c.edge("dep", [c.pt("pr", "E"), c.pt("im", "W")], "pr", "im",
           label=("delega", 0, 0.5, 0, -0.10))
    c.edge("dep", [c.pt("pu", "S"), c.pt("fb", "N", at=pu.cx)], "pu", "fb")
    _im_cx = c.boxes["im"].cx
    _dc_cx = c.boxes["dc"].cx
    _dao_cx = c.boxes["dao"].cx
    _mid = 3.82
    c.edge("dep", [c.pt("im", "S", at=_im_cx), (_im_cx, _mid),
                   (_dc_cx, _mid), c.pt("dc", "N", at=_dc_cx)], "im", "dc")
    c.edge("dep", [c.pt("im", "S", at=_dao_cx), c.pt("dao", "N", at=_dao_cx)], "im", "dao")
    issues = c.validate()
    assert not issues, issues
    c.fit()
    c.render(out)


JOBS = {
    "fig7": fig7_factory,
    "fig8": fig8_builder,
    "fig9": fig9_proxy,
    "fig10": fig10_decorator,
    "fig6": fig6_dao,
    "fig11": fig11_control,
}

if __name__ == "__main__":
    only = sys.argv[1:] or sorted(JOBS)
    os.makedirs(os.path.join(AQUI, "png"), exist_ok=True)
    os.makedirs(os.path.join(AQUI, "svg"), exist_ok=True)
    for n in only:
        JOBS[n](os.path.join(AQUI, "png", f"{n}-brindis.png"))
        JOBS[n](os.path.join(AQUI, "svg", f"{n}-brindis.svg"))
        print(f"[OK] {n} sin solapes")
