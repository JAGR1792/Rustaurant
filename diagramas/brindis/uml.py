"""Mini-librería UML: cajas de clase + conectores ortogonales con trazado explícito.

Unidades: pulgadas. Eje y hacia abajo. Todo se dibuja a tamaño real (1 unidad = 1 pulgada),
de modo que la fuente en pt es el tamaño real de impresión en el Word.
Incluye validación: ninguna línea atraviesa una caja, las etiquetas no pisan cajas ni otras líneas.
"""
import math

import matplotlib

matplotlib.use("Agg")
matplotlib.rcParams["svg.fonttype"] = "none"   # el texto queda editable en los SVG
import matplotlib.pyplot as plt
from matplotlib import font_manager as _fm
from matplotlib.backends.backend_agg import RendererAgg
from matplotlib.font_manager import FontProperties
from matplotlib.patches import Polygon, Rectangle

_disp = {f.name for f in _fm.fontManager.ttflist}
FONT = next((f for f in ("Liberation Sans", "Arial", "Helvetica", "DejaVu Sans") if f in _disp), "sans-serif")
_R = RendererAgg(10, 10, 72)

TIPO = {
    "interface": "«interface»",
    "abstract": "«abstracta»",
    "singleton": "«singleton»",
    "utility": "«utility»",
}


def tw(text, size, weight="normal", style="normal"):
    fp = FontProperties(family=FONT, size=size, weight=weight, style=style)
    w, h, d = _R.get_text_width_height_descent(text, fp, False)
    return w / 72.0


class Box:
    def __init__(self, key, x, y, w, h):
        self.key, self.x, self.y, self.w, self.h = key, x, y, w, h

    @property
    def r(self):
        return self.x + self.w

    @property
    def b(self):
        return self.y + self.h

    @property
    def cx(self):
        return self.x + self.w / 2

    @property
    def cy(self):
        return self.y + self.h / 2


class Canvas:
    def __init__(self, W, H, fs=8.0, name="fig"):
        self.W, self.H, self.fs, self.name = W, H, fs, name
        self.lh = fs * 1.30 / 72.0
        self.boxes = {}
        self.cmds = []      # dibujo diferido de cajas
        self.edges = []
        self.labels = []    # (text, cx, cy, ha, owner_edge_index)
        self.notes = {}

    # ------------------------------------------------------------------ cajas
    def cls(self, key, x, y, name, attrs=(), meths=(), kind="class", stereo=None,
            w=None, minw=0.0, fill="white", ec="black", pad=0.075):
        fs, lh = self.fs, self.lh
        italic = kind in ("interface", "abstract")
        if stereo is None:
            stereo = TIPO.get(kind)
        hw = tw(name, fs, "bold", "italic" if italic else "normal")
        if stereo:
            hw = max(hw, tw(stereo, fs))
        lines = lambda seq: [l for a in seq for l in a.split("\n")]
        aw = max([tw(l, fs) for l in lines(attrs)] or [0])
        mw = max([tw(l, fs) for l in lines(meths)] or [0])
        W = w if w else max(hw, aw, mw, minw) + 2 * pad
        head_h = (2 if stereo else 1) * lh + 0.10 if stereo else lh + 0.12
        comps = []
        if attrs:
            comps.append(("attrs", lines(attrs)))
        if meths:
            comps.append(("meths", lines(meths)))
        if not comps:
            comps.append(("empty", []))
        heights = [max(0.13, len(ls) * lh + 0.09) for _, ls in comps]
        H = head_h + sum(heights)
        b = Box(key, x, y, W, H)
        self.boxes[key] = b
        self.cmds.append(("cls", dict(b=b, name=name, stereo=stereo, italic=italic, head_h=head_h,
                                      comps=comps, heights=heights, fill=fill, ec=ec, pad=pad)))
        return b

    def note(self, key, x, y, lines, w=None, fill="#FFF7CC"):
        fs = self.fs * 0.92
        lh = fs * 1.30 / 72.0
        W = w if w else max(tw(l, fs) for l in lines) + 0.20
        H = len(lines) * lh + 0.12
        b = Box(key, x, y, W, H)
        self.boxes[key] = b
        self.cmds.append(("note", dict(b=b, lines=lines, fill=fill, fs=fs, lh=lh)))
        return b

    def pt(self, key, side, t=0.5, at=None):
        """Punto de anclaje en el borde de la caja. at = coordenada absoluta (y para E/W, x para N/S)."""
        b = self.boxes[key]
        if side == "N":
            return (at if at is not None else b.x + t * b.w, b.y)
        if side == "S":
            return (at if at is not None else b.x + t * b.w, b.b)
        if side == "W":
            return (b.x, at if at is not None else b.y + t * b.h)
        if side == "E":
            return (b.r, at if at is not None else b.y + t * b.h)
        raise ValueError(side)

    # ------------------------------------------------------------------ aristas
    def edge(self, kind, pts, src=None, dst=None, label=None, mt=None, mh=None, color="black"):
        """kind: dep, real, gen, agg, comp, assoc, assocA, note, plain.
        pts: lista de puntos; el extremo final lleva la flecha; agg/comp llevan el rombo en el inicial.
        label: (texto, seg, frac, dx, dy[, ha])
        mt / mh: (texto, lado)  multiplicidad en cola / cabeza; lado = +1 o -1 (a qué lado de la línea)
        """
        idx = len(self.edges)
        self.edges.append(dict(kind=kind, pts=[tuple(p) for p in pts], src=src, dst=dst,
                               color=color, i=idx))
        if label:
            text, seg, frac, dx, dy = label[:5]
            ha = label[5] if len(label) > 5 else "center"
            p0, p1 = pts[seg], pts[seg + 1]
            x = p0[0] + (p1[0] - p0[0]) * frac + dx
            y = p0[1] + (p1[1] - p0[1]) * frac + dy
            self.labels.append(dict(text=text, x=x, y=y, ha=ha, edge=idx, kind="label"))
        for spec, end in ((mt, "tail"), (mh, "head")):
            if not spec:
                continue
            text, side = spec
            if end == "tail":
                a, b = pts[0], pts[1]
            else:
                a, b = pts[-1], pts[-2]
            dx, dy = b[0] - a[0], b[1] - a[1]
            L = math.hypot(dx, dy)
            d = (dx / L, dy / L)
            n = (d[1] * side, -d[0] * side)
            off_along, off_perp = 0.13, 0.085
            if end == "tail" and self.edges[idx]["kind"] in ("agg", "comp"):
                off_along = 0.27
            x = a[0] + d[0] * off_along + n[0] * off_perp
            y = a[1] + d[1] * off_along + n[1] * off_perp
            self.labels.append(dict(text=text, x=x, y=y, ha="center", edge=idx, kind="mult"))
        return idx

    def _plain(self, pts, dashed, dst=None, src=None, color="black"):
        self.edges.append(dict(kind="plain_dashed" if dashed else "plain_solid", pts=[tuple(p) for p in pts],
                               src=src, dst=dst, color=color, i=len(self.edges)))

    def tree_v(self, kind, parent, px, bus_y, children, color="black"):
        """Destino compartido, hijos por debajo (o encima) del padre.
        Tronco vertical en x=px desde el padre hasta bus_y; barra horizontal; ramas verticales a cada hijo."""
        P = self.boxes[parent]
        below = bus_y > P.b
        ptop = (px, P.b) if below else (px, P.y)
        self.edges.append(dict(kind=kind, pts=[(px, bus_y), ptop], src=parent, dst=parent, color=color,
                               i=len(self.edges)))
        xs = [self.boxes[k].cx for k in children] + [px]
        lo, hi = min(xs), max(xs)
        dashed = kind == "real"
        if px - lo > 1e-6:
            self._plain([(px, bus_y), (lo, bus_y)], dashed, color=color)
        if hi - px > 1e-6:
            self._plain([(px, bus_y), (hi, bus_y)], dashed, color=color)
        for k in children:
            B = self.boxes[k]
            end = (B.cx, B.y) if B.y > bus_y else (B.cx, B.b)
            self._plain([(B.cx, bus_y), end], dashed, dst=k, color=color)

    def spine(self, kind, parent, px, children, side="W", color="black"):
        """Destino compartido con tronco vertical lateral. Hijos a un lado del tronco (side = lado del hijo que se toca)."""
        P = self.boxes[parent]
        last = max(self.boxes[k].cy for k in children)
        self.edges.append(dict(kind=kind, pts=[(px, last), (px, P.b)], src=parent, dst=parent, color=color,
                               i=len(self.edges)))
        dashed = kind == "real"
        for k in children:
            B = self.boxes[k]
            end = (B.x, B.cy) if side == "W" else (B.r, B.cy)
            if abs(px - end[0]) > 1e-6:
                self._plain([(px, B.cy), end], dashed, dst=k, color=color)

    def fit(self, margin=0.12):
        """Recorta el lienzo al contenido (derecha y abajo)."""
        r = max(b.r for b in self.boxes.values())
        b_ = max(b.b for b in self.boxes.values())
        self.validate(verbose=False)
        for L in self.labels:
            r = max(r, L["rect"][2])
            b_ = max(b_, L["rect"][3])
        self.W = min(self.W, r + margin)
        self.H = min(self.H, b_ + margin)

    # ------------------------------------------------------------------ geometría
    @staticmethod
    def _seg_rect_len(p, q, rect):
        """longitud del segmento p-q dentro del rectángulo (x0,y0,x1,y1) (solo horizontales/verticales)."""
        x0, y0, x1, y1 = rect
        (ax, ay), (bx, by) = p, q
        if abs(ay - by) < 1e-9:  # horizontal
            if y0 < ay < y1:
                lo, hi = max(min(ax, bx), x0), min(max(ax, bx), x1)
                return max(0.0, hi - lo)
            return 0.0
        if abs(ax - bx) < 1e-9:  # vertical
            if x0 < ax < x1:
                lo, hi = max(min(ay, by), y0), min(max(ay, by), y1)
                return max(0.0, hi - lo)
            return 0.0
        return 0.0

    def validate(self, verbose=True):
        issues = []
        W, H = self.W, self.H
        keys = list(self.boxes)
        # cajas dentro del lienzo y sin solaparse
        for k in keys:
            b = self.boxes[k]
            if b.x < 0.02 or b.y < 0.02 or b.r > W - 0.02 or b.b > H - 0.02:
                issues.append(f"caja fuera del lienzo: {k} ({b.x:.2f},{b.y:.2f})-({b.r:.2f},{b.b:.2f}) lienzo {W}x{H}")
        for i, a in enumerate(keys):
            for bkey in keys[i + 1:]:
                A, B = self.boxes[a], self.boxes[bkey]
                if A.x < B.r + 0.05 and B.x < A.r + 0.05 and A.y < B.b + 0.05 and B.y < A.b + 0.05:
                    issues.append(f"cajas solapadas o pegadas: {a} / {bkey}")
        # aristas vs cajas
        for e in self.edges:
            for k in keys:
                b = self.boxes[k]
                ends = {e.get("src"), e.get("dst")}
                if k in ends:
                    rect = (b.x + 0.012, b.y + 0.012, b.r - 0.012, b.b - 0.012)
                else:
                    rect = (b.x - 0.04, b.y - 0.04, b.r + 0.04, b.b + 0.04)
                for p, q in zip(e["pts"], e["pts"][1:]):
                    ln = self._seg_rect_len(p, q, rect)
                    if ln > 0.001:
                        issues.append(f"arista {e['i']} ({e['kind']}) atraviesa/roza la caja {k} ({ln:.2f} in)")
        # etiquetas
        rects = []
        for L in self.labels:
            fs = self.fs * (0.95 if L["kind"] == "label" else 0.92)
            w = tw(L["text"], fs) + 0.05
            h = fs * 1.15 / 72.0 + 0.02
            x0 = L["x"] - w / 2 if L["ha"] == "center" else (L["x"] if L["ha"] == "left" else L["x"] - w)
            L["rect"] = (x0, L["y"] - h / 2, x0 + w, L["y"] + h / 2)
            rects.append(L)
        for L in rects:
            x0, y0, x1, y1 = L["rect"]
            if x0 < 0.02 or y0 < 0.02 or x1 > W - 0.02 or y1 > H - 0.02:
                issues.append(f"etiqueta fuera del lienzo: {L['text']}")
            for k in keys:
                b = self.boxes[k]
                if x0 < b.r + 0.01 and b.x - 0.01 < x1 and y0 < b.b + 0.01 and b.y - 0.01 < y1:
                    issues.append(f"etiqueta '{L['text']}' pisa la caja {k}")
            for e in self.edges:
                if e["i"] == L["edge"] and L["kind"] == "label":
                    continue
                for p, q in zip(e["pts"], e["pts"][1:]):
                    if self._seg_rect_len(p, q, (x0 - 0.005, y0 - 0.005, x1 + 0.005, y1 + 0.005)) > 0.001:
                        # las multiplicidades pegadas a su propia arista son normales
                        if L["kind"] == "mult" and e["i"] == L["edge"]:
                            continue
                        issues.append(f"etiqueta '{L['text']}' cruzada por la arista {e['i']} ({e['kind']})")
            for M in rects:
                if M is L:
                    continue
                a0, b0, a1, b1 = M["rect"]
                if x0 < a1 and a0 < x1 and y0 < b1 and b0 < y1 and id(M) > id(L):
                    issues.append(f"etiquetas solapadas: '{L['text']}' / '{M['text']}'")
        if verbose:
            print(f"[{self.name}] {'OK sin incidencias' if not issues else str(len(issues)) + ' incidencias'}")
            for s in issues:
                print("   -", s)
        return issues

    # ------------------------------------------------------------------ dibujo
    def _hops(self):
        """Puntos de cruce: tramos horizontales de una arista que cruzan tramos verticales de otra."""
        hops = {}
        for e in self.edges:
            if e["kind"] == "note":
                continue
            for si, (p, q) in enumerate(zip(e["pts"], e["pts"][1:])):
                if abs(p[1] - q[1]) > 1e-9:
                    continue
                y = p[1]
                lo, hi = min(p[0], q[0]), max(p[0], q[0])
                for f in self.edges:
                    if f is e or f["kind"] == "note":
                        continue
                    for a, b in zip(f["pts"], f["pts"][1:]):
                        if abs(a[0] - b[0]) > 1e-9:
                            continue
                        x = a[0]
                        ylo, yhi = min(a[1], b[1]), max(a[1], b[1])
                        if lo + 0.07 < x < hi - 0.07 and ylo + 0.03 < y < yhi - 0.03:
                            hops.setdefault((e["i"], si), []).append(x)
        return hops

    def render(self, path, dpi=300):
        fig = plt.figure(figsize=(self.W, self.H), dpi=dpi)
        fig.patch.set_facecolor("white")
        ax = fig.add_axes([0, 0, 1, 1])
        ax.set_xlim(0, self.W)
        ax.set_ylim(self.H, 0)
        ax.axis("off")
        LW = 0.8
        hops = self._hops()

        def line(xs, ys, dashed=False, color="black", z=2):
            ax.plot(xs, ys, color=color, lw=LW, solid_capstyle="butt",
                    dashes=(4.2, 2.6) if dashed else (None, None), zorder=z)

        for e in self.edges:
            kind, pts, color = e["kind"], list(e["pts"]), e["color"]
            dashed = kind in ("dep", "real", "plain_dashed")
            dotted = kind == "note"
            # recorte para el marcador
            start_trim = end_trim = 0.0
            if kind == "real" or kind == "gen":
                end_trim = 0.115
            elif kind in ("agg", "comp"):
                start_trim = 0.17
            def trim(pts_, start, end):
                pts_ = list(pts_)
                if start:
                    (x0, y0), (x1, y1) = pts_[0], pts_[1]
                    L = math.hypot(x1 - x0, y1 - y0)
                    pts_[0] = (x0 + (x1 - x0) / L * start, y0 + (y1 - y0) / L * start)
                if end:
                    (x0, y0), (x1, y1) = pts_[-1], pts_[-2]
                    L = math.hypot(x1 - x0, y1 - y0)
                    pts_[-1] = (x0 + (x1 - x0) / L * end, y0 + (y1 - y0) / L * end)
                return pts_
            tp = trim(pts, start_trim, end_trim)
            # construir vértices con saltos
            verts = []
            for si in range(len(tp) - 1):
                p, q = tp[si], tp[si + 1]
                if si == 0:
                    verts.append(p)
                hx = sorted(hops.get((e["i"], si), []), key=lambda v: v if q[0] > p[0] else -v)
                if hx and abs(p[1] - q[1]) < 1e-9:
                    sgn = 1 if q[0] > p[0] else -1
                    r = 0.05
                    for x in hx:
                        verts.append((x - sgn * r, p[1]))
                        n = 14
                        for k in range(1, n):
                            th = math.pi * k / n
                            verts.append((x - sgn * r * math.cos(th), p[1] - r * math.sin(th)))
                        verts.append((x + sgn * r, p[1]))
                verts.append(q)
            xs, ys = [v[0] for v in verts], [v[1] for v in verts]
            if dotted:
                ax.plot(xs, ys, color="#666666", lw=0.7, dashes=(1.2, 2.2), zorder=1)
            else:
                line(xs, ys, dashed=dashed, color=color)
            # marcadores
            if kind in ("dep", "assocA"):
                self._vee(ax, pts[-1], pts[-2], color)
            elif kind in ("real", "gen"):
                self._tri(ax, pts[-1], pts[-2], color)
            elif kind in ("agg", "comp"):
                self._diamond(ax, pts[0], pts[1], filled=(kind == "comp"), color=color)
            elif e.get("trunk"):
                pass

        for c, d in self.cmds:
            if c == "cls":
                self._draw_cls(ax, d, LW)
            else:
                self._draw_note(ax, d)

        for L in self.labels:
            fs = self.fs * (0.95 if L["kind"] == "label" else 0.92)
            x0, y0, x1, y1 = L["rect"] if "rect" in L else (0, 0, 0, 0)
            if "rect" not in L:
                self.validate(verbose=False)
                x0, y0, x1, y1 = L["rect"]
            ax.add_patch(Rectangle((x0, y0), x1 - x0, y1 - y0, fc="white", ec="none", zorder=5))
            ax.text((x0 + x1) / 2, (y0 + y1) / 2 + 0.004, L["text"], fontsize=fs, family=FONT,
                    ha="center", va="center", zorder=6, color="black")
        lg = getattr(self, "legend", None)
        if lg:
            x, y, items = lg
            ax.text(x, y, "Leyenda", fontsize=self.fs, family=FONT, fontweight="bold", ha="left", va="center")
            yy = y + 0.26
            for kind, text, color in items:
                if kind == "real":
                    ax.plot([x, x + 0.5], [yy, yy], color=color, lw=0.8, dashes=(4.2, 2.6))
                    self._tri(ax, (x + 0.5, yy), (x, yy), color)
                else:
                    ax.plot([x, x + 0.5], [yy, yy], color=color, lw=0.8, dashes=(4.2, 2.6))
                    self._vee(ax, (x + 0.5, yy), (x, yy), color)
                ax.text(x + 0.62, yy, text, fontsize=self.fs * 0.95, family=FONT, ha="left", va="center")
                yy += 0.22
        fig.savefig(path, dpi=dpi, facecolor="white")
        plt.close(fig)

    # marcadores -----------------------------------------------------------
    @staticmethod
    def _unit(a, b):
        dx, dy = b[0] - a[0], b[1] - a[1]
        L = math.hypot(dx, dy)
        return dx / L, dy / L

    def _vee(self, ax, tip, prev, color):
        d = self._unit(tip, prev)
        n = (-d[1], d[0])
        L, Wd = 0.10, 0.042
        p1 = (tip[0] + d[0] * L + n[0] * Wd, tip[1] + d[1] * L + n[1] * Wd)
        p2 = (tip[0] + d[0] * L - n[0] * Wd, tip[1] + d[1] * L - n[1] * Wd)
        ax.plot([p1[0], tip[0], p2[0]], [p1[1], tip[1], p2[1]], color=color, lw=0.8,
                solid_joinstyle="miter", zorder=3)

    def _tri(self, ax, tip, prev, color):
        d = self._unit(tip, prev)
        n = (-d[1], d[0])
        L, Wd = 0.115, 0.058
        b1 = (tip[0] + d[0] * L + n[0] * Wd, tip[1] + d[1] * L + n[1] * Wd)
        b2 = (tip[0] + d[0] * L - n[0] * Wd, tip[1] + d[1] * L - n[1] * Wd)
        ax.add_patch(Polygon([tip, b1, b2], closed=True, fc="white", ec=color, lw=0.8, zorder=3))

    def _diamond(self, ax, tip, nxt, filled, color):
        d = self._unit(tip, nxt)
        n = (-d[1], d[0])
        L, Wd = 0.17, 0.052
        mid = (tip[0] + d[0] * L / 2, tip[1] + d[1] * L / 2)
        far = (tip[0] + d[0] * L, tip[1] + d[1] * L)
        p1 = (mid[0] + n[0] * Wd, mid[1] + n[1] * Wd)
        p2 = (mid[0] - n[0] * Wd, mid[1] - n[1] * Wd)
        ax.add_patch(Polygon([tip, p1, far, p2], closed=True, fc=("black" if filled else "white"),
                             ec=color, lw=0.8, zorder=3))

    def _draw_cls(self, ax, d, LW):
        b, fs, lh, pad = d["b"], self.fs, self.lh, d["pad"]
        ax.add_patch(Rectangle((b.x, b.y), b.w, b.h, fc=d["fill"], ec=d["ec"], lw=LW, zorder=4))
        y = b.y + (0.05 if d["stereo"] else 0.06)
        if d["stereo"]:
            ax.text(b.cx, y + lh / 2, d["stereo"], fontsize=fs, family=FONT, ha="center", va="center", zorder=5)
            y += lh
        ax.text(b.cx, y + lh / 2 + (0 if d["stereo"] else 0), d["name"], fontsize=fs, family=FONT,
                fontweight="bold", fontstyle="italic" if d["italic"] else "normal", ha="center", va="center",
                zorder=5)
        y = b.y + d["head_h"]
        for (kind, lines_), h in zip(d["comps"], d["heights"]):
            ax.plot([b.x, b.r], [y, y], color=d["ec"], lw=LW, zorder=5)
            ty = y + 0.045
            for ln in lines_:
                ax.text(b.x + pad, ty + lh / 2, ln, fontsize=fs, family=FONT, ha="left", va="center", zorder=5)
                ty += lh
            y += h

    def _draw_note(self, ax, d):
        b = d["b"]
        f = 0.11
        pts = [(b.x, b.y), (b.r - f, b.y), (b.r, b.y + f), (b.r, b.b), (b.x, b.b)]
        ax.add_patch(Polygon(pts, closed=True, fc=d["fill"], ec="#8a7a2a", lw=0.7, zorder=4))
        ax.plot([b.r - f, b.r - f, b.r], [b.y, b.y + f, b.y + f], color="#8a7a2a", lw=0.6, zorder=5)
        y = b.y + 0.06
        for ln in d["lines"]:
            ax.text(b.x + 0.07, y + d["lh"] / 2, ln, fontsize=d["fs"], family=FONT, ha="left", va="center", zorder=5)
            y += d["lh"]


def ortho(p0, s0, p1, s1, mid=None):
    """Conector ortogonal entre dos anclajes. s0 = lado de salida, s1 = lado de llegada."""
    x0, y0 = p0
    x1, y1 = p1
    h0, h1 = s0 in "EW", s1 in "EW"
    if h0 and h1:
        if abs(y0 - y1) < 1e-6:
            return [p0, p1]
        xm = mid if mid is not None else (x0 + x1) / 2
        return [p0, (xm, y0), (xm, y1), p1]
    if (not h0) and (not h1):
        if abs(x0 - x1) < 1e-6:
            return [p0, p1]
        ym = mid if mid is not None else (y0 + y1) / 2
        return [p0, (x0, ym), (x1, ym), p1]
    if h0 and not h1:
        return [p0, (x1, y0), p1]
    return [p0, (x0, y1), p1]
