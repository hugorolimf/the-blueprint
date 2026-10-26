-- Pessoas, convites e sessões do modo hospedado (docs/colaboracao.md, seção 2). Fora do data.json, como `pastas`: sem chave estrangeira
-- para a árvore, que o PUT /api/dados apaga e recria. Segredos nunca são gravados: só o sha256 em hexadecimal minúsculo.
CREATE TABLE membros (
    id          TEXT PRIMARY KEY,
    nome        TEXT NOT NULL,
    criado_em   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    removido_em TEXT
);
CREATE TABLE acessos (
    membro_id    TEXT NOT NULL REFERENCES membros (id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL,
    blueprint_id TEXT NOT NULL DEFAULT '',
    papel        TEXT NOT NULL CHECK (papel IN ('dono', 'colaborador', 'supervisor')),
    convite_id   TEXT,
    criado_em    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (membro_id, workspace_id, blueprint_id)
);
CREATE TABLE convites (
    id           TEXT PRIMARY KEY,
    hash         TEXT NOT NULL UNIQUE,
    papel        TEXT NOT NULL CHECK (papel IN ('dono', 'colaborador', 'supervisor', 'visitante')),
    workspace_id TEXT NOT NULL,
    blueprint_id TEXT NOT NULL DEFAULT '',
    abre         TEXT NOT NULL DEFAULT 'folha' CHECK (abre IN ('folha', 'apresentacao')),
    criado_por   TEXT,
    criado_em    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    expira_em    TEXT NOT NULL,
    usos_max     INTEGER,
    usos         INTEGER NOT NULL DEFAULT 0,
    revogado_em  TEXT
);
CREATE TABLE sessoes (
    hash       TEXT PRIMARY KEY,
    membro_id  TEXT REFERENCES membros (id) ON DELETE CASCADE,
    convite_id TEXT,
    criada_em  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    expira_em  TEXT NOT NULL,
    visto_em   TEXT NOT NULL,
    CHECK (membro_id IS NOT NULL OR convite_id IS NOT NULL)
);
CREATE INDEX sessoes_membro ON sessoes (membro_id);
CREATE TABLE chaves_mcp (
    hash      TEXT PRIMARY KEY,
    membro_id TEXT NOT NULL UNIQUE REFERENCES membros (id) ON DELETE CASCADE,
    prefixo   TEXT NOT NULL,
    criada_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    usada_em  TEXT
);
CREATE TABLE atividade (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    em           TEXT NOT NULL,
    membro_id    TEXT,
    nome         TEXT NOT NULL DEFAULT '',
    tipo         TEXT NOT NULL,
    workspace_id TEXT NOT NULL DEFAULT '',
    blueprint_id TEXT NOT NULL DEFAULT '',
    alvo         TEXT NOT NULL DEFAULT ''
);
CREATE INDEX atividade_ws ON atividade (workspace_id, id);
CREATE INDEX atividade_em ON atividade (em);
