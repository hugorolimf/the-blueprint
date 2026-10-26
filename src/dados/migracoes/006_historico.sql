-- Histórico de campos (frente SINCRONIA, docs/colaboracao.md seção 2): cada campo mudado por POST /api/mudancas, com o valor de antes e o de
-- depois, quem mudou e a versão dos dados criada pelo lote. Responde também "quem mudou este campo por último" para o aviso de conflito.
-- Fora da árvore (sem chave estrangeira): o PUT /api/dados apaga e recria a árvore e o histórico fica. O PUT não grava histórico.
-- Retenção a cada gravação: 100 linhas por item e nada com mais de 180 dias.
CREATE TABLE historico_campos (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    versao       INTEGER NOT NULL,
    em           TEXT NOT NULL,
    membro_id    TEXT NOT NULL,
    nome         TEXT NOT NULL DEFAULT '',
    workspace_id TEXT NOT NULL,
    blueprint_id TEXT NOT NULL DEFAULT '',
    tipo         TEXT NOT NULL,
    item_id      TEXT NOT NULL,
    campo        TEXT NOT NULL,
    operacao     TEXT NOT NULL,
    antes        TEXT,
    depois       TEXT
);
CREATE INDEX historico_item ON historico_campos (tipo, item_id, campo, id);
CREATE INDEX historico_em ON historico_campos (em);
