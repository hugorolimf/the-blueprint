"""The open action, as the original sheet draws it: on the paper, no card, with the dimension line and the underlined columns.
The new functions (plan states, decisions, questions, details, tables, the diff) come in as more columns in that same style."""
from escala import X0, X1, ESCALA, dt, x_de
from pecas import AMBAR, ANDANDO, BAIXO, CARTAO, FEITO, MEDIO, PAPEL, TINTA, bola, fio, icone_visto, losango, ret, traco, txt

VERDE, VERM, CAMPO = FEITO, "#ff8f80", "#081d3c"
COL_PLANOS, COL_CHECK, COL_DIREITA = (X0, 370), (X0 + 400, 290), (X0 + 720, X1 - X0 - 720)


def rotulo(t, x, y, s, w):
    """The original's section heading: small caps and a line under it."""
    txt(t, x, y, s.upper(), 9, MEDIO, trilha="12%")
    fio(t, x, y + 15, x + w, y + 15, BAIXO)


def botao(t, x, y, s, primario=False):
    w = len(s) * 6.6 + 18
    ret(t, x, y, w, 22, fill=TINTA if primario else None, stroke=TINTA)
    txt(t, x + 9, y + 5, s.upper(), 10, PAPEL if primario else TINTA, trilha="6%")
    return w


def controle(t, xfim, y, meio="Revisar"):
    """Approve, Revise (or Edit) and Reject, one square set. Returns where it starts."""
    partes = [("APROVAR", VERDE), (meio.upper(), AMBAR), ("REPROVAR", VERM)]
    total = sum(len(s) * 6.2 + 16 for s, _ in partes)
    x = xfim - total
    ini = x
    ret(t, x, y, total, 22, fill=CAMPO, stroke=MEDIO)
    for i, (s, cor) in enumerate(partes):
        if i:
            fio(t, x, y, x, y + 22, MEDIO)
        txt(t, x + 8, y + 5, s, 9.5, cor if i == 0 else TINTA, trilha="4%")
        x += len(s) * 6.2 + 16
    return ini


def caixa(t, x, y, estado):
    ret(t, x, y, 13, 13, fill=TINTA if estado == "pronto" else PAPEL, stroke=TINTA, sw=1.3)
    if estado == "pronto":
        traco(t, x + 2.5, y + 3, "M0 3.5L3 6.5L8 0.5", PAPEL, 2)
    elif estado == "trabalhando":
        ret(t, x + 3.5, y + 3.5, 6, 6, fill="#bfe0ff")
    elif estado == "fila":
        bola(t, x + 6.5, y + 6.5, 2, AMBAR)


def nota(t, x, y, cor=TINTA):
    traco(t, x, y, "M0 0H9L14 5V16H0Z M9 0V5H14 M3 9H11 M3 12.5H11", cor)


def cabeca(t, a, y, x_recolher=X1):
    """The name line and the dimension line. Returns the y where they end."""
    xa, xb = x_de(dt(a["de"])), x_de(dt(a["ate"])) + ESCALA
    rot = f"{a['ref']} · {a['nome'].upper()}"
    txt(t, X0, y, rot, 11, TINTA, trilha="6%")
    nota(t, X0 + len(rot) * 7 + 12, y - 2)
    # no bar: the dimension line alone says where the action falls in time
    cy = y + 36
    if a.get("agentes"):
        bola(t, xb + 18, cy, 3, ANDANDO)
        txt(t, xb + 25, cy - 6, f"×{a['agentes']} trabalhando", 10, ANDANDO)
    for x in (xa, xb):
        fio(t, x, cy - 6, x, cy + 6, MEDIO)
        fio(t, x - 4, cy + 4, x + 4, cy - 4, MEDIO)
    fio(t, xa, cy, xb, cy, MEDIO)
    if a.get("marco"):
        losango(t, x_de(dt(a["marco"])) + ESCALA / 2, cy, 7, PAPEL, TINTA)
    dias = (dt(a["ate"]) - dt(a["de"])).days + 1
    s = f"{dias} DIAS"
    txt(t, (xa + xb) / 2, cy - 15, s, 10, MEDIO, meio=True)
    txt(t, xa, cy + 8, a["de"], 10, MEDIO)
    txt(t, xb - 30, cy + 8, a["ate"], 10, MEDIO)
    w_rec = len("− RECOLHER") * 6.6 + 18
    botao(t, x_recolher - w_rec, y - 4, "− Recolher")
    return cy + 26


def texto_e_links(t, a, y):
    if a.get("texto"):
        txt(t, X0, y, a["texto"], 11.5, MEDIO)
        y += 22
    return y


def planos(t, lista, feitos, x, y, w):
    rotulo(t, x, y, f"Planos · {feitos}/{len(lista)}", w)
    y += 26
    for s, estado, sob in lista:
        caixa(t, x, y, estado)
        txt(t, x + 22, y, s, 11, MEDIO if estado == "pronto" else TINTA)
        y += 19
        if sob:
            cor = {"pronto": VERDE, "trabalhando": ANDANDO, "fila": AMBAR, "bloqueado": AMBAR}.get(estado, MEDIO)
            bola(t, x + 26, y + 6, 3, cor)
            txt(t, x + 34, y, sob, 9.5, cor)
            y += 17
        y += 5
    livres = [p for p in lista if p[1] == "nao"]
    if len(livres) > 1:
        y += 2
        botao(t, x, y, f"Iniciar os {len(livres)}")
        y += 30
    return y


def checkpoints(t, lista, x, y, w):
    rotulo(t, x, y, f"Checkpoints · 0/{len(lista)}", w)
    y += 26
    for nome, data in lista:
        losango(t, x + 6.5, y + 6, 6.5, PAPEL, TINTA)
        txt(t, x + 22, y, f"{nome} · {data}", 11, TINTA)
        y += 24
    return y


def decisoes(t, lista, x, y, w):
    rotulo(t, x, y, f"Decisões · {len(lista)}", w)
    y += 26
    for titulo, corpo, estado in lista:
        txt(t, x, y + 3, titulo, 11.5, TINTA, peso="Bold")
        if estado == "pendente":
            controle(t, x + w, y)
            y += 24
        else:
            cor = VERDE if estado == "aprovada" else AMBAR
            s = estado.upper()
            txt(t, x + w - len(s) * 6.3, y + 3, s, 9.5, cor, trilha="6%")
            y += 20
        for linha in corpo:
            txt(t, x, y, linha, 11, MEDIO)
            y += 15
        y += 12
    return y


def perguntas(t, lista, x, y, w):
    rotulo(t, x, y, f"Perguntas · {len(lista)}", w)
    y += 26
    for cod, s, opcoes, recomendada in lista:
        txt(t, x, y, cod, 10.5, MEDIO, peso="Bold")
        txt(t, x + 26, y - 1, s, 11.5, TINTA)
        y += 20
        bx = x + 26
        for i, o in enumerate(opcoes + ["Outra…"]):
            lw = botao(t, bx, y, o)
            if i == recomendada:
                txt(t, bx + lw / 2, y + 26, "recomendado", 9, MEDIO, meio=True)
            bx += lw + 6
        y += 46
    return y


def tabela(t, titulo, colunas, linhas, x, y, w):
    rotulo(t, x, y, titulo, w)
    y += 24
    cw = w / len(colunas)
    for i, c in enumerate(colunas):
        txt(t, x + i * cw, y, c.upper(), 9.5, MEDIO, trilha="10%")
    y += 17
    for l in linhas:
        fio(t, x, y, x + w, y, BAIXO, op=0.5)
        for i, v in enumerate(l):
            txt(t, x + i * cw, y + 6, v, 11, TINTA)
        y += 24
    return y + 8


def figura_tira(t, x, y, w, h, tipo):
    """A sketch standing for the action's figure."""
    if tipo == 0:
        for i, alto in enumerate((22, 38, 30, 52, 44, 60, 34, 48)):
            ret(t, x + 16 + i * ((w - 32) / 8), y + h - 10 - alto, (w - 32) / 8 - 8, alto, stroke=TINTA, sw=1)
        fio(t, x + 10, y + h - 10, x + w - 10, y + h - 10, MEDIO)
    else:
        for i in range(3):
            for j in range(2):
                ret(t, x + 14 + i * ((w - 28) / 3), y + 12 + j * 44, (w - 28) / 3 - 8, 36, stroke=TINTA, sw=1)


def detalhes(t, a, lista, espera, x, y, w):
    s = f"Detalhes · {len(lista)}" + (f" · {espera} aguardando você" if espera else "")
    rotulo(t, x, y, s, w)
    y += 26
    u = (w - 36) / 4
    for i, (cod, titulo, estado) in enumerate(lista):
        xx = x + i * (u + 12)
        ret(t, xx, y, u, 118, fill=CAMPO, stroke=BAIXO)
        figura_tira(t, xx, y, u, 118, i % 2)
        ret(t, xx, y + 118, u, 40, fill=CARTAO, stroke=BAIXO)
        if estado:
            ret(t, xx, y, 3, 158, fill=AMBAR)
        txt(t, xx + 10, y + 123, f"{a['ref']}-{cod}", 9.5, MEDIO)
        if estado:
            txt(t, xx + u - 8 - len(estado) * 6, y + 123, estado.upper(), 9.5, AMBAR)
        txt(t, xx + 10, y + 138, titulo, 10.5, TINTA)
    return y + 170


def diff(t, a, x, y, w, linhas, resumo):
    txt(t, x, y + 3, "Plano alterado", 12, TINTA, peso="Bold")
    txt(t, x + 118, y + 5, resumo, 10, MEDIO)
    controle(t, x + w, y, "Editar")
    y += 26
    alto = 21
    ret(t, x, y, w, alto * len(linhas) + 1, fill=CAMPO, stroke=BAIXO)
    y += 1
    for sinal, ident, pedacos in linhas:
        cor = {"+": VERDE, "−": VERM}.get(sinal, MEDIO)
        if sinal != " ":
            ret(t, x, y, w, alto, fill=cor, op_fill=0.13)
            ret(t, x, y, 3, alto, fill=cor)
        txt(t, x + 12, y + 4, sinal, 11, cor, peso="Bold")
        txt(t, x + 30, y + 5, ident, 10, MEDIO)
        xx = x + 78
        for s, forte in pedacos:
            l = len(s) * 6.6
            if forte:
                ret(t, xx - 1, y + 3, l + 2, alto - 6, fill=cor, op_fill=0.32)
            txt(t, xx, y + 4, s, 11, TINTA)
            xx += l
        y += alto
    y += 8
    txt(t, x, y, "Aprovar vale a versão nova; reprovar volta ao estado original. O que mudou só pode ser iniciado depois disso.", 10, MEDIO)
    return y + 26


def desenhar(t, a, y):
    """Draws the open action from `y`. Returns the y where it ends, with the closing line."""
    c = a["conteudo"]
    yy = texto_e_links(t, a, cabeca(t, a, y))
    if c.get("diff"):
        yy = diff(t, a, X0, yy, X1 - X0, c["diff"], c["resumo"])
    ini = yy
    x, w = COL_PLANOS
    y1 = planos(t, c["planos"], a["planos"][0], x, ini, w) if c.get("planos") else ini
    x2, w2 = COL_CHECK if c.get("planos") else COL_PLANOS
    y2 = checkpoints(t, c["checkpoints"], x2, ini, w2) if c.get("checkpoints") else ini
    x3, w3 = COL_DIREITA
    y3 = decisoes(t, c["decisoes"], x3, ini, w3) if c.get("decisoes") else ini
    if c.get("perguntas"):
        y3 = perguntas(t, c["perguntas"], x3, y3 + 4, w3)
    esq = max(y1, y2)
    if c.get("tabela"):
        tit, cols, lin = c["tabela"]
        esq = tabela(t, tit, cols, lin, X0, esq + 6, 690)
    yy = max(esq, y3) + 6
    if c.get("detalhes"):
        yy = detalhes(t, a, c["detalhes"], c.get("espera", 0), X0, yy, X1 - X0)
    fio(t, X0, yy + 2, X1, yy + 2, TINTA, op=0.5)
    return yy + 12
