# Atualizações

[Voltar ao README](../../README.pt-BR.md) · [English](../updates.md)

O app procura sozinho uma versão nova nas Releases do GitHub (ao abrir, no máximo uma vez por dia), **baixa o instalador em segundo plano** e confere o SHA-256 contra o `SHA256SUMS` da release. **Nada é instalado sem o seu clique:** quando o instalador está pronto aparece o aviso na janela (e o item "Atualizar para X…" na bandeja); só ao clicar em **Atualizar** o app copia o banco para `%APPDATA%\Blueprint\backups`, roda o instalador em silêncio, fecha e abre de novo. Se houver aviso do agente que a tela ainda não aplicou, ele recusa e diz o porquê (fechar o app o perderia). Na linha de comando: `blueprint atualizar` (procura e baixa), `blueprint atualizar --verificar` (só consulta) e `blueprint atualizar --instalar` (o clique, pelo terminal).

O app só faz um `GET` ao GitHub (sem token, sem identificador, sem telemetria), pelo `curl.exe` do Windows. Para desligar: item "Verificar sozinho ao abrir" na bandeja ou `BLUEPRINT_SEM_ATUALIZACAO=1`; desligado, o app não verifica nem baixa nada. `BLUEPRINT_ATUALIZACAO_SO_ESTAVEL=1` ignora as releases marcadas como pré-lançamento.
