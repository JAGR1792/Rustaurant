"""Genera los 15 diagramas (PNG a 300 dpi y SVG editable) en ./png y ./svg.
Uso:  python make_all.py            (todos)
      python make_all.py 5 8        (solo las figuras 5 y 8)
"""
import os
import sys

os.makedirs("png", exist_ok=True)
os.makedirs("svg", exist_ok=True)

import fig_abstract as A
import fig_builder as B
import fig_factory as F
import fig_proto as P
import fig_single as S

JOBS = {
    1: F.fig1, 2: F.fig2, 3: F.fig3,
    4: lambda o: A.make(A.SPEC4, "fig4", 6.4, 5.6, o, fs_start=8.0, extra=A.extra4),
    5: lambda o: A.make(A.SPEC5, "fig5", 9.95, 6.2, o, fs_start=8.0),
    6: lambda o: A.make(A.SPEC6, "fig6", 9.95, 6.2, o, fs_start=8.0),
    7: B.fig7, 8: B.fig8, 9: B.fig9,
    10: P.fig10, 11: P.fig11, 12: P.fig12,
    13: S.fig13, 14: S.fig14, 15: S.fig15,
}

if __name__ == "__main__":
    only = [int(a) for a in sys.argv[1:]] or sorted(JOBS)
    for n in only:
        JOBS[n](f"png/fig{n}.png")
        JOBS[n](f"svg/fig{n}.svg")
