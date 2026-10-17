from uml import Canvas

GREY = "#EFEFEF"


def fig1(out):
    c = Canvas(6.4, 3.45, fs=8, name="fig1")
    Y0 = 0.12
    cli = c.cls("cli", 0.15, Y0, "Cliente")
    cre = c.cls("cre", 0.15, 1.42, "Creador", meths=["+metodoFabrica() : Producto", "+operacion() : void"],
                kind="abstract")
    # cliente centrado sobre Creador
    cli.x = cre.cx - cli.w / 2
    xa = cre.r + 0.85
    ca = c.cls("ca", xa, 0.12, "CreadorConcretoA", meths=["+metodoFabrica() : ProductoA"])
    pr = c.cls("pr", ca.r + 0.75, 1.50, "Producto", meths=["+operacion() : void"], kind="interface")
    pa = c.cls("pa", 0, 0.12, "ProductoA")
    pa.x = pr.cx - pa.w / 2
    pa.y = ca.cy - pa.h / 2
    cb = c.cls("cb", xa, 2.62, "CreadorConcretoB", meths=["+metodoFabrica() : ProductoB"])
    pb = c.cls("pb", 0, 0, "ProductoB")
    pb.x = pr.cx - pb.w / 2
    pb.y = cb.cy - pb.h / 2
    # ajustar creador vertical al centro de la zona
    # Cliente -> Creador
    c.edge("dep", [c.pt("cli", "S"), c.pt("cre", "N")], "cli", "cre", label=("usa", 0, 0.5, 0.16, 0, "left"))
    # Generalización desde A y B hacia el lado E del Creador
    xm = cre.r + 0.42
    yA = c.pt("cre", "E", t=0.22)
    yB = c.pt("cre", "E", t=0.78)
    c.edge("gen", [c.pt("ca", "W"), (xm, ca.cy), (xm, yA[1]), yA], "ca", "cre")
    c.edge("gen", [c.pt("cb", "W"), (xm, cb.cy), (xm, yB[1]), yB], "cb", "cre")
    # Creador -> Producto
    c.edge("dep", [c.pt("cre", "E", t=0.5), c.pt("pr", "W", at=cre.cy)], "cre", "pr",
           label=("usa", 0, 0.62, 0, -0.10))
    # crea
    c.edge("dep", [c.pt("ca", "E"), c.pt("pa", "W", at=ca.cy)], "ca", "pa", label=("crea", 0, 0.5, 0, -0.10))
    c.edge("dep", [c.pt("cb", "E"), c.pt("pb", "W", at=cb.cy)], "cb", "pb", label=("crea", 0, 0.5, 0, -0.10))
    # realización
    c.edge("real", [c.pt("pa", "S"), c.pt("pr", "N", at=pa.cx)], "pa", "pr")
    c.edge("real", [c.pt("pb", "N"), c.pt("pr", "S", at=pb.cx)], "pb", "pr")
    c.validate()
    c.fit()
    c.render(out)


def fig2(out):
    c = Canvas(6.4, 4.5, fs=8, name="fig2")
    taq = c.cls("taq", 0.1, 0.12, "Taquillero", meths=["+venderTiquete() : Tiquete"])
    fab = c.cls("fab", 2.2, 0.12, "FabricaTarifa", meths=["+crearTarifa(tipo, datos) : Tarifa"], kind="abstract")
    imp = c.cls("imp", fab.x + 0.4, 1.10, "FabricaTarifaImpl")
    imp.x = fab.cx - imp.w / 2
    dat = c.cls("dat", 4.75, 1.02, "DatosPasajero", attrs=["-documento : string", "-soporte : string"])
    nota = c.note("nota", 0.1, 1.02, ["Crea UNA de las 4 tarifas", "según tipo + soporte:", "Plena, AdultoMayor,",
                                        "Estudiante, Infante"])
    tar = c.cls("tar", 2.3, 2.02, "Tarifa", meths=["+calcularValor(base : double) : double",
                                                     "+validarSoporte(s : string) : bool",
                                                     "+obtenerDescuento() : int"], kind="interface")
    tiq = c.cls("tiq", 0.1, 2.35, "Tiquete", attrs=["-valorFinal : double"])
    # hijos
    y4 = 3.65
    hs = [
        c.cls("tp", 0.25, y4, "TarifaPlena"),
        c.cls("ta", 1.55, y4, "TarifaAdultoMayor", attrs=["-edadMinima : int = 60"]),
        c.cls("te", 3.35, y4, "TarifaEstudiante"),
        c.cls("ti", 4.85, y4, "TarifaInfante", attrs=["-edadMaxima : int = 11"]),
    ]
    # Taquillero -> FabricaTarifa
    c.edge("dep", [c.pt("taq", "E", at=fab.y + 0.22), c.pt("fab", "W", at=fab.y + 0.22)], "taq", "fab",
           label=("usa", 0, 0.5, 0, -0.10))
    # Impl -|> Fabrica
    c.edge("gen", [c.pt("imp", "N"), c.pt("fab", "S", at=imp.cx)], "imp", "fab")
    # Impl -> Datos (valida)
    c.edge("dep", [c.pt("imp", "E", at=imp.cy), c.pt("dat", "W", at=imp.cy)], "imp", "dat",
           label=("valida", 0, 0.5, 0, -0.10))
    # Impl -> Tarifa (crea)
    c.edge("dep", [c.pt("imp", "S"), c.pt("tar", "N", at=imp.cx)], "imp", "tar",
           label=("crea", 0, 0.5, 0.20, 0, "left"))
    # nota
    c.edge("note", [c.pt("nota", "E", at=imp.cy), c.pt("imp", "W")], "nota", "imp")
    # Tiquete ◇— Tarifa
    yr = tar.y + 0.25
    tiq.w = 1.2
    c.edge("agg", [c.pt("tiq", "E", at=tiq.cy), (tar.x, tiq.cy)], "tiq", "tar",
           label=("aplica", 0, 0.5, 0, -0.10), mt=("0..*", -1), mh=("1", 1))
    # árbol de realización hacia Tarifa
    c.tree_v("real", "tar", tar.cx, y4 - 0.32, [h.key for h in hs])
    c.validate()
    c.fit()
    c.render(out)


def fig3(out):
    c = Canvas(6.4, 4.7, fs=8, name="fig3")
    aux = c.cls("aux", 0.1, 0.12, "AuxiliarEncomiendas", meths=["+registrarEnvio() : void"])
    fab = c.cls("fab", 2.3, 0.12, "FabricaServicioEnvio",
                meths=["+crearServicio(tipo, ruta) : ServicioEnvio"], kind="abstract")
    imp = c.cls("imp", 0, 1.10, "FabricaServicioEnvioImpl")
    imp.x = fab.cx - imp.w / 2
    rut = c.cls("rut", 5.1, 1.95, "Ruta", attrs=["-origen : string", "-destino : string", "-tarifaKm : double"])
    srv = c.cls("srv", 2.05, 2.05, "ServicioEnvio", meths=["+calcularRecargo(flete : double) : double",
                                                            "+obtenerPrioridad() : int",
                                                            "+obtenerSLAHoras() : int"], kind="interface")
    srv.x = fab.cx - srv.w / 2
    env = c.cls("env", 0.1, 2.45, "Envio", attrs=["-flete : double", "-recargo : double"])
    y4 = 3.6
    est = c.cls("est", 1.0, y4, "ServicioEstandar", attrs=["-recargo : double = 0.0", "-prioridad : int = 2"])
    pri = c.cls("pri", 3.35, y4, "ServicioPrioritario", attrs=["-recargo : double = 0.35", "-prioridad : int = 1",
                                                                "-slaHoras : int = 24"])
    c.edge("dep", [c.pt("aux", "E", at=fab.y + 0.22), c.pt("fab", "W", at=fab.y + 0.22)], "aux", "fab",
           label=("usa", 0, 0.5, 0, -0.10))
    c.edge("gen", [c.pt("imp", "N"), c.pt("fab", "S", at=imp.cx)], "imp", "fab")
    c.edge("dep", [c.pt("imp", "E", at=imp.cy), (rut.cx, imp.cy), c.pt("rut", "N")], "imp", "rut",
           label=("usa tarifa", 0, 0.5, 0, -0.10))
    c.edge("dep", [c.pt("imp", "S"), c.pt("srv", "N", at=imp.cx)], "imp", "srv",
           label=("crea", 0, 0.5, 0.20, 0, "left"))
    env.w = 1.2
    c.edge("agg", [c.pt("env", "E", at=env.cy), (srv.x, env.cy)], "env", "srv",
           label=("aplica", 0, 0.5, 0, -0.10), mt=("0..*", -1), mh=("1", 1))
    c.tree_v("real", "srv", srv.cx, y4 - 0.30, ["est", "pri"])
    c.validate()
    c.fit()
    c.render(out)


if __name__ == "__main__":
    fig1("png/fig1.png")
    fig2("png/fig2.png")
    fig3("png/fig3.png")
