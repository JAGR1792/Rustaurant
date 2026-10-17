from uml import Canvas, ortho


def sing(c, key, x, y, name, attrs, meths):
    return c.cls(key, x, y, name, kind="singleton", attrs=attrs, meths=meths)


def fig13(out):
    c = Canvas(6.4, 3.0, fs=8, name="fig13")
    s = sing(c, "s", 3.3, 0.2, "Singleton", ["-instancia : Singleton"],
             ["-Singleton()", "+obtenerInstancia() : Singleton", "+operacion() : void"])
    a = c.cls("a", 0.1, 0.2, "ClienteA")
    b = c.cls("b", 0.1, 1.75, "ClienteB")
    ya = s.y + 0.30
    yb = s.y + 0.85
    c.edge("dep", ortho(c.pt("a", "E"), "E", c.pt("s", "W", at=ya), "W", mid=1.3), "a", "s",
           label=("obtenerInstancia()", 2, 0.5, 0, -0.10))
    c.edge("dep", ortho(c.pt("b", "E"), "E", c.pt("s", "W", at=yb), "W", mid=1.3), "b", "s",
           label=("obtenerInstancia()", 2, 0.5, 0, -0.10))
    c.validate()
    c.fit()
    c.render(out)


def fig14(out):
    fs = 8.0
    while True:
        c = Canvas(6.4, 5.0, fs=fs, name="fig14")
        tq = c.cls("tq", 0.1, 0, "Taquillero", meths=["+vender() : void"])
        g = sing(c, "g", 0, 0.15, "GestorBloqueoSillas", ["-instancia : GestorBloqueoSillas", "-bloqueos : mapa"],
                 ["-GestorBloqueoSillas()", "+obtenerInstancia() : GestorBloqueoSillas",
                  "+bloquear(silla, canal) : bool", "+liberar(silla) : void", "+expirarBloqueos() : void"])
        sq = sing(c, "sq", 0, g.b + 0.32, "SecuenciadorTiquete", ["-instancia : SecuenciadorTiquete",
                                                               "-consecutivo : int"],
                  ["-SecuenciadorTiquete()", "+obtenerInstancia() : SecuenciadorTiquete", "+siguiente() : string"])
        v = sing(c, "v", 0, sq.b + 0.32, "ValidadorTarifaEspecial", ["-instancia : ValidadorTarifaEspecial",
                                                                    "-cache : mapa"],
                 ["-ValidadorTarifaEspecial()", "+obtenerInstancia() : ValidadorTarifaEspecial",
                  "+validar(tarifa, datos) : bool"])
        w = max(g.w, sq.w, v.w)
        for b in (g, sq, v):
            b.w = w
        sil = c.cls("sil", 0, 0, "Silla", attrs=["-numero : int", "-estado : string"])
        tiq = c.cls("tiq", 0, 0, "Tiquete", attrs=["-numero : string"])
        gap = 0.62
        xb = tq.r + 0.32
        x0 = xb + 0.55
        for b in (g, sq, v):
            b.x = x0
        rw = max(sil.w, tiq.w)
        sil.w = tiq.w = rw
        sil.x = tiq.x = x0 + w + 1.05
        need = sil.r + 0.12
        if need <= 6.4 or fs <= 6.5:
            break
        fs -= 0.25
    print("[fig14] fs", fs, "ancho", round(need, 2))
    c.W = max(need, 6.4) if need <= 6.4 else need
    yg = g.y + 0.30
    ys = sq.y + 0.30
    yv = v.y + 0.30
    tq.y = ys - 0.25
    yt = tq.y + 0.30
    # Taquillero -> singletons (tronco compartido)
    c._plain([(tq.r, ys), (xb, ys)], True, src="tq")
    c._plain([(xb, yg), (xb, yv)], True)
    for key, yy, bx in (("g", yg, g), ("sq", ys, sq), ("v", yv, v)):
        c.edge("dep", [(xb, yy), (x0, yy)], None, key, label=("usa", 0, 0.5, 0, -0.10))
    sil.y = g.y + 0.25
    c.edge("assocA", [c.pt("g", "E", at=sil.y + 0.30), c.pt("sil", "W", at=sil.y + 0.30)] if False else
           [(g.r, sil.y + 0.30), (sil.x, sil.y + 0.30)], "g", "sil", label=("controla", 0, 0.5, 0, -0.10),
           mt=("1", -1), mh=("1..*", -1))
    tiq.y = v.cy - tiq.h / 2
    c.edge("dep", [(sq.r, sq.cy), (tiq.cx, sq.cy), (tiq.cx, tiq.y)], "sq", "tiq",
           label=("numera", 0, 0.5, 0, -0.10))
    c.edge("dep", [(v.r, v.cy), (tiq.x, v.cy)], "v", "tiq", label=("valida", 0, 0.5, 0, -0.10))
    c.H = v.b + 0.15
    c.validate()
    c.render(out)


def fig15(out):
    fs = 8.0
    while True:
        c = Canvas(6.4, 6.0, fs=fs, name="fig15")
        aux = c.cls("aux", 0, 0, "AuxiliarEncomiendas", meths=["+registrar() : void"])
        sec = sing(c, "sec", 0, 0, "SecuenciadorGuia", ["-instancia : SecuenciadorGuia", "-consecutivo : int"],
                   ["-SecuenciadorGuia()", "+obtenerInstancia() :\n   SecuenciadorGuia", "+siguiente() : string"])
        env = c.cls("env", 0, 0, "Envio", attrs=["-guia : string", "-pesoFacturable : double",
                                                  "-pesoVolumetrico : double", "-estado : string"])
        ges = sing(c, "ges", 0, 0, "GestorModoOffline", ["-instancia : GestorModoOffline", "-cola : lista"],
                   ["-GestorModoOffline()", "+obtenerInstancia() :\n   GestorModoOffline", "+encolar(op) : void",
                    "+sincronizar() : void"])
        val = sing(c, "val", 0, 0, "ValidadorCapacidadBus",
                   ["-instancia :\n   ValidadorCapacidadBus", "-cargaPorViaje : mapa"],
                   ["-ValidadorCapacidadBus()", "+obtenerInstancia() :\n   ValidadorCapacidadBus",
                    "+validar(viaje, peso,\n   prioridad) : bool"])
        Lw = aux.w
        Cw = max(sec.w, env.w, ges.w)
        gap = 0.55
        need = 0.1 + Lw + gap + Cw + gap + val.w + 0.12
        if need <= 6.4 or fs <= 6.5:
            break
        fs -= 0.25
    print("[fig15] fs", fs, "ancho", round(need, 2))
    for b in (sec, env, ges):
        b.w = Cw
    xL, xC = 0.1, 0.1 + Lw + gap
    xR = xC + Cw + gap
    aux.x = xL
    sec.x = env.x = ges.x = xC
    val.x = xR
    sec.y = 0.15
    env.y = sec.b + 0.62
    ges.y = env.b + 0.62
    aux.y = sec.y
    val.y = env.cy - val.h / 2
    ya = aux.y + 0.28
    c.edge("dep", [(aux.r, ya), (sec.x, ya)], "aux", "sec", label=("usa", 0, 0.5, 0, -0.10))
    c.edge("dep", [c.pt("sec", "S"), c.pt("env", "N")], "sec", "env", label=("numera", 0, 0.5, 0.10, 0, "left"))
    c.edge("dep", [c.pt("ges", "N"), c.pt("env", "S")], "ges", "env", label=("respalda", 0, 0.5, 0.10, 0, "left"))
    c.edge("dep", [(val.x, env.cy), (env.r, env.cy)], "val", "env", label=("autoriza", 0, 0.5, 0, -0.10))
    c.W = need
    c.H = max(ges.b, val.b) + 0.15
    c.validate()
    c.render(out)


if __name__ == "__main__":
    fig13("png/fig13.png")
    fig14("png/fig14.png")
    fig15("png/fig15.png")
