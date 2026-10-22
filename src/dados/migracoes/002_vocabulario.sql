-- Vocabulário novo (2026-10-22): as listas do JSON passam a se chamar workspaces, blueprints, acoes, planos e checkpoints.
-- As tabelas já tinham os nomes novos; só `listas` (as chaves de lista que existiam no JSON) guarda os nomes antigos.
UPDATE workspaces  SET listas = REPLACE(REPLACE(listas, '"companhias"', '"workspaces"'), '"projetos"', '"blueprints"');
UPDATE blueprints  SET listas = REPLACE(listas, '"fases"', '"acoes"');
UPDATE acoes       SET listas = REPLACE(REPLACE(listas, '"tarefas"', '"planos"'), '"marcos"', '"checkpoints"');
