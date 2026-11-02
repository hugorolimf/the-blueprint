"""The sheet's measures and the timeline's scale, shared by montar.py and acao_aberta.py."""
from datetime import date

W = 1366
X0, X1 = 52, W - 52
HOJE = date(2026, 11, 2)
FIM = date(2026, 12, 24)
INI = (FIM - HOJE).days * 0.08
ESCALA = (X1 - X0) / ((FIM - HOJE).days + INI)


def x_de(d):
    return X0 + ((d - HOJE).days + INI) * ESCALA


def dt(s):
    d, m = s.split("/")
    return date(2026, int(m), int(d))
