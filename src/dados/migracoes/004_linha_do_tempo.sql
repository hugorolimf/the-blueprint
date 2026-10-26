-- Linha do tempo da execução do agente por ação (2026-10-26; regra em docs/claude-na-folha.md, "Linha do tempo da execução por ação").
-- Fora do data.json de propósito (como `pastas`): o PUT /api/dados não apaga e o GET não cresce. Sem chave estrangeira (o PUT recria as ações).
-- Retenção feita a cada gravação: os 50 mais recentes por ação e nada com mais de 30 dias. Nunca guarda conteúdo de arquivo, comando nem evidência.
CREATE TABLE linha_do_tempo (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id TEXT NOT NULL,
    blueprint_id TEXT NOT NULL,
    acao_id      TEXT NOT NULL,
    alvo         TEXT,            -- id do plano ou do checkpoint, conforme o tipo
    em           TEXT NOT NULL,   -- ISO em UTC, com milissegundos
    agente       TEXT,
    tipo         TEXT NOT NULL,
    estado       TEXT,
    texto        TEXT NOT NULL DEFAULT '',
    onde         TEXT
);
CREATE INDEX linha_do_tempo_acao ON linha_do_tempo (workspace_id, blueprint_id, acao_id, id);
CREATE INDEX linha_do_tempo_em ON linha_do_tempo (em);
