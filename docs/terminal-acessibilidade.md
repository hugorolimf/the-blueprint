# Terminal do app: acessibilidade e atalhos

Checklist do painel do terminal (`web/js/terminal.js`, `terminal-sessao.js`). Os itens marcados com (e2e) são conferidos por
`node scripts/terminal-e2e.mjs`; os com (unit), por `node terminal-logica.test.mjs`.

## Teclado

| Tecla | Onde | O que faz |
| --- | --- | --- |
| Ctrl+` | qualquer lugar (também dentro do terminal) | mostra ou esconde o painel; ao esconder, o foco volta ao botão Terminal da barra (e2e) |
| F6 | dentro do terminal | sai do terminal para a aba ativa (e2e) |
| Shift+F6 | dentro do terminal | sai do terminal para as ferramentas do painel (e2e) |
| F6 | numa aba | entra no terminal da aba |
| Setas, Home, End | nas abas | trocam de aba (padrão ARIA de abas: só a aba ativa está na ordem do Tab) (e2e) |
| Delete | numa aba | fecha a sessão, com confirmação se o processo ainda roda (e2e) |
| Ctrl+PageUp / Ctrl+PageDown | dentro do terminal | sessão anterior / próxima (e2e) |
| Ctrl+= / Ctrl+- / Ctrl+0 | dentro do terminal | fonte maior / menor / padrão (unit) |
| Ctrl+C com seleção, Ctrl+Shift+C | dentro do terminal | copia a seleção (unit) |
| Ctrl+V | dentro do terminal | cola (o navegador cola e o xterm manda com bracketed paste) (unit) |

**Por que Esc não sai do terminal:** Esc e Ctrl+C são do Claude Code (interromper, voltar na conversa com Esc duas vezes, cancelar). Se o painel
tomasse o Esc, o usuário perderia esses comandos. O escape do terminal é F6, o atalho padrão de trocar de painel (o mesmo do navegador e do
Windows); ele está escrito no rodapé do painel e no rótulo acessível do campo do terminal. O terminal nunca prende o foco (e2e).

## Foco e leitura

- Foco visível: contorno de 2 px na tinta do app em abas, botões e no "x" de fechar; a tela com foco ganha um contorno interno.
- Abas com `role=tablist`/`tab`/`tabpanel`, `aria-selected`, `aria-controls` e `aria-labelledby`; o "x" tem rótulo "Fechar a sessão <título>".
- Botões de ligar e desligar (Dividir, Leitor de tela, Maximizar) usam `aria-pressed` (e2e para Dividir e Leitor de tela).
- "Leitor de tela" liga o `screenReaderMode` do xterm, que mantém uma árvore de texto para NVDA, JAWS e Narrador (e2e). Fica desligado por
  padrão porque custa desempenho; a escolha é guardada no navegador.
- O campo do terminal tem o rótulo "Terminal da sessão <título>. F6 sai do terminal."

## Contraste, fonte e movimento

- Todas as cores ANSI do tema passam de 4,5:1 sobre o papel azul (unit), e o `minimumContrastRatio: 4.5` do xterm clareia o que o Claude
  pintar com pouco contraste (WCAG AA).
- Fonte de 10 a 28 px (A−, A+ e os atalhos), guardada no navegador; a fonte é a mono do app (e2e).
- Sem animação: o cursor não pisca (também evita trabalho em repouso) e o painel não tem transição; `prefers-reduced-motion` desliga qualquer
  transição que venha a existir.
- Tela estreita (até 720 px): o painel ocupa 60% da altura e o Dividir empilha as telas.
