from uml import Canvas, ortho

GREY = "#EDEDED"


def fig7(out):
    c = Canvas(6.4, 3.4, fs=8, name="fig7")
    y = 0.14
    d = c.cls("dir", 0.1, y, "Director", attrs=["-constructor : Constructor"], meths=["+ensamblar() : void"])
    k = c.cls("con", d.r + 0.60, y, "Constructor", kind="interface",
              meths=["+pasoA() : void", "+pasoB() : void", "+pasoC() : void", "+construir() : Producto"])
    cc = c.cls("cc", k.r + 0.75, y, "ConstructorConcreto", attrs=["-producto : Producto"],
               meths=["+pasoA() : void", "+pasoB() : void", "+pasoC() : void", "+construir() : Producto"])
    pr = c.cls("pr", 0, cc.b + 0.55, "Producto", attrs=["-parteA : string", "-parteB : string", "-parteC : string"])
    pr.x = cc.cx - pr.w / 2
    yy = y + 0.36
    c.edge("dep", [c.pt("dir", "E", at=yy), c.pt("con", "W", at=yy)], "dir", "con",
           label=("dirige", 0, 0.5, 0, -0.10))
    c.edge("real", [c.pt("cc", "W", at=yy), c.pt("con", "E", at=yy)], "cc", "con")
    c.edge("dep", [c.pt("cc", "S"), c.pt("pr", "N")], "cc", "pr", label=("construye", 0, 0.5, 0.12, 0, "left"))
    c.validate()
    c.fit()
    c.render(out)


def parts(c, tiq, left, right, y_first, spacing, kinds, mults=None, mtt=None):
    """Conecta las partes de Tiquete/Envio: izquierda y derecha, con un desplazamiento en la calle."""
    mults = mults or {}
    mtt = mtt or {}
    n = len(left)
    ys = [y_first + i * spacing for i in range(n)]
    off = [-0.24, 0.0, 0.24]
    for side, group in (("W", left), ("E", right)):
        for i, key in enumerate(group):
            b = c.boxes[key]
            b.y = ys[i] - b.h / 2
            ay = tiq.cy + off[i]
            kind = kinds[key][0]
            role = kinds[key][1]
            if side == "W":
                p0, p1 = (tiq.x, ay), (b.r, ys[i])
                xm = (b.r + tiq.x) / 2
                s0, s1 = "W", "E"
            else:
                p0, p1 = (tiq.r, ay), (b.x, ys[i])
                xm = (b.x + tiq.r) / 2
                s0, s1 = "E", "W"
            pts = ortho(p0, s0, p1, s1, mid=xm)
            seg = len(pts) - 2
            lab = (role, seg, 0.5 if len(pts) > 2 else 0.6, 0, 0.11)
            mh = None
            if key in mults:
                mh = (mults[key], 1 if side == "W" else -1)
            mt = (mtt.get(key, "1"), 1 if side == "W" else -1)
            c.edge(kind, pts, "tiq", key, label=lab, mt=mt, mh=mh)


def fig8(out):
    c = Canvas(6.4, 6.0, fs=8, name="fig8")
    y = 0.12
    taq = c.cls("taq", 0.1, y, "Taquillero", meths=["+venderTiquete() : Tiquete"])
    cti = c.cls("cti", taq.r + 0.60, y, "ConstructorTiquete", kind="interface",
                meths=["+fijarViaje(v) : void", "+bloquearSilla(s) : void", "+fijarPasajero(p) : void",
                       "+fijarTarifa(t) : void", "+fijarPago(pago) : void", "+emitirFactura() : void",
                       "+construir() : Tiquete"])
    cim = c.cls("cim", cti.r + 0.55, y, "ConstructorTiqueteImpl", attrs=["-tiquete : Tiquete"])
    yy = y + 0.36
    c.edge("dep", [c.pt("taq", "E", at=yy), c.pt("cti", "W", at=yy)], "taq", "cti", label=("dirige", 0, 0.5, 0, -0.10))
    c.edge("real", [c.pt("cim", "W", at=yy), c.pt("cti", "E", at=yy)], "cim", "cti")
    W = 6.4
    pw = 1.25
    tiq = c.cls("tiq", 0, 2.35, "Tiquete", attrs=["-numero : string", "-fechaVenta : string", "-estado : string",
                                                   "-valorFinal : double"], w=1.55)
    tiq.x = W / 2 - tiq.w / 2
    L = [c.cls("via", 0.1, 0, "Viaje", attrs=["-origen : string", "-destino : string"], w=pw),
         c.cls("sil", 0.1, 0, "Silla", attrs=["-numero : int", "-estado : string"], w=pw),
         c.cls("pas", 0.1, 0, "Pasajero", attrs=["-documento : string", "-nombre : string"], w=pw)]
    R = [c.cls("pag", W - 0.1 - pw, 0, "Pago", attrs=["-metodo : string", "-valor : double"], w=pw),
         c.cls("fac", W - 0.1 - pw, 0, "Factura", attrs=["-cufe : string", "-total : double"], w=pw),
         c.cls("tar", W - 0.1 - pw, 0, "Tarifa", kind="interface", w=pw, fill=GREY)]
    ymid = cti.b + 0.75 + tiq.h / 2
    tiq.y = ymid - tiq.h / 2
    # Impl -> Tiquete
    ymid_route = cti.b + 0.28
    c.edge("dep", ortho(c.pt("cim", "S"), "S", c.pt("tiq", "N"), "N", mid=ymid_route), "cim", "tiq",
           label=("obtiene", 1, 0.5, 0, -0.10))
    kinds = {"via": ("agg", "viaje"), "sil": ("agg", "silla"), "pas": ("agg", "pasajero"),
             "pag": ("agg", "pago"), "fac": ("agg", "factura"), "tar": ("agg", "tarifa")}
    parts(c, tiq, [b.key for b in L], [b.key for b in R], ymid - 0.72, 0.72, kinds,
          mults={"via": "1", "sil": "1", "pas": "1", "pag": "1", "fac": "1", "tar": "1"},
          mtt={"via": "0..*", "sil": "0..*", "pas": "0..*", "pag": "1", "fac": "1", "tar": "0..*"})
    fabt = c.cls("ftar", 0, 0, "FabricaTarifa", kind="abstract", fill=GREY)
    tarb = c.boxes["tar"]
    fabt.x = tarb.cx - fabt.w / 2
    fabt.y = tarb.b + 0.55
    c.edge("dep", [c.pt("ftar", "N"), c.pt("tar", "S")], "ftar", "tar", label=("crea", 0, 0.5, 0.12, 0, "left"))
    c.validate()
    c.fit()
    c.render(out)


def fig9(out):
    c = Canvas(6.4, 6.2, fs=8, name="fig9")
    y = 0.12
    aux = c.cls("aux", 0.1, y, "AuxiliarEncomiendas", meths=["+registrarEnvio() : Envio"])
    cen = c.cls("cen", aux.r + 0.50, y, "ConstructorEnvio", kind="interface",
                meths=["+inspeccionar() : void", "+fijarPesoDims(peso, dims) : void", "+fijarServicio(s) : void",
                       "+fijarActores(r, d) : void", "+fijarPago(pago) : void", "+emitirGuia() : void",
                       "+fijarBodega(b) : void", "+despachar(viaje) : void", "+generarPlanilla() : void",
                       "+construir() : Envio"])
    cim = c.cls("cim", cen.r + 0.45, y, "ConstructorEnvioImpl", attrs=["-envio : Envio"])
    yy = y + 0.36
    c.edge("dep", [c.pt("aux", "E", at=yy), c.pt("cen", "W", at=yy)], "aux", "cen", label=("dirige", 0, 0.5, 0, -0.10))
    c.edge("real", [c.pt("cim", "W", at=yy), c.pt("cen", "E", at=yy)], "cim", "cen")
    W = 6.4
    pw = 1.25
    tiq = c.cls("tiq", 0, 2.6, "Envio", attrs=["-pesoFacturable : double", "-valorDeclarado : double",
                                                "-tipoServicio : string", "-estado : string"], w=1.65)
    tiq.x = W / 2 - tiq.w / 2
    L = [c.cls("gui", 0.1, 0, "Guia", attrs=["-numero : string", "-codigoBarras : string"], w=pw),
         c.cls("bod", 0.1, 0, "Bodega", attrs=["-ubicacion : string"], w=pw),
         c.cls("pla", 0.1, 0, "PlanillaCarga", attrs=["-conductor : string", "-firma : bool"], w=pw)]
    R = [c.cls("rut", W - 0.1 - pw, 0, "Ruta", attrs=["-origen : string", "-destino : string"], w=pw),
         c.cls("cli", W - 0.1 - pw, 0, "Cliente", attrs=["-documento : string"], w=pw),
         c.cls("srv", W - 0.1 - pw, 0, "ServicioEnvio", kind="interface", w=pw, fill=GREY)]
    ymid = cen.b + 0.85 + tiq.h / 2
    tiq.y = ymid - tiq.h / 2
    c.edge("dep", ortho(c.pt("cim", "S"), "S", c.pt("tiq", "N"), "N", mid=cen.b + 0.30), "cim", "tiq",
           label=("obtiene", 1, 0.5, 0, -0.10))
    kinds = {"gui": ("comp", "guia"), "bod": ("agg", "ubica"), "pla": ("agg", "planilla"),
             "rut": ("agg", "sigue"), "cli": ("agg", "actores"), "srv": ("agg", "servicio")}
    parts(c, tiq, [b.key for b in L], [b.key for b in R], ymid - 0.72, 0.72, kinds,
          mults={"bod": "0..1", "cli": "1", "gui": "1", "pla": "0..1", "rut": "1", "srv": "1"},
          mtt={"gui": "1", "bod": "0..*", "pla": "1..*", "rut": "0..*", "cli": "0..*", "srv": "0..*"})
    fabs = c.cls("fsrv", 0, 0, "FabricaServicioEnvio", kind="abstract", fill=GREY)
    sb = c.boxes["srv"]
    fabs.x = min(sb.cx - fabs.w / 2, W - 0.1 - fabs.w)
    fabs.y = sb.b + 0.55
    c.edge("dep", [c.pt("fsrv", "N", at=sb.cx), c.pt("srv", "S")], "fsrv", "srv",
           label=("crea", 0, 0.5, 0.12, 0, "left"))
    c.validate()
    c.fit()
    c.render(out)


if __name__ == "__main__":
    fig7("png/fig7.png")
    fig8("png/fig8.png")
    fig9("png/fig9.png")
