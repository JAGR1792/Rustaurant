"""Reescribe fig15 sin CalculadorPeso ni ConfiguracionTarifas."""
import os

BASE = os.path.dirname(os.path.abspath(__file__))
FP = os.path.join(BASE, "fig_single.py")
s = open(FP, encoding="utf-8").read()

ini = s.index("def fig15(out):")
fin = s.index('if __name__ == "__main__":')
nueva = '''def fig15(out):
    fs = 8.0
    while True:
        c = Canvas(6.4, 6.0, fs=fs, name="fig15")
        aux = c.cls("aux", 0, 0, "AuxiliarEncomiendas", meths=["+registrar() : void"])
        sec = sing(c, "sec", 0, 0, "SecuenciadorGuia", ["-instancia : SecuenciadorGuia", "-consecutivo : int"],
                   ["-SecuenciadorGuia()", "+obtenerInstancia() :\\n   SecuenciadorGuia", "+siguiente() : string"])
        env = c.cls("env", 0, 0, "Envio", attrs=["-guia : string", "-pesoFacturable : double",
                                                  "-pesoVolumetrico : double", "-estado : string"])
        ges = sing(c, "ges", 0, 0, "GestorModoOffline", ["-instancia : GestorModoOffline", "-cola : lista"],
                   ["-GestorModoOffline()", "+obtenerInstancia() :\\n   GestorModoOffline", "+encolar(op) : void",
                    "+sincronizar() : void"])
        val = sing(c, "val", 0, 0, "ValidadorCapacidadBus",
                   ["-instancia :\\n   ValidadorCapacidadBus", "-cargaPorViaje : mapa"],
                   ["-ValidadorCapacidadBus()", "+obtenerInstancia() :\\n   ValidadorCapacidadBus",
                    "+validar(viaje, peso,\\n   prioridad) : bool"])
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


'''
open(FP, "w", encoding="utf-8").write(s[:ini] + nueva + s[fin:])
print("fig15 reescrita")
