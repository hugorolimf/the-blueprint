-- A fila do MCP (propostas, registros e marcações que a página ainda não aplicou) passa a sobreviver ao reinício do app (2026-10-27, ação «Dados à prova de perda»).
-- Fora do data.json (como `linha_do_tempo`): o PUT /api/dados não apaga e o GET não cresce. A memória continua sendo a fonte das leituras; isto é só a cópia durável.
-- `item` é o item da fila em JSON, como a rota /api/mcp/fila o devolve; `entrada` é o que o agente mandou na proposta (para `ajustar`).
CREATE TABLE fila_mcp (
    id      INTEGER PRIMARY KEY,
    item    TEXT NOT NULL,
    entrada TEXT
);
-- Os contadores que nunca voltam atrás: tipo 'id' (o último id de item; alvo vazio), 'proposta' e 'registro' (o último número de cada alvo).
CREATE TABLE fila_mcp_contadores (
    tipo         TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    blueprint_id TEXT NOT NULL,
    ultimo       INTEGER NOT NULL,
    PRIMARY KEY (tipo, workspace_id, blueprint_id)
);
