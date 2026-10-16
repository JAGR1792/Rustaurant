from uml import Canvas


def reg_box(c, key, x, y, ret):
    return c.cls(key, x, y, "RegistroPrototipos", kind="singleton", attrs=["-prototipos : map"],
                 meths=["+registrar(clave, p) : void", f"+clonar(clave) : {ret}"])


def fig10(out):
    c = Canvas(6.4, 3.6, fs=8, name="fig10")
    y = 0.15
    cli = c.cls("cli", 0.1, y, "Cliente")
    reg = reg_box(c, "reg", cli.r + 0.95, y, "Prototipo")
    pro = c.cls("pro", reg.r + 1.0, y, "Prototipo", kind="interface", meths=["+clonar() : Prototipo"])
    yy = y + 0.55
    c.edge("dep", [c.pt("cli", "E", at=y + 0.25), c.pt("reg", "W", at=y + 0.25)], "cli", "reg",
           label=("pide copia", 0, 0.5, 0, -0.10))
    c.edge("agg", [c.pt("reg", "E", at=yy), c.pt("pro", "W", at=yy)], "reg", "pro",
           label=("guarda", 0, 0.5, 0, -0.10), mt=("1", -1), mh=("0..*", -1))
    y2 = pro.b + 0.65
    p1 = c.cls("p1", 0, y2, "PrototipoConcreto1", attrs=["-campo1 : string"], meths=["+clonar() : Prototipo"])
    p2 = c.cls("p2", 0, y2, "PrototipoConcreto2", attrs=["-campo2 : int"], meths=["+clonar() : Prototipo"])
    p2.x = 6.3 - p2.w
    p1.x = p2.x - 0.3 - p1.w
    c.tree_v("real", "pro", pro.cx, y2 - 0.32, ["p1", "p2"])
    n = c.note("n", 0.1, y2 + 0.05, ["clonar() devuelve una copia", "del ejemplar: superficial", "o profunda, según la clase"])
    c.edge("note", [c.pt("n", "E", at=p1.y + 0.4), c.pt("p1", "W", at=p1.y + 0.4)], "n", "p1")
    c.validate()
    c.fit()
    c.render(out)


def fig11(out):
    c = Canvas(6.4, 5.0, fs=8, name="fig11")
    jefe = c.cls("jefe", 0.1, 0.15, "JefeOperaciones", meths=["+programarViaje() : Viaje"])
    reg = reg_box(c, "reg", 2.75, 0.15, "PrototipoViaje")
    tq = c.cls("tq", 0.1, 1.55, "Taquillero", meths=["+reprogramar() : TiqueteAbierto"])
    pro = c.cls("pro", 0, reg.b + 0.65, "PrototipoViaje", kind="interface", meths=["+clonar() : PrototipoViaje"])
    pro.x = reg.x + 0.75
    yb = pro.b + 0.32
    yc = yb + 0.32
    via = c.cls("via", 0, yc, "Viaje", attrs=["-ruta : string", "-horario : string", "-bus : string",
                                                "-tarifaBase : double"], meths=["+clonar() : Viaje"])
    tab = c.cls("tab", 0, yc, "TiqueteAbierto", attrs=["-vigenciaMeses : int = 3", "-penalidad : double"],
                meths=["+clonar() : TiqueteAbierto"])
    tab.x = 6.3 - tab.w
    via.x = tab.x - 0.35 - via.w
    yj = 0.15 + 0.36
    c.edge("dep", [c.pt("jefe", "E", at=yj), c.pt("reg", "W", at=yj)], "jefe", "reg",
           label=("clona viaje", 0, 0.5, 0, -0.10))
    xs = reg.x + 0.32
    c.edge("dep", [c.pt("tq", "E", at=tq.cy), (xs, tq.cy), (xs, reg.b)][:3], "tq", "reg",
           label=("clona tiquete", 0, 0.5, 0, -0.10))
    c.edge("agg", [c.pt("reg", "S", at=pro.cx), c.pt("pro", "N")], "reg", "pro",
           label=("guarda", 0, 0.5, 0.12, 0, "left"), mt=("1", -1), mh=("0..*", -1))
    c.tree_v("real", "pro", pro.cx, yb, ["via", "tab"])
    n1 = c.note("n1", 0.1, via.y + 0.05, ["Copia superficial:", "los campos son valores;", "solo cambia la fecha"])
    c.edge("note", [c.pt("n1", "E", at=via.y + 0.4), c.pt("via", "W", at=via.y + 0.4)], "n1", "via")
    n2 = c.note("n2", 0, max(via.b, tab.b) + 0.30, ["Copia superficial: conserva pago y", "vigencia; solo cambia el viaje"])
    n2.x = 6.3 - n2.w
    c.edge("note", [c.pt("tab", "S", at=tab.cx), (tab.cx, n2.y)], "tab", "n2")
    c.validate()
    c.fit()
    c.render(out)


def fig12(out):
    c = Canvas(6.4, 5.6, fs=8, name="fig12")
    aux = c.cls("aux", 0.1, 0.15, "AuxiliarEncomiendas", meths=["+radicarRemesa() : Remesa"])
    reg = reg_box(c, "reg", 2.55, 0.15, "PrototipoGuia")
    pro = c.cls("pro", 0, reg.b + 0.60, "PrototipoGuia", kind="interface", meths=["+clonar() : PrototipoGuia"])
    pro.x = reg.x + 0.75
    yb = pro.b + 0.32
    yc = yb + 0.32
    tul = c.cls("tul", 0.1, yc, "Tula", attrs=["-precinto : string", "-destino : string"], meths=["+clonar() : Tula"])
    gui = c.cls("gui", 0, yc, "Guia", attrs=["-numero : string", "-remitente : string", "-servicio : string"],
                meths=["+clonar() : Guia"])
    rem = c.cls("rem", 0, yc, "Remesa", attrs=["-numero : string", "-cliente : string"], meths=["+clonar() : Remesa"])
    gui.x = tul.r + 0.80
    rem.x = gui.r + 0.80
    ya = 0.15 + 0.36
    c.edge("dep", [c.pt("aux", "E", at=ya), c.pt("reg", "W", at=ya)], "aux", "reg", label=("clona", 0, 0.5, 0, -0.10))
    c.edge("agg", [c.pt("reg", "S", at=pro.cx), c.pt("pro", "N")], "reg", "pro",
           label=("guarda", 0, 0.5, 0.12, 0, "left"), mt=("1", -1), mh=("0..*", -1))
    c.tree_v("real", "pro", pro.cx, yb, ["tul", "gui", "rem"])
    yy = yc + 0.60
    c.edge("agg", [c.pt("tul", "E", at=yy), c.pt("gui", "W", at=yy)], "tul", "gui",
           label=("contiene", 0, 0.5, 0, -0.10), mt=("0..1", -1), mh=("1..*", 1))
    c.edge("agg", [c.pt("rem", "W", at=yy), c.pt("gui", "E", at=yy)], "rem", "gui",
           label=("contiene", 0, 0.5, 0, -0.10), mt=("0..1", 1), mh=("1..*", -1))
    yn = max(b.b for b in (tul, gui, rem)) + 0.35
    nt = c.note("nt", 0.1, yn, ["Copia profunda: su propia", "lista de guías y su precinto"])
    ng = c.note("ng", 0, yn, ["Copia superficial:", "número, remitente y", "servicio son valores"])
    ng.x = gui.cx - ng.w / 2
    nr = c.note("nr", 0, yn, ["Copia profunda:", "su propio lote de", "guías del cliente"])
    nr.x = rem.cx - nr.w / 2
    for nk, bk, b in (("nt", "tul", tul), ("ng", "gui", gui), ("nr", "rem", rem)):
        nb = c.boxes[nk]
        x = min(max(b.cx, nb.x + 0.3), nb.r - 0.3)
        c.edge("note", [(x, b.b), (x, nb.y)], bk, nk)
    c.validate()
    c.fit()
    c.render(out)


if __name__ == "__main__":
    fig10("png/fig10.png")
    fig11("png/fig11.png")
    fig12("png/fig12.png")
