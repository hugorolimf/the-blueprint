//! SÓ PARA CAPTURAS DE TELA, em depuração: com `BLUEPRINT_UI_DEMO=1` (ou uma lista: `agentes,chegou,escrita`) este arquivo põe dados inventados no estado local
//! para ver as telas Agentes, «O que chegou» e a escrita ao vivo com gente trabalhando, sem lançar agente de verdade. Nunca grava nada no servidor, e em versão
//! de produção (sem `debug_assertions`) e no navegador `ativo` é sempre falso.

use super::super::modelo::{Agentes, Chegou, ChegouAlterada, ChegouNova, ChegouRemovida, Escrita, EstadoEscrita, Folha, NumerosAgentes, Orquestrador, PlanoAgente, Resumo, Situacao};
use super::super::Estado;

/// Se a demonstração de `quais` está ligada.
pub fn ativo(quais: &str) -> bool {
    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    {
        std::env::var("BLUEPRINT_UI_DEMO").is_ok_and(|v| (v == "1" && !quais.contains('-')) || v.split(',').any(|p| p == quais))
    }
    #[cfg(not(all(debug_assertions, not(target_arch = "wasm32"))))]
    {
        let _ = quais;
        false
    }
}

/// A tela de agentes: troca o que o servidor respondeu pelos dados inventados, a cada quadro.
pub fn aplicar(plano: &mut Estado) {
    if ativo("agentes-ocioso") || ativo("agentes") {
        let mut a = agentes();
        if ativo("agentes-ocioso") {
            // o orquestrador ligado e dormindo: sem ninguém trabalhando
            a.orquestrador.acordado = false;
            a.orquestrador.agora.truncate(1);
        }
        plano.agentes = Some(a);
    }
}

/// Um pedido seu já na conversa do orquestrador, para ver o balão do lado direito.
pub fn semear(ctx: &egui::Context, ws: &str) {
    if ativo("agentes") && super::conversa::enviadas(ctx, ws).is_empty() {
        super::conversa::registrar(ctx, ws, "Pode integrar o 1.1.1 agora?", &["Respondi ao agente 1.1.2: pode seguir.".to_string()]);
    }
}

/// «O que chegou»: os nomes vêm da folha de verdade, para as marcas caírem nas linhas certas.
pub fn aplicar_chegou(plano: &mut Estado) {
    if ativo("chegou-vazio") {
        plano.chegou = Some(Chegou::default());
    } else if ativo("chegou") {
        plano.chegou = Some(chegou(plano.folha.as_ref()));
    }
}

#[allow(clippy::too_many_arguments)]
fn plano(grupo: &str, acao: &str, id: &str, texto: &str, situacao: Situacao, depende: &[&str], area: &str, ramo: &str) -> PlanoAgente {
    PlanoAgente {
        blueprint_id: "demo".into(),
        blueprint_nome: grupo.into(),
        acao_id: acao.into(),
        id: id.into(),
        referencia: id.into(),
        texto: texto.into(),
        situacao,
        depende_de: depende.iter().map(|d| d.to_string()).collect(),
        area: area.into(),
        ramo: ramo.into(),
        destino: "main".into(),
        execucao_id: format!("x-{id}"),
        ..Default::default()
    }
}

fn agentes() -> Agentes {
    use Situacao::*;
    let (a, b, c) = ("1.1 Coleta dos medidores", "1.2 Painel por sala", "1.3 Alerta de pico");
    let mut p = vec![
        plano(a, "a1", "1.1.1", "Ler os medidores pela rede local", Pronto, &[], "coleta", "agente/1.1.1"),
        plano(a, "a1", "1.1.2", "Guardar as leituras", Pronto, &["1.1.1"], "banco", "agente/1.1.2"),
        plano(b, "a2", "1.2.1", "Somar as leituras por sala", Trabalhando, &["1.1.2"], "painel/dados", "agente/1.2.1"),
        plano(b, "a2", "1.2.2", "Uma barra por sala no painel", Dependencia, &["1.2.1"], "painel/tela", ""),
        plano(b, "a2", "1.2.3", "Mostrar o dia e a semana", Trabalhando, &[], "painel/tela", "agente/1.2.3"),
        plano(c, "a3", "1.3.1", "Definir o limite por sala", Fila, &[], "alertas", ""),
        plano(c, "a3", "1.3.2", "Avisar quando passar do limite", Area, &["1.3.1"], "painel/tela", ""),
        plano(c, "a3", "1.3.3", "Gravar o histórico de picos", Bloqueado, &[], "banco", ""),
        plano(c, "a3", "1.3.4", "Enviar o resumo diário", Parado, &[], "email", ""),
    ];
    for q in &mut p {
        q.pode_integrar = q.situacao == Pronto;
        q.pode_cancelar = matches!(q.situacao, Trabalhando | Bloqueado);
        q.pode_retomar = matches!(q.situacao, Bloqueado | Parado);
    }
    p[3].por = "1.2.1".into();
    p[6].por = "1.2.3".into();
    p[2].subagentes = 1;
    p[7].causa = "ajuda".into();
    p[8].motivo = "terminou-sem-concluir".into();
    p[8].sem_sinal = true;
    Agentes {
        limite: 3,
        orquestrador: Orquestrador {
            ligado: true,
            acordado: true,
            agora: ["Revisando 1.2.1: falta o teste da soma por sala.", "Respondi ao agente 1.1.2: pode seguir.", "Integrei agente/1.1.1 em main, sem conflitos.", "Montei as ações do plano: 3 ações, 8 planos."].map(String::from).to_vec(),
        },
        numeros: NumerosAgentes { trabalhando: 2, fila: 1, esperando: 2, parados: 2, prontos: 2 },
        avisos: ["1.2.2 espera 1.2.1 ficar pronto.", "1.3.2 espera a área painel/tela, que 1.2.3 está usando."].map(String::from).to_vec(),
        planos: p,
    }
}

/// Novos, alterados e removidos, com os nomes das primeiras ações da folha.
fn chegou(folha: Option<&Folha>) -> Chegou {
    let acoes: Vec<(String, String)> = folha.map_or_else(Vec::new, |f| f.projetos.iter().flat_map(|p| &p.acoes).map(|a| (a.referencia.clone(), a.nome.clone())).collect());
    let pega = |i: usize, padrao: (&str, &str)| acoes.get(i).cloned().unwrap_or((padrao.0.into(), padrao.1.into()));
    let (r0, r1, r2, r3) = (pega(0, ("1.1", "Coleta dos medidores")), pega(1, ("1.2", "Painel por sala")), pega(2, ("1.3", "Alerta de pico")), pega(3, ("1.4", "Relatório mensal")));
    let nova = |(referencia, nome): (String, String), planos, detalhes, checkpoints, pode_colocar| ChegouNova { referencia, nome, planos, detalhes, checkpoints, pode_colocar };
    Chegou {
        numero: 3,
        agente: "Claude".into(),
        origem: "salvar".into(),
        novas: vec![nova(r2, 3, 2, 1, true), nova(r3, 2, 0, 0, true), nova(("1.9".into(), "Integração com o ERP".into()), 0, 0, 0, false)],
        alteradas: vec![
            ChegouAlterada { referencia: r0.0, nome: r0.1, resumo: Resumo { mais: 2, menos: 1, alterados: 3, detalhes: 1, outros: 0 } },
            ChegouAlterada { referencia: r1.0, nome: r1.1, resumo: Resumo { mais: 0, menos: 2, alterados: 1, detalhes: 0, outros: 1 } },
        ],
        removidas: vec![ChegouRemovida { referencia: "1.8".into(), nome: "Painel antigo".into() }],
        iguais: 4,
    }
}

/// A escrita ao vivo na folha: as `estagio` primeiras ações já com plano, a seguinte sendo escrita e as outras aguardando. Com `parada_min`, a escrita parou há esses minutos.
pub fn escrita(folha: &mut Folha, estagio: usize, parada_min: i64) {
    folha.escrita = Some(Escrita { escrita_id: "demo".into(), agente: "Claude".into(), origem: "escrita".into(), parada_ms: parada_min * 60_000, parada: parada_min > 0 });
    for (i, a) in folha.projetos.iter_mut().flat_map(|p| &mut p.acoes).enumerate() {
        a.nova = true;
        a.escrita = match i.cmp(&estagio) {
            std::cmp::Ordering::Less => EstadoEscrita::Pronta,
            std::cmp::Ordering::Equal => EstadoEscrita::Escrevendo,
            std::cmp::Ordering::Greater => EstadoEscrita::Aguardando,
        };
    }
    folha.renovar();
}

/// Os `n` primeiros planos de cada ação feitos e o seguinte em andamento, só na memória: é para ver a obra subir na aba Planta.
pub fn obra(f: &mut crate::plano::modelo::Folha, n: usize) {
    for a in f.projetos.iter_mut().flat_map(|p| p.acoes.iter_mut()) {
        for (i, q) in a.planos.iter_mut().enumerate() {
            match i.cmp(&n) {
                std::cmp::Ordering::Less => {
                    q.feito = true;
                    q.situacao = Situacao::Pronto;
                    q.agente = "plano-a".into();
                }
                std::cmp::Ordering::Equal => {
                    q.situacao = Situacao::Trabalhando;
                    q.agente = "plano-b".into();
                }
                std::cmp::Ordering::Greater => {}
            }
        }
    }
    f.renovar();
}
