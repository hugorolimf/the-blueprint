# The Blueprint v0.2.0 (beta)

[English](../release-v0.2.0.md)

O The Blueprint deixa de ser de uma pessoa só: agora dá para trabalhar no mesmo blueprint com outras pessoas, ao vivo.

## Novidades

### Trabalhar junto

- **Compartilhar um workspace pelo app.** Um botão Compartilhar na folha abre um workspace (ou vários) para quem você convidar, direto do seu app, por um túnel HTTPS da Cloudflare. Sem servidor e sem conta: o app baixa o `cloudflared` na primeira vez (versão fixa, conferida pelo SHA-256). Os convidados usam o navegador e só veem o workspace do convite; os seus outros workspaces, o terminal, o Iniciar com o Claude e o atualizador ficam fechados para eles. Limites: funciona enquanto o seu app estiver aberto, e o endereço grátis muda a cada vez que você liga. Veja [Compartilhar pelo app](compartilhar.md).
- **Servidor de equipe.** `blueprint serve --hospedar`, atrás de um proxy HTTPS como o Caddy, hospeda workspaces para uma equipe, sempre ligado. Veja [Hospedar](hospedar.md).
- **Convites, papéis e pessoas.** O link é o convite (sem senha): dono, colaborador, supervisor e visitante, por workspace ou por blueprint, com validade e revogação. O segredo vai depois do `#`, então nunca chega ao log do proxy, e só o hash fica guardado. Painel Pessoas, links de convite mostrados uma vez, painel Acompanhar para o supervisor e link público só de leitura que abre um blueprint ou a apresentação dele.
- **Mudanças ao vivo e conflitos.** A página grava só o que mudou. As mudanças chegam a todos em menos de um segundo, a presença mostra quem está vendo e o item que outra pessoa edita ganha um contorno. Quando duas pessoas mudam o mesmo campo, vale a última e as duas veem quem mudou; o histórico por campo fica guardado.
- **O agente de cada pessoa.** Cada membro tem uma chave de MCP, e o Claude ou o Codex dele lê e propõe com o papel dele (`BLUEPRINT_URL` e `BLUEPRINT_CHAVE`).

### O Claude dentro e em volta do app

- **Claude Code dentro do app:** um painel Terminal com abas roda o `claude` oficial (todos os recursos), e o Iniciar com o Claude pode abrir nele.
- **Conversar com o plano:** um painel em cada ação manda as suas mensagens à sua sessão do Claude e mostra as respostas; o que ele sugerir chega como proposta, que você aplica. O plano ganhou também o botão Enviar ao agente.
- **Linha do tempo do agente por ação**, guardada no banco (os 50 últimos eventos, 30 dias).
- **Modo foco** (um cartão por agente trabalhando) e **pesquisa nativa** na barra, no workspace e no blueprint aberto.
- **Apresentação e impressão** do blueprint (tela cheia, A4, exportação em arquivo único).

### Também

- Inglês primeiro: a interface segue o idioma do Windows, com seletor na entrada; mensagens do servidor, hooks e status line falam o seu idioma, e o que o agente lê está em inglês.
- Tela Roadmap dentro do app, `blueprint exportar --markdown`, completar comandos no PowerShell e no Bash.
- Duas janelas da mesma pessoa não se sobrescrevem mais.
- Correções: o canal das sessões abertas pelo botão agora chega a elas (`@inline`), sessões que fecham mal saem da lista em 24 h, e um build de desenvolvimento não toma mais o seu plugin do Claude.

## Segurança

Compartilhar expõe o seu app à internet pelo túnel. Os convidados chegam por uma porta local separada, onde nada vale como você; toda rota confere o papel do convidado, e as rotas só locais nem existem ali. Leia [Compartilhar pelo app](compartilhar.md) e a [revisão de segurança](../seguranca-servidor.md) antes de compartilhar com quem você não conhece. O HTTPS num servidor hospedado de verdade foi testado por um proxy de teste, não com certificado real.

## Como instalar

Baixe o `Blueprint-0.2.0-instalador.exe` abaixo (instala só para o seu usuário). Quem está na 0.1.2 ou mais nova recebe a atualização sozinho e instala com um clique em Atualizar. O `SHA256SUMS` traz o hash do instalador.

O instalador não é assinado: o Windows ou o Defender podem avisar ou bloquear na primeira execução.
