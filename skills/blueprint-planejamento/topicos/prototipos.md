# Protótipos de ação

Quando o usuário pedir um **protótipo** de uma ação (uma tela, um dashboard, uma figura que mostre a ideia), gere-o e anexe com `anexar_prototipo(workspace_id, blueprint_id, acao_id, titulo, html)`. O app o mostra num visor ao lado da ação: ícone de tela na folha do blueprint, ou "Ver protótipo" no botão direito da ação.

- O `html` é **um documento HTML único e autocontido**: CSS, SVG e JavaScript inline. Sem imagens ou bibliotecas de fora e sem chamadas de rede: o visor roda isolado (sandbox, sem rede, sem acesso ao app) e bloqueia o que vier de fora. Máximo de 2 MB.
- Para figuras e explicações visuais, **use a skill anatomy** (figuras isométricas em SVG, com título, dica e movimento calmo) e entregue o resultado já embutido num HTML. Para telas, use HTML e CSS simples.
- Dados de exemplo, nunca dados reais do usuário; deixe claro no próprio protótipo que é uma simulação.
- Para **revisar**, chame de novo com o `prototipo_id` (vem em `ler_blueprint` com `acao_id`, em `prototipos`): ele é substituído, não duplicado. Sem `prototipo_id`, vira um protótipo novo da ação.
- O protótipo não conta no progresso e não muda planos nem checkpoints; ele ilustra as instruções da ação.
