from uml import Canvas

CREA = "#1F4E79"
TINTS = ["#E3EEFA", "#E5F4E3", "#FCEBD5"]


def build(W, H, fs, spec, name):
    c = Canvas(W, H, fs=fs, name=name)
    m = 0.14
    ncol = len(spec["ifaces"])
    nrow = len(spec["fams"])

    # ---- crear cajas (posición provisional)
    ifs = [c.cls(f"i{j}", 0, 0, n, meths=ms, kind="interface") for j, (n, ms) in enumerate(spec["ifaces"])]
    prods = []
    for r, fam in enumerate(spec["fams"]):
        row = []
        for j, (pn, pa) in enumerate(fam["prods"]):
            row.append(c.cls(f"p{r}_{j}", 0, 0, pn, attrs=pa, fill=TINTS[r % 3]))
        prods.append(row)
    F = c.cls("F", 0, 0, spec["abs_fab"][0], meths=spec["abs_fab"][1], kind="interface")
    facs = [c.cls(f"f{r}", 0, 0, fam["name"], fill=TINTS[r % 3]) for r, fam in enumerate(spec["fams"])]
    cn, ca, cm = spec["client"]
    cli = c.cls("cli", 0, 0, cn, attrs=ca, meths=cm)

    # ---- anchos coherentes
    for j in range(ncol):
        cw = max(prods[r][j].w for r in range(nrow))
        for r in range(nrow):
            prods[r][j].w = cw
        ifs[j].w = max(ifs[j].w, cw + 0.30)
    fw = max(f.w for f in facs)
    for f in facs:
        f.w = fw
    F.w = max(F.w, fw + 0.30)

    # ---- posiciones X
    gapc = spec.get("gapc", 0.22)
    x = m
    for j in range(ncol):
        ifs[j].x = x
        x += ifs[j].w + gapc
    xF = x - gapc + spec.get("gapF", 0.60)
    F.x = xF
    for f in facs:
        f.x = xF
    for r in range(nrow):
        for j in range(ncol):
            prods[r][j].x = ifs[j].r - prods[r][j].w
    cli.x = F.r - cli.w

    # ---- posiciones Y
    cli.y = m
    y0 = cli.b + 0.40
    for b in ifs + [F]:
        b.y = y0
    row0_h = max(b.h for b in ifs + [F])
    y = y0 + row0_h + 0.40
    bus_y = []
    for r in range(nrow):
        rowh = max([prods[r][j].h for j in range(ncol)] + [facs[r].h])
        for j in range(ncol):
            prods[r][j].y = y
        facs[r].y = y
        bus_y.append(y + rowh + 0.24)
        y = bus_y[-1] + 0.32
    # ---- aristas
    c.edge("dep", [c.pt("cli", "S", at=F.cx), c.pt("F", "N", at=F.cx)], "cli", "F",
           label=("usa", 0, 0.5, 0.12, 0, "left"))
    c.spine("real", "F", F.r - 0.14, [f.key for f in facs], side="E")
    for j in range(ncol):
        c.spine("real", f"i{j}", ifs[j].x + 0.14, [prods[r][j].key for r in range(nrow)], side="W")
    for r in range(nrow):
        f = facs[r]
        sx = f.x + 0.30
        first = prods[r][0]
        last = prods[r][-1]
        path = [(sx, f.b), (sx, bus_y[r]), (first.cx, bus_y[r])]
        xg = (last.r + xF) / 2
        frac = (sx - xg) / (sx - first.cx)
        c.edges.append(dict(kind="plain_dashed", pts=path, src=f.key, dst=None, color=CREA, i=len(c.edges)))
        c.labels.append(dict(text="crea", x=xg, y=bus_y[r] + 0.105, ha="center", edge=len(c.edges) - 1, kind="label"))
        for j in range(ncol):
            p = prods[r][j]
            c.edge("dep", [(p.cx, bus_y[r]), (p.cx, p.b)], None, p.key, color=CREA)
    need_w = F.r + m
    need_h = bus_y[-1] + 0.22
    return c, need_w, need_h, dict(ifs=ifs, prods=prods, F=F, facs=facs, cli=cli, bus_y=bus_y)


def make(spec, name, W, H, out, fs_start=8.0, extra=None):
    fs = fs_start
    while True:
        c, nw, nh, info = build(W, H, fs, spec, name)
        if nw <= W - 0.02 or fs <= 6.2:
            break
        fs -= 0.25
    print(f"[{name}] fs={fs:.2f} ancho necesario {nw:.2f} de {W}, alto {nh:.2f} de {H}")
    if extra:
        extra(c, info)
    nw = max(nw, max(b.r for b in c.boxes.values()) + 0.12)
    nh = max(nh, max(b.b for b in c.boxes.values()) + 0.12)
    c.W, c.H = nw, nh
    if spec.get("legend"):
        c.legend = (0.14, 0.14, [("real", "realiza (implementa la interfaz)", "black"),
                                  ("dep", "crea (dependencia)", CREA)])
    c.validate()
    c.render(out)
    return c


SPEC5 = dict(
    ifaces=[
        ("BloqueoSilla", ["+bloquear(silla) : bool", "+liberar(silla) : void"]),
        ("ValidadorPago", ["+validar(datos) : bool"]),
        ("EmisorFactura", ["+emitir(tiquete) : Factura"]),
        ("Notificador", ["+avisar(tiquete) : void"]),
    ],
    abs_fab=("FabricaVenta", ["+crearBloqueo() : BloqueoSilla", "+crearPago() : ValidadorPago",
                              "+crearFactura() : EmisorFactura", "+crearAviso() : Notificador"]),
    client=("ServicioVentaTiquete", ["-fabrica : FabricaVenta"],
            ["+ServicioVentaTiquete(f : FabricaVenta)", "+fijarCanal(c) : void", "+construir() : Tiquete"]),
    fams=[
        dict(name="FabricaTaquilla", prods=[("BloqueoTaquilla", ["-tiempoSeg : int = 180"]), ("PagoTaquilla", []),
                                             ("FacturaTaquilla", []), ("AvisoTaquilla", [])]),
        dict(name="FabricaWeb", prods=[("BloqueoWeb", []), ("PagoWeb", []), ("FacturaWeb", []), ("AvisoWeb", [])]),
        dict(name="FabricaAliada", prods=[("BloqueoAliada", []), ("PagoAliada", []), ("FacturaAliada", []),
                                          ("AvisoAliada", [])]),
    ],
    legend=True,
)

SPEC6 = dict(
    ifaces=[
        ("CalculadorFlete", ["+calcular(ruta, peso) : double", "+obtenerPrioridad() : int"]),
        ("ValidadorPeso", ["+validar(peso, dims) : bool"]),
        ("GeneradorGuia", ["+generar(datos) : Guia", "+generarLote(n, base) : list"]),
        ("NotificadorEnvio", ["+avisar(envio) : void"]),
        ("ValidadorCredito", ["+validarCupo(cliente,\n   monto) : bool"]),
    ],
    abs_fab=("FabricaEnvio", ["+crearFlete() : CalculadorFlete", "+crearPeso() : ValidadorPeso",
                              "+crearGuia() : GeneradorGuia", "+crearAviso() : NotificadorEnvio",
                              "+crearCredito() : ValidadorCredito"]),
    client=("ServicioRegistroEnvio", ["-fabrica : FabricaEnvio"],
            ["+ServicioRegistroEnvio(f : FabricaEnvio)", "+fijarModalidad(m) : void", "+construir() : Envio"]),
    fams=[
        dict(name="FabricaPaqueteo", prods=[("FletePaqueteo", []), ("PesoPaqueteo", ["-maxKg : int = 30"]),
                                             ("GuiaPaqueteo", []), ("AvisoPaqueteo", []), ("CreditoPaqueteo", [])]),
        dict(name="FabricaCorporativa", prods=[("FleteCorporativa", []),
                                                ("PesoCorporativa", ["-maxKg : int = 500"]),
                                                ("GuiaCorporativa", []), ("AvisoCorporativa", []),
                                                ("CreditoCorporativa", [])]),
        dict(name="FabricaRemesa", prods=[("FleteRemesa", []), ("PesoRemesa", []), ("GuiaRemesa", []),
                                           ("AvisoRemesa", []), ("CreditoRemesa", [])]),
    ],
    gapc=0.16,
    gapF=0.55,
    legend=True,
)

SPEC4 = dict(
    ifaces=[
        ("ProductoAbstractoA", ["+operacionA() : void"]),
        ("ProductoAbstractoB", ["+operacionB() : void"]),
    ],
    abs_fab=("FabricaAbstracta", ["+crearProductoA() : ProductoAbstractoA", "+crearProductoB() : ProductoAbstractoB"]),
    client=("Cliente", ["-fabrica : FabricaAbstracta"], ["+Cliente(f : FabricaAbstracta)", "+operacion() : void"]),
    fams=[
        dict(name="FabricaConcreta1", prods=[("ProductoA1", []), ("ProductoB1", [])]),
        dict(name="FabricaConcreta2", prods=[("ProductoA2", []), ("ProductoB2", [])]),
    ],
    gapc=0.30,
    gapF=0.70,
)


def extra4(c, info):
    cli, F, facs = info["cli"], info["F"], info["facs"]
    n1 = c.note("n1", 0.14, cli.y + 0.05, ["ProductoAbstractoA pa =", "  fabrica.crearProductoA()"])
    c.edge("note", [c.pt("n1", "E", at=cli.cy), c.pt("cli", "W")], "n1", "cli")
    f2 = facs[1]
    n2 = c.note("n2", f2.x + 0.55, f2.b + 0.50, ["return new ProductoA2()"])
    xn = f2.x + 1.05
    c.edge("note", [(xn, f2.b), (xn, n2.y)], "f1", "n2")


if __name__ == "__main__":
    make(SPEC5, "fig5", 9.95, 6.2, "png/fig5.png", fs_start=8.0)
    make(SPEC6, "fig6", 9.95, 6.2, "png/fig6.png", fs_start=8.0)
    make(SPEC4, "fig4", 6.4, 5.6, "png/fig4.png", fs_start=8.0, extra=extra4)
