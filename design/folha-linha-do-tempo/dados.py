"""The sheet's example data, invented. `conteudo` is what the open action shows."""
from pecas import ANDANDO, FEITO, MEDIO

_PAINEL = dict(
    planos=[
        ("Ler as leituras por sala", "pronto", "O agente concluiu e marcou"),
        ("Montar o gráfico do dia", "trabalhando", "Trabalhando · Claude"),
        ("Comparar com a semana anterior", "fila", "Na fila · depende de 1.2"),
        ("Filtrar por andar", "nao", None),
        ("Exportar em CSV", "nao", None),
        ("Avisar sala sem dados", "nao", None),
    ],
    checkpoints=[("Painel no ar", "24/11")],
    decisoes=[
        ("Barras em vez de linhas", ["Quem lê o painel compara salas, não acompanha uma curva."], "aprovada"),
        ("Atualizar a cada 5 minutos", ["O medidor manda a cada minuto, mas ninguém olha", "nesse ritmo e a rede do prédio é fraca."], "pendente"),
        ("Guardar 12 meses de leituras", ["Reduzir para 6 se o disco do servidor não aguentar."], "revisar"),
    ],
    perguntas=[("P1", "Mostrar o custo em reais junto do consumo?", ["Sim", "Não"], 0)],
    tabela=("Salas do primeiro corte", ["Sala", "Medidor", "Leitura"], [["Reunião 1", "M-01", "por minuto"], ["Laboratório", "M-02", "por minuto"], ["Recepção", "M-03", "a cada 5 min"]]),
    detalhes=[("01", "Tela por sala", "aguardando"), ("02", "Comparação entre salas", None)],
    espera=1,
)

_NORMALIZAR = dict(
    diff=[
        ("−", "prazo", [("03/11 → ", False), ("11/11", True)]),
        ("+", "prazo", [("03/11 → ", False), ("13/11", True)]),
        (" ", "1.2.1", [("Converter as unidades para kWh", False)]),
        ("−", "1.2.2", [("Descartar leitura ", False), ("repetida", True)]),
        ("+", "1.2.2", [("Descartar leitura ", False), ("repetida ou zerada", True)]),
        ("+", "1.2.5", [("Registrar o que foi descartado", False)]),
        ("−", "1.2.4", [("Recalcular o histórico", False)]),
    ],
    resumo="Claude · +1 · −1 · 1 alterado · prazo",
    checkpoints=[("Leituras limpas", "13/11")],
    decisoes=[("Zerada conta como falha do medidor", ["Antes era tratada como consumo zero."], "pendente")],
)

PROJETOS = [
    ("Coleta", 42, FEITO, [
        dict(ref="1.1", nome="Ler os medidores", de="29/10", ate="02/11", sit="feita", planos=(4, 4), det=0, dec=1, aprovar=0),
        dict(ref="1.2", nome="Normalizar leituras", de="03/11", ate="13/11", sit="andando", planos=(2, 5), det=0, dec=2, aprovar=3, alterado=True,
             texto="Deixar as leituras no mesmo formato antes de guardar.", conteudo=_NORMALIZAR),
        dict(ref="1.3", nome="Bloquear leitura inválida", de="08/11", ate="13/11", sit="planejada", planos=(0, 3), det=0, dec=0, aprovar=0),
    ]),
    ("Painel", 8, ANDANDO, [
        dict(ref="2.1", nome="Painel por sala", de="10/11", ate="24/11", sit="andando", planos=(1, 6), det=2, dec=3, aprovar=4, alterado=True, agentes=2, marco="24/11",
             texto="Uma tela por sala, com o consumo do dia e a comparação com a semana anterior.", conteudo=_PAINEL),
        dict(ref="2.2", nome="Comparar salas", de="20/11", ate="01/12", sit="planejada", planos=(0, 4), det=1, dec=1, aprovar=0),
    ]),
    ("Alertas", 0, MEDIO, [
        dict(ref="3.1", nome="Alerta de pico", de="28/11", ate="12/12", sit="proposta", planos=(0, 5), det=1, dec=1, aprovar=5),
        dict(ref="3.2", nome="Relatório semanal", de="10/12", ate="22/12", sit="aguardando", planos=(0, 0), det=0, dec=0, aprovar=0),
    ]),
]
