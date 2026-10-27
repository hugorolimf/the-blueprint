# O modelo: Workspace, Blueprint, Projeto, Ação, Plano e Checkpoint

[Voltar ao README](../../README.pt-BR.md) · [English](../model.md)

```
Workspace
└─ Blueprint            o escopo
   └─ Projeto           agrupa ações dentro do blueprint (opcional; sem projeto, a ação fica em "Geral")
      └─ Ação           início, fim, status, instruções (era "fase")
         ├─ Plano       título, descrição, horas estimadas, feito (era "tarefa")
         └─ Checkpoint  nome, data, feito, evidência (era "marco")
```

Na folha, cada projeto é uma seção numerada (01, 02…) com o cabeçalho de um blueprint e as ações dele abaixo (nasce fechada); as ações são numeradas por projeto (02.3 = projeto 2, ação 3).

- A porcentagem de cada ação e blueprint vem só dos planos e checkpoints marcados.
- **Tempo estimado e decorrido:** planos aceitam qualquer estimativa positiva. O primeiro aviso de progresso e o aviso de conclusão registram início e fim de forma durável, mesmo sem página aberta; a conclusão continua sujeita a revisão. A diferença é tempo decorrido, incluindo pausas. O guia de planejamento usa o histórico como contexto, não como promessa.
- **Dias e Horas:** no blueprint, escolha Horas (ou Alt+H) e selecione um dia para comparar a sequência estimada com o tempo decorrido na mesma folha. As estimativas começam à meia-noite da data de início da ação; plano sem estimativa usa 1,5 hora. Passe o mouse ou dê foco a uma faixa para ver o plano e a duração. O tempo em andamento é atualizado a cada aviso do agente. A preferência fica neste navegador.
- Tudo o que o agente envia é **proposta**: nada muda nos seus dados até você aplicar na tela **Planejar**. Concluir um plano exige **evidência** (comando ou teste rodado e o resultado), que vira comentário do plano.
- O app não manda trabalho a ninguém por conta própria: **você** inicia uma ação, ou digitando `blueprint claude`, ou pelo botão **Iniciar com o Claude** na folha da ação, que pede confirmação (gasta tokens), abre um terminal novo na pasta do blueprint com o Claude Code e o prompt da ação como argumento da sessão nova (nunca por monitor ou canal, e nada vai para uma sessão já aberta). Fora isso o app só mostra o que os agentes informam: o Claude na folha (um por agente e subagente, no plano em que trabalha; o concluído fica até você clicar e ler a mensagem).
