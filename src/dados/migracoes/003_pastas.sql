-- Vínculo pasta → blueprint (2026-10-22): o hook de sessão do Claude só sabe a pasta (cwd); o app guarda de qual blueprint ela é, no lugar da linha `Blueprint:` do CLAUDE.md.
-- Fica fora do data.json de propósito (PUT /api/dados não apaga) e sem chave estrangeira (o PUT recria os blueprints); o vínculo é validado ao criar.
CREATE TABLE pastas (
    caminho      TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    blueprint_id TEXT NOT NULL,
    criado_em    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
