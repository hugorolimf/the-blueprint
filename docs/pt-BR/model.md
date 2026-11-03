# O modelo: Workspace, Blueprint, Projeto, Ação, Plano e Checkpoint

[Voltar ao README](../../README.pt-BR.md) · [English](../model.md)

```
Workspace
└─ Blueprint            o escopo
   └─ Projeto           agrupa ações dentro do blueprint (opcional; sem projeto, a ação fica em "Geral")
      └─ Ação           status, instruções (era "fase")
         ├─ Plano       título, descrição, feito (era "tarefa")
         └─ Checkpoint  nome, feito, evidência (era "marco")
```

Na folha, cada projeto é uma seção numerada (01, 02…) com o cabeçalho de um blueprint e as ações dele abaixo (nasce fechada); as ações são numeradas por projeto (02.3 = projeto 2, ação 3).

- A porcentagem de cada ação e blueprint vem só dos planos e checkpoints marcados.
- **Sem datas nem durações:** o blueprint não tem início, fim, prazo, estimativa nem tempo medido. As ações seguem a ordem em que foram escritas, e a folha as desenha em escada, cada uma começando onde a anterior termina, com o andamento dentro e os checkpoints no fim da barra.
- Tudo o que o agente envia é **proposta**: nada muda nos seus dados até você aplicar na tela **Planejar**. Concluir um plano exige **evidência** (comando ou teste rodado e o resultado), que vira comentário do plano.
- O app não manda trabalho a ninguém por conta própria: **você** inicia uma ação, ou digitando `blueprint claude`, ou pelo botão **Iniciar com o Claude** na folha da ação, que pede confirmação (gasta tokens), abre um terminal novo na pasta do blueprint com o Claude Code e o prompt da ação como argumento da sessão nova (nunca por monitor ou canal, e nada vai para uma sessão já aberta). Fora isso o app só mostra o que os agentes informam: o Claude na folha (um por agente e subagente, no plano em que trabalha; o concluído fica até você clicar e ler a mensagem).
