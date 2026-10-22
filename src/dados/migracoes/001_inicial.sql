-- Esquema inicial: a árvore workspace > blueprint > ação > (plano | checkpoint | protótipo), mais os comentários do agente em cada plano.
-- Toda linha tem id estável (o do data.json), ordem entre irmãos, revisao e atualizado_em. `extra` guarda em JSON tudo o que não tem coluna,
-- e `listas` diz quais listas filhas existiam no JSON original (para a exportação ser igual ao original). Sem tabelas de usuário ainda.

CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    ordem INTEGER NOT NULL,
    nome TEXT,
    descricao TEXT,
    listas TEXT NOT NULL DEFAULT '[]',
    extra TEXT NOT NULL DEFAULT '{}',
    revisao INTEGER NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE blueprints (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    ordem INTEGER NOT NULL,
    nome TEXT,
    descricao TEXT,
    obs TEXT,
    listas TEXT NOT NULL DEFAULT '[]',
    extra TEXT NOT NULL DEFAULT '{}',
    revisao INTEGER NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX blueprints_pai ON blueprints (workspace_id, ordem);

CREATE TABLE acoes (
    id TEXT PRIMARY KEY,
    blueprint_id TEXT NOT NULL REFERENCES blueprints (id) ON DELETE CASCADE,
    ordem INTEGER NOT NULL,
    nome TEXT,
    inicio TEXT,
    fim TEXT,
    obs TEXT,
    status TEXT,
    progresso NUMERIC,
    listas TEXT NOT NULL DEFAULT '[]',
    extra TEXT NOT NULL DEFAULT '{}',
    revisao INTEGER NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX acoes_pai ON acoes (blueprint_id, ordem);

CREATE TABLE planos (
    id TEXT PRIMARY KEY,
    acao_id TEXT NOT NULL REFERENCES acoes (id) ON DELETE CASCADE,
    ordem INTEGER NOT NULL,
    texto TEXT,
    descricao TEXT,
    feito INTEGER,
    agente TEXT,
    listas TEXT NOT NULL DEFAULT '[]',
    extra TEXT NOT NULL DEFAULT '{}',
    revisao INTEGER NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX planos_pai ON planos (acao_id, ordem);

CREATE TABLE checkpoints (
    id TEXT PRIMARY KEY,
    acao_id TEXT NOT NULL REFERENCES acoes (id) ON DELETE CASCADE,
    ordem INTEGER NOT NULL,
    nome TEXT,
    data TEXT,
    evidencia TEXT,
    feito_em TEXT,
    feito INTEGER,
    listas TEXT NOT NULL DEFAULT '[]',
    extra TEXT NOT NULL DEFAULT '{}',
    revisao INTEGER NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX checkpoints_pai ON checkpoints (acao_id, ordem);

CREATE TABLE comentarios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plano_id TEXT NOT NULL REFERENCES planos (id) ON DELETE CASCADE,
    ordem INTEGER NOT NULL,
    texto TEXT,
    em TEXT,
    tipo TEXT,
    sessao_id TEXT,
    listas TEXT NOT NULL DEFAULT '[]',
    extra TEXT NOT NULL DEFAULT '{}',
    revisao INTEGER NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX comentarios_pai ON comentarios (plano_id, ordem);

CREATE TABLE prototipos (
    id TEXT PRIMARY KEY,
    acao_id TEXT NOT NULL REFERENCES acoes (id) ON DELETE CASCADE,
    ordem INTEGER NOT NULL,
    titulo TEXT,
    em TEXT,
    listas TEXT NOT NULL DEFAULT '[]',
    extra TEXT NOT NULL DEFAULT '{}',
    revisao INTEGER NOT NULL DEFAULT 1,
    atualizado_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX prototipos_pai ON prototipos (acao_id, ordem);
