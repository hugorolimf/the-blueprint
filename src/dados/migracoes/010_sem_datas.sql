-- O programa não tem mais datas nem durações de planejamento (pedido do usuário em 2026-11-02): início e fim das ações, data e hora dos checkpoints, estimativa
-- em horas e tempo medido dos planos. Apaga de vez, sem cópia. O JSON aninhado (versões do plano, fila do MCP, decisões do blueprint) é limpo em `migracoes.rs`.
ALTER TABLE acoes DROP COLUMN inicio;
ALTER TABLE acoes DROP COLUMN fim;
ALTER TABLE checkpoints DROP COLUMN data;
ALTER TABLE checkpoints DROP COLUMN feito_em;
DROP TABLE tempos_planos;
UPDATE acoes SET extra = json_remove(extra, '$.inicio', '$.fim', '$.semData') WHERE json_valid(extra);
UPDATE checkpoints SET extra = json_remove(extra, '$.data', '$.feitoEm') WHERE json_valid(extra);
UPDATE planos SET extra = json_remove(extra, '$.horas', '$.feitoEm') WHERE json_valid(extra);
UPDATE planos SET agente = json_remove(agente, '$.inicioEm', '$.feitoEm') WHERE agente IS NOT NULL AND json_valid(agente);
DELETE FROM historico_campos WHERE (tipo = 'acao' AND campo IN ('inicio', 'fim', 'semData')) OR (tipo = 'checkpoint' AND campo IN ('data', 'feitoEm')) OR (tipo = 'plano' AND campo IN ('horas', 'agente.inicioEm', 'agente.feitoEm'));
