#!/usr/bin/env python3
"""The sheet without the table: the timeline takes the whole width and the numbers go inside each action block.

Run: python3 design/folha-linha-do-tempo/montar.py [destination.figcraft]
Example data is invented. Colors are the app's theme (ui/src/tema.rs, folha_estilo.rs).
"""
import json
import os
import subprocess
import sys
import tempfile
from datetime import date

import acao_aberta
from dados import PROJETOS
from escala import ESCALA, FIM, HOJE, W, X0, X1, dt, x_de
from pecas import *  # noqa: F403  the canvas helpers and the theme

Y_REGUA = 150
ALTO_BLOCO, LINHA, CABECA = 44, 54, 34
RAIO = 6


def moldura(t, h):
    ret(t, 7, 7, W - 14, h - 14, stroke=BAIXO)
    ret(t, 26, 26, W - 52, h - 52, stroke=TINTA, sw=1.6)
    for i, x in enumerate((W * 0.25, W * 0.75)):
        txt(t, x, 13, str(i + 1), 9, MEDIO, meio=True)
    fio(t, W / 2, 7, W / 2, 26, BAIXO)
    # the letters down the sides, as in a drawing sheet
    n = max(1, round((h - 52) / 150))
    passo = (h - 52) / n
    for i in range(n):
        y = 26 + (i + 0.5) * passo
        for x in (16.5, W - 16.5):
            txt(t, x, y - 3, chr(65 + i % 26), 9, MEDIO, meio=True)
        if i:
            yl = 26 + i * passo
            fio(t, 7, yl, 26, yl, BAIXO)
            fio(t, W - 26, yl, W - 7, yl, BAIXO)


def grade(t, H):
    # the paper's grids of 12 and 60 px, same alphas as planta/papel.rs
    for x in range(0, W, 12):
        fio(t, x, 0, x, H, "#ffffff", op=0x1c / 255 if x % 60 == 0 else 0x0c / 255)
    for y in range(0, H, 12):
        fio(t, 0, y, W, y, "#ffffff", op=0x1c / 255 if y % 60 == 0 else 0x0c / 255)


def topo(t):
    txt(t, X0, 50, "Monitor de consumo", 25, TINTA, peso="Bold")
    txt(t, X0, 88, "MEU WORKSPACE · FOLHA 2 DE 3", 10.5, MEDIO, trilha="14%")
    x = X1
    for rotulo, estilo in (("Compartilhar", "c"), ("Planejar", "p"), ("Agentes · 2", "c"), ("O que chegou", "c"), ("Aguardando você · 12", "a")):
        w = len(rotulo) * 6.6 + 22
        x -= w
        cor = AMBAR if estilo == "a" else TINTA
        ret(t, x, 48, w, 26, fill=(TINTA if estilo == "p" else None), stroke=cor, raio=5, op_fill=1)
        txt(t, x + 11, 54, rotulo, 11, PAPEL if estilo == "p" else cor, peso="Bold" if estilo in "pa" else None)
        x -= 8


def regua(t, y_fim, vaos=()):
    fio(t, X0, Y_REGUA, X1, Y_REGUA, TINTA, op=0.9)
    d = date.fromordinal(HOJE.toordinal() - 4)
    while d <= FIM:
        x = x_de(d)
        if d.day == 1:
            fio(t, x, Y_REGUA - 9, x, Y_REGUA, TINTA)
            fio(t, x, Y_REGUA, x, y_fim, TINTA, op=0.14)
            txt(t, x + 5, Y_REGUA - 22, MESES[d.month - 1], 10, TINTA, peso="Bold", trilha="12%")
        elif d.weekday() == 0:
            fio(t, x, Y_REGUA - 4, x, Y_REGUA, MEDIO)
            fio(t, x, Y_REGUA, x, y_fim, TINTA, op=0.07)
            txt(t, x, Y_REGUA + 4, f"{d.day:02d}", 8.5, MEDIO, meio=True)
        d = date.fromordinal(d.toordinal() + 1)
    xh = x_de(HOJE)
    # the today line doesn't cross an open action: it would run through the text
    y = Y_REGUA
    for a, b in vaos:
        fio(t, xh, y, xh, a, TINTA, op=0.75, dash=[10, 2, 4, 4])
        y = b
    fio(t, xh, y, xh, y_fim, TINTA, op=0.75, dash=[10, 2, 4, 4])
    ret(t, xh - 34, Y_REGUA - 28, 68, 17, fill=TINTA, raio=5)
    txt(t, xh, Y_REGUA - 26, "HOJE 02/11", 9.5, PAPEL, peso="Bold", meio=True)


def cabecalho_projeto(t, nome, pct, tom, acoes, y):
    pasta(t, X0 + 2, y + 3, tom)
    txt(t, X0 + 26, y + 2, nome, 14, TINTA, peso="Bold")
    if pct:
        xp = X0 + 26 + len(nome) * 8.5 + 8
        ret(t, xp, y + 3, 34, 16, fill=tom, op_fill=0.16, raio=6)
        txt(t, xp + 17, y + 5, f"{pct}%", 9.5, tom, peso="Bold", meio=True)
    xa, xb = x_de(dt(acoes[0]["de"])), x_de(dt(max((a["ate"] for a in acoes), key=lambda s: dt(s))))
    # the project's period, without progress
    fio(t, xa, y + 26, xb, y + 26, TINTA, op=0.5)
    for x in (xa, xb):
        fio(t, x, y + 22, x, y + 30, TINTA, op=0.5)


def pontos(t, a, x, y, cor, largura_max=None):
    """The row of points. Devolve a largura. A zero count simply isn't drawn."""
    ini = x
    feitos, total = a["planos"]
    if total:
        icone_plano(t, x, y + 2, cor)
        x += 13
        s = f"{feitos}/{total}"
        txt(t, x, y, s, 10, cor, peso="Bold")
        x += len(s) * 6.1 + 9
    if a["det"]:
        icone_detalhe(t, x, y + 2, cor)
        x += 13
        txt(t, x, y, str(a["det"]), 10, cor, peso="Bold")
        x += 6.1 + 9
    if a["dec"]:
        icone_decisao(t, x, y + 2, cor)
        x += 13
        txt(t, x, y, str(a["dec"]), 10, cor, peso="Bold")
        x += 6.1 + 9
    if a["aprovar"]:
        s = str(a["aprovar"])
        w = 28
        ret(t, x, y - 2, w, 16, fill=AMBAR, op_fill=0.18, stroke=AMBAR, op_stroke=0.65, raio=5)
        icone_visto(t, x + 4, y + 2, AMBAR)
        txt(t, x + 15, y, s, 10, AMBAR, peso="Bold")
        x += w + 9
    return x - ini


def largura_conteudo(a):
    nome = 8 + len(a["ref"]) * 6.6 + 8 + len(a["nome"]) * 7.3 + 16 + (14 if a.get("alterado") else 0)
    p, d, k = a["planos"][1], a["det"], a["dec"]
    linha2 = 8 + (13 + len(f"{a['planos'][0]}/{p}") * 6.1 + 9 if p else 0) + (28 if d else 0) + (28 if k else 0) + (37 if a["aprovar"] else 0)
    return max(nome, linha2)


def bloco(t, a, y):
    xa, xb = x_de(dt(a["de"])), x_de(dt(a["ate"]) ) + ESCALA
    xa = max(xa, X0)
    w, sit = xb - xa, a["sit"]
    topo_b = y + (LINHA - ALTO_BLOCO) / 2
    cor, sub = TINTA, MEDIO
    if sit == "feita":
        ret(t, xa, topo_b, w, ALTO_BLOCO, fill=FEITO, op_fill=0.9, stroke=FEITO, raio=RAIO, nome=f"Bloco {a['ref']}")
        cor, sub = PAPEL, PAPEL
    elif sit == "andando":
        ret(t, xa, topo_b, w, ALTO_BLOCO, fill=CARTAO, op_fill=0.95, stroke=ANDANDO, sw=1.2, raio=RAIO, nome=f"Bloco {a['ref']}")
    elif sit == "planejada":
        ret(t, xa, topo_b, w, ALTO_BLOCO, fill=CARTAO, op_fill=0.6, stroke=MEDIO, sw=1.2, raio=RAIO, nome=f"Bloco {a['ref']}")
    elif sit == "proposta":
        ret(t, xa, topo_b, w, ALTO_BLOCO, fill=TINTA, op_fill=0.06, stroke=TINTA, sw=1.2, raio=RAIO, dash=[4, 3], nome=f"Bloco {a['ref']}")
    else:
        ret(t, xa, topo_b, w, ALTO_BLOCO, fill=AMBAR, op_fill=0.08, stroke=AMBAR, sw=1.2, raio=RAIO, dash=[4, 3], nome=f"Bloco {a['ref']}")

    # doesn't fit: the writing leaves the block and hangs to its right, on the paper
    cabe = w >= largura_conteudo(a)
    xt = xa + 9 if cabe else xb + 8
    if not cabe:
        cor, sub = TINTA, MEDIO
    y1, y2 = topo_b + 7, topo_b + 24
    txt(t, xt, y1, a["ref"], 11, sub)
    xn = xt + len(a["ref"]) * 6.7 + 8
    txt(t, xn, y1 - 1, a["nome"], 12.5, cor, peso="Bold")
    if a.get("alterado"):
        xd = xn + len(a["nome"]) * 7.55 + 7
        bola(t, xd + 4, y1 + 7, 4, AMBAR)
    if sit == "aguardando":
        txt(t, xt, y2, "Aguardando plano", 10, AMBAR)
    else:
        pontos(t, a, xt, y2, cor)

    if a.get("agentes"):
        bola(t, xb - 44, topo_b + 13, 3, ANDANDO)
        txt(t, xb - 37, topo_b + 7, f"×{a['agentes']}", 10, ANDANDO, peso="Bold")
    if a.get("marco"):
        losango(t, x_de(dt(a["marco"])) + ESCALA, topo_b + ALTO_BLOCO / 2, 6, PAPEL, TINTA)
    if sit == "feita" and cabe:
        icone_visto(t, xb - 18, topo_b + 16, PAPEL)
    # the dates: right of the block; if that side is full or there's no room, left
    rot = f"{a['de']} → {a['ate']}"
    largura_rot = len(rot) * 6
    if cabe and xb + 10 + largura_rot < X1:
        txt(t, xb + 10, topo_b + 16, rot, 10, MEDIO)
    elif cabe:
        txt(t, xa - 8 - largura_rot, topo_b + 16, rot, 10, MEDIO)
    else:
        txt(t, xt + max(len(a["ref"]) * 6.7 + 8 + len(a["nome"]) * 7.55, 90) + 26, topo_b + 16, rot, 10, MEDIO)


def legenda(t, H):
    y = H - 84
    x = X0
    for rot, f in (("concluída", 0), ("em andamento", 1), ("planejada", 2), ("aguardando", 3), ("proposta", 4)):
        if f == 0:
            ret(t, x, y, 22, 12, fill=FEITO, op_fill=0.9, stroke=FEITO, raio=4)
        elif f == 1:
            ret(t, x, y, 22, 12, fill=CARTAO, stroke=ANDANDO, raio=4)
        elif f == 2:
            ret(t, x, y, 22, 12, fill=CARTAO, op_fill=0.6, stroke=MEDIO, raio=4)
        elif f == 3:
            ret(t, x, y, 22, 12, stroke=AMBAR, raio=4, dash=[4, 3])
        else:
            ret(t, x, y, 22, 12, stroke=TINTA, raio=4, dash=[4, 3])
        txt(t, x + 28, y, rot, 10, MEDIO)
        x += 28 + len(rot) * 6 + 16
    y += 24
    x = X0
    for rot, f in (("planos", icone_plano), ("detalhes", icone_detalhe), ("decisões", icone_decisao), ("aprovar", icone_visto), ("alterado", None), ("checkpoint", "m")):
        if f is None:
            bola(t, x + 5, y + 6, 4, AMBAR)
        elif f == "m":
            losango(t, x + 5, y + 6, 5, PAPEL, TINTA)
        else:
            f(t, x, y + 1, AMBAR if rot == "aprovar" else TINTA)
        txt(t, x + 15, y, rot, 10, MEDIO)
        x += 15 + len(rot) * 6 + 16


def selo(t, H):
    celulas = [("WORKSPACE", "Meu workspace", 140), ("BLUEPRINT", "Monitor de consumo", 176), ("SITUAÇÃO", "Em andamento", 124), ("PROGRESSO", "26%", 90), ("FOLHA", "2 de 3", 72)]
    w = sum(c[2] for c in celulas)
    x, y = X1 - w, H - 94
    ret(t, x, y, w, 60, fill=CARTAO, op_fill=0.96, stroke=TINTA)
    for i, (r, v, l) in enumerate(celulas):
        if i:
            fio(t, x, y + 8, x, y + 52, MEDIO, op=0.5)
        txt(t, x + 12, y + 10, r, 9, MEDIO, trilha="14%")
        txt(t, x + 12, y + 28, v, 12.5, TINTA, peso="Bold")
        x += l


def folha(nome, x, h, aberta=None, raio=6):
    """One sheet. `aberta` is the ref of the action drawn open; the others stay as blocks."""
    global RAIO
    RAIO = raio
    t = tela(nome, W, h, x)
    grade(t, h)
    moldura(t, h)
    topo(t)
    y_fim = h - 104

    def percorrer(y, com_blocos):
        vaos = []
        for nome_p, pct, tom, acoes in PROJETOS:
            if com_blocos:
                cabecalho_projeto(t, nome_p, pct, tom, acoes, y)
            y += CABECA
            for a in acoes:
                if a["ref"] == aberta:
                    fim = acao_aberta.desenhar(t, a, y + 4)
                    vaos.append((y + 2, fim))
                    y = fim
                else:
                    if com_blocos:
                        bloco(t, a, y)
                    y += LINHA
        return y, vaos

    # measure first (the ruler goes under everything and needs the gaps), then draw
    n = len(PASSOS)
    fim, vaos = percorrer(Y_REGUA + 18, False)
    del PASSOS[n:]
    regua(t, y_fim, vaos)
    percorrer(Y_REGUA + 18, True)
    legenda(t, h)
    selo(t, h)
    print(f"{nome}: fim {fim:.0f}, limite {y_fim}")


def montar(destino):
    folha("Folha fechada", 0, 768)
    folha("Ação aberta", 1466, 1430, "2.1", 0)
    folha("Ação aberta · plano alterado", 2932, 1140, "1.2", 0)
    if os.path.exists(destino):
        raise SystemExit(f"{destino} já existe: confira se foi alterado à mão.")
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(PASSOS, f, ensure_ascii=False)
    fc = os.path.expanduser("~/.claude/skills/figcraft/scripts/fc")
    r = subprocess.run([fc, "run", f.name, "--out", destino], capture_output=True, text=True)
    os.unlink(f.name)
    if r.returncode:
        raise SystemExit((r.stdout + r.stderr)[-1500:])


if __name__ == "__main__":
    montar(sys.argv[1] if len(sys.argv) > 1 else "design/folha-linha-do-tempo/folha-v3.figcraft")
