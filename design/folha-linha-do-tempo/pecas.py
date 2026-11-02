"""Canvas helpers for montar.py: absolute-positioned layers on one frame, and the small stroke icons of the block's points."""

PAPEL, TINTA, MEDIO, BAIXO = "#0c3a78", "#eaf4ff", "#9cc4ee", "#5b93d0"
CARTAO, AMBAR, FEITO, ANDANDO, GRADE = "#082b5e", "#ffcf6b", "#8be3b5", "#6cb4ff", "#bfe0ff"
MESES = ["JAN", "FEV", "MAR", "ABR", "MAI", "JUN", "JUL", "AGO", "SET", "OUT", "NOV", "DEZ"]
FONTE = "Hack"
PASSOS = []
_n = [0]


def _novo():
    _n[0] += 1
    return f"n{_n[0]}"


def _hex(cor, op=None):
    """#rrggbb with an alpha, the format FigCraft takes."""
    return cor if op is None or op >= 1 else f"{cor}{round(op * 255):02x}"


def _criar(pai, tipo, x, y, w=None, h=None, nome=None, **props):
    i = _novo()
    p = {"type": tipo, "x": x, "y": y, "parent": {"$ref": pai}, "select": False, "props": props}
    if w is not None:
        p["width"], p["height"] = w, h
    if nome:
        p["name"] = nome
    PASSOS.append({"command": "node.create", "as": i, "params": p})
    return i


def tela(nome, w, h, x=0):
    i = _novo()
    PASSOS.append({"command": "frame.create", "as": i, "params": {"x": x, "y": 0, "width": w, "height": h, "name": nome}})
    # the paper's radial (planta/papel.rs): lighter at the middle, darker at the far corners
    fundo = {"type": "radialGradient", "stops": ["#1250a0", PAPEL, "#082b5e"]}
    PASSOS.append({"command": "node.set", "params": {"ids": [{"$ref": i}], "props": {"fills": [fundo], "clip": True}}})
    return i


def ret(pai, x, y, w, h, fill=None, stroke=None, sw=1.0, raio=0, dash=None, op_fill=None, op_stroke=None, nome=None):
    props = {"fill": _hex(fill, op_fill) if fill else "none", "cornerRadius": raio}
    if stroke:
        props.update(stroke=_hex(stroke, op_stroke), strokeWeight=sw)
    if dash:
        props["dashes"] = dash
    return _criar(pai, "rectangle", x, y, w, h, nome, **props)


def fio(pai, x0, y0, x1, y1, cor, sw=1.0, op=None, dash=None):
    props = {"fill": "none", "stroke": _hex(cor, op), "strokeWeight": sw}
    if dash:
        props["dashes"] = dash
    i = _novo()
    PASSOS.append({"command": "node.create", "as": i, "params": {"type": "vector", "parent": {"$ref": pai}, "select": False, "points": [[x0, y0], [x1, y1]], "props": props}})
    return i


def traco(pai, x, y, d, cor, sw=1.2):
    """An SVG path at (x, y), in stroke."""
    i = _novo()
    PASSOS.append({"command": "node.create", "as": i, "params": {"type": "vector", "parent": {"$ref": pai}, "select": False, "x": x, "y": y, "path": d, "props": {"fill": "none", "stroke": cor, "strokeWeight": sw, "strokeCap": "round", "strokeJoin": "round"}}})
    return i


def txt(pai, x, y, s, tam, cor, peso=None, trilha=None, meio=False):
    props = {"fontFamily": FONTE, "fontSize": tam, "fill": cor}
    if peso:
        props["fontStyle"] = peso
    if trilha:
        props["letterSpacing"] = trilha
    if meio:
        x -= len(s) * tam * 0.3
    i = _novo()
    PASSOS.append({"command": "node.create", "as": i, "params": {"type": "text", "parent": {"$ref": pai}, "select": False, "x": x, "y": y, "text": s, "props": props}})
    return i


def bola(pai, x, y, r, cor):
    return _criar(pai, "ellipse", x - r, y - r, r * 2, r * 2, None, fill=cor)


def losango(pai, x, y, r, fundo, cor):
    i = _novo()
    d = f"M{r} 0L{2 * r} {r}L{r} {2 * r}L0 {r}Z"
    PASSOS.append({"command": "node.create", "as": i, "params": {"type": "vector", "parent": {"$ref": pai}, "select": False, "x": x - r, "y": y - r, "path": d, "closed": True, "props": {"fill": fundo, "stroke": cor, "strokeWeight": 1.4}}})
    return i


def icone_plano(pai, x, y, cor):
    """Plan: the little checkbox."""
    return traco(pai, x, y, "M1.5 0H8.5A1.5 1.5 0 0 1 10 1.5V8.5A1.5 1.5 0 0 1 8.5 10H1.5A1.5 1.5 0 0 1 0 8.5V1.5A1.5 1.5 0 0 1 1.5 0Z M2.6 5.2L4.4 7L7.6 3.2", cor)


def icone_detalhe(pai, x, y, cor):
    """Detail: a sheet with a picture."""
    return traco(pai, x, y, "M0 0H10V10H0Z M0 7.2L3 4.4L5.4 6.6L7.2 5L10 7.6", cor)


def icone_decisao(pai, x, y, cor):
    """Decision: a path that forks."""
    return traco(pai, x, y, "M0 5H3.4L8 1 M3.4 5L8 9 M6 1H8.4V3.2 M6 9H8.4V6.8", cor)


def icone_visto(pai, x, y, cor):
    """Approve: the check."""
    return traco(pai, x, y, "M0 5L3.2 8.2L9 1.4", cor, 1.6)


def pasta(pai, x, y, tom):
    """The project's isometric folder: same proportions as folha_figura.rs, drawn flat in stroke."""
    i = _novo()
    d = "M0 8L9 3.5L11 4.5L11 7.5L18 11L9 15.5L0 11Z M0 8L9 12.5L18 8 M0 11L9 15.5L18 11"
    PASSOS.append({"command": "node.create", "as": i, "params": {"type": "vector", "parent": {"$ref": pai}, "select": False, "x": x, "y": y, "path": d, "props": {"fill": _hex(tom, 0.18), "stroke": tom, "strokeWeight": 1.1, "strokeJoin": "round"}}})
    return i
