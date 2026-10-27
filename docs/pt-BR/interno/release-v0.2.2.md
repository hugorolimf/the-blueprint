# The Blueprint v0.2.2 (beta)

[English](../../interno/release-v0.2.2.md)

## Novidades

- **O link de convite abre primeiro o app instalado.** O instalador registra o protocolo `blueprint://`. Quando o convidado abre um link de convite, a página tenta o app instalado (o navegador e o app perguntam uma vez, e o app mostra o endereço que vai abrir); se o app não estiver instalado ou não responder em 2,5 s, a versão web continua na mesma aba. O app mostra o workspace compartilhado numa janela própria. Veja [Compartilhar pelo app](../compartilhar.md).
- **O Claude do convidado se conecta por essa janela.** Em Minha chave, **Conectar meu Claude neste computador** faz o que o `blueprint conectar` faz, depois de uma pergunta nativa com o endereço, sem comando para copiar.
- **Janela de progresso quando o app se atualiza.**

## Como instalar

Baixe o `Blueprint-0.2.2-instalador.exe` abaixo (instala só para o seu usuário). Quem está na 0.1.2 ou mais nova recebe a atualização sozinho e instala com um clique em Atualizar. O `SHA256SUMS` traz o hash do instalador. O protocolo `blueprint://` é registrado pelo instalador, então atualize (ou reinstale) para ter o fluxo de convite.

O instalador não é assinado: o Windows ou o Defender podem avisar ou bloquear na primeira execução.
