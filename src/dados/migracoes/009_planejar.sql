-- Planejar: fora da árvore e sem FK. PUT /api/dados apaga e recria a árvore; estas linhas sobrevivem, e a limpeza é por blueprint_id.
ALTER TABLE pastas ADD COLUMN caminho_real TEXT;

CREATE TABLE plano_versoes (
    blueprint_id TEXT NOT NULL, numero INTEGER NOT NULL, workspace_id TEXT NOT NULL,
    escrita_id TEXT NOT NULL UNIQUE,
    origem TEXT NOT NULL CHECK (origem IN ('escrita','salvar')),
    situacao TEXT NOT NULL CHECK (situacao IN ('escrevendo','fechada','resolvida')),
    agente TEXT NOT NULL DEFAULT '',
    plano TEXT NOT NULL,
    aguardando TEXT NOT NULL DEFAULT '[]', -- refs ainda sem conteúdo
    tocadas TEXT NOT NULL DEFAULT '[]', removidas TEXT NOT NULL DEFAULT '[]', -- o que esta escrita mandou; o resto é refeito da base ao fechar
    atualizada_em INTEGER NOT NULL, criada_em INTEGER NOT NULL, fechada_em INTEGER,
    PRIMARY KEY (blueprint_id, numero)
);
CREATE UNIQUE INDEX plano_uma_escrita ON plano_versoes (blueprint_id) WHERE situacao = 'escrevendo';
CREATE UNIQUE INDEX plano_uma_pendente ON plano_versoes (blueprint_id) WHERE situacao = 'fechada';

CREATE TABLE plano_figuras (
    blueprint_id TEXT NOT NULL, numero INTEGER NOT NULL, caminho TEXT NOT NULL,
    hash TEXT NOT NULL, tipo TEXT NOT NULL CHECK (tipo IN ('svg','png','jpg','webp')), tamanho INTEGER NOT NULL, largura INTEGER, altura INTEGER,
    PRIMARY KEY (blueprint_id, numero, caminho)
);

CREATE TABLE plano_revisoes (
    blueprint_id TEXT NOT NULL, peca TEXT NOT NULL, -- d:<id>, det:<acao>/<id>, diff:<acao>, q:<id> (ref_valido não aceita "/")
    numero INTEGER NOT NULL,
    estado TEXT NOT NULL CHECK (estado IN ('aprovada','revisar','reprovada','respondida')),
    observacao TEXT NOT NULL DEFAULT '', hash_peca TEXT NOT NULL DEFAULT '', em INTEGER NOT NULL,
    PRIMARY KEY (blueprint_id, peca)
);

CREATE TABLE execucoes (
    id TEXT PRIMARY KEY, -- 16 hex
    workspace_id TEXT NOT NULL, blueprint_id TEXT NOT NULL, acao_id TEXT NOT NULL, plano_id TEXT NOT NULL, plano_ref TEXT NOT NULL,
    situacao TEXT NOT NULL CHECK (situacao IN ('marcada','preparando','rodando','ajuda','pronta','sumiu','falhou','cancelada')),
    motivo TEXT NOT NULL DEFAULT '',
    area TEXT NOT NULL DEFAULT '', -- copiada da árvore ao marcar e ao lançar; vale enquanto a execução vive
    tentativa INTEGER NOT NULL DEFAULT 0, segredo TEXT NOT NULL DEFAULT '', -- 32 hex, novo a cada tentativa; só sai em execucao/partida
    pasta_principal TEXT NOT NULL DEFAULT '', pasta_trabalho TEXT NOT NULL DEFAULT '', repositorio TEXT NOT NULL DEFAULT '', ramo TEXT NOT NULL DEFAULT '', base TEXT NOT NULL DEFAULT '',
    marcada_em INTEGER NOT NULL, iniciada_em INTEGER, partida_em INTEGER, ultimo_sinal INTEGER, terminada_em INTEGER, cancelada_em INTEGER, integrada_em INTEGER, parar INTEGER NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX execucao_viva ON execucoes (blueprint_id, plano_id) WHERE situacao IN ('marcada','preparando','rodando','ajuda','sumiu','falhou');
CREATE INDEX execucoes_ws ON execucoes (workspace_id, situacao);

CREATE TABLE recados (
    id INTEGER PRIMARY KEY AUTOINCREMENT, workspace_id TEXT NOT NULL, blueprint_id TEXT NOT NULL DEFAULT '', acao_id TEXT NOT NULL DEFAULT '',
    execucao_id TEXT NOT NULL DEFAULT '', de TEXT NOT NULL CHECK (de IN ('usuario','orquestrador','agente')), texto TEXT NOT NULL, em INTEGER NOT NULL
);
CREATE INDEX recados_ws ON recados (workspace_id, id);

CREATE TABLE orquestrador (
    workspace_id TEXT PRIMARY KEY, ligado INTEGER NOT NULL DEFAULT 0, motivos TEXT NOT NULL DEFAULT '[]',
    urgente INTEGER NOT NULL DEFAULT 0, rodando INTEGER NOT NULL DEFAULT 0, ultimo_em INTEGER, hora_inicio INTEGER, na_hora INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE planejar_config (chave TEXT PRIMARY KEY, valor TEXT NOT NULL); -- limite:<ws>, comando_agente, comando_orquestrador
