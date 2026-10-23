# Movimento de expandir e recolher (ação 1.13)

Decisão de 2026-10-23, para o usuário aprovar no checkpoint «Usuário aprovou o movimento de expandir e recolher».

## O que muda na tela

Abrir ou fechar as ações do blueprint, uma seção de projeto ou "Expandir tudo / Recolher tudo" (botões e menu do botão direito) deixa de ser um pulo. Vale também para o clique numa ação compacta, que abre as ações e leva até ela.

| Momento | O que acontece | Duração |
| --- | --- | --- |
| Recolher | Planos e checkpoints esmaecem (`fill-opacity` e `stroke-opacity` de 1 a 0), de baixo para cima. | 150 ms |
| Redesenho | A folha é redesenhada já no estado final. | instantâneo |
| Depois do redesenho | O que está abaixo desliza da posição antiga para a nova (`posicoesDaFolha` + `organizarFolha`). | 350 ms |
| Expandir | Planos e checkpoints que não existiam aparecem de cima para baixo (de 0 a 1, atraso de 20 ms por item, no máximo 240 ms). | 220 ms cada |

- **Curva:** `cubic-bezier(.45,.05,.2,1)`, a mesma do resto da folha. O deslize passa de 650 ms (troca de período e filtro) para 350 ms só aqui; os outros caminhos não mudam.
- **Barras:** continuam desenhando-se (`desenharBarras`) como hoje depois de abrir e fechar.
- **Limites (regra do projeto, nada de centenas de elementos):** deslize até 220 peças (`MAX_MOVIDOS`, já existente); entrada até 60 itens e só dentro da janela; o excedente aparece sem animação.
- **Sem animação:** movimento reduzido, aba oculta e peças fora da janela. O resultado final é idêntico.
- **Clique rápido:** o estado muda na hora a cada clique; o clique novo cancela as animações em curso, mede a folha como ela está e anima só até o estado final. Três cliques seguidos dão três trocas e nenhuma animação presa.
- **Foco:** depois do redesenho o foco volta ao controle que o recebeu (cabeçalho do blueprint, do projeto ou botão).

## O que fica fora

Rolagem automática ao expandir (só o clique numa ação compacta rola, como já fazia), animação do editor e da tela de blueprints.

## Onde fica o código

Um único ponto de entrada, `web/js/expandir.js` (`animarAbrirFechar`), chamado por `alternar` e `abrirOuFecharTudo` (`navegacao.js`), `alternarProjeto` (`projetos.js`) e `irPara`.
