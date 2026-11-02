//! O diário e os avisos da tela de agentes. `PediuAjuda`, `AgenteSumiu` e `FalhouAoIniciar` saem de `EstadoEsc.causa`. As frases guardam a chave (`<bp>|<ref>`); o texto mostra só o `ref`.
use super::escalonar::{Causa, EstadoEsc, PlanoEsc, Situacao};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frase {
    PediuAjuda { id: String },
    Chamou { id: String },
    Segurou { id: String, por: String, area: String },
    Conferiu { id: String },
    EstaParado { id: String, por: String },
    DependeSemIniciar { id: String, faltam: Vec<String> },
    DependeFalta { id: String, dep: String },
    EsperaPlano { id: String, por: String },
    Mudou { id: String },
    EsperaArea { id: String, por: String, area: String },
    AgenteSumiu { id: String },
    FalhouAoIniciar { id: String, motivo: String },
    PlanoRemovido { id: String },
}

fn de_causa(id: String, causa: Option<&Causa>) -> Frase {
    match causa {
        Some(Causa::Sumiu) => Frase::AgenteSumiu { id },
        Some(Causa::Falhou(motivo)) => Frase::FalhouAoIniciar { id, motivo: motivo.clone() },
        Some(Causa::Ajuda) | None => Frase::PediuAjuda { id },
    }
}

/// O que está acontecendo agora, em ordem de atenção: ajuda primeiro, depois quem foi chamado, segurado e concluído.
pub fn diario(planos: &[PlanoEsc], exec: &BTreeMap<String, EstadoEsc>) -> Vec<Frase> {
    let de = |situacao: Situacao| planos.iter().filter_map(move |q| exec.get(&q.chave).filter(|e| e.situacao == situacao).map(|e| (q.chave.clone(), e)));
    let mut saida: Vec<Frase> = de(Situacao::Bloqueado).map(|(id, e)| de_causa(id, e.causa.as_ref())).collect();
    saida.extend(de(Situacao::Trabalhando).map(|(id, _)| Frase::Chamou { id }));
    saida.extend(de(Situacao::Area).map(|(id, e)| Frase::Segurou { id, por: e.por.clone().unwrap_or_default(), area: e.area.clone().unwrap_or_default() }));
    saida.extend(de(Situacao::Pronto).map(|(id, _)| Frase::Conferiu { id }));
    saida
}

/// O que a pessoa precisa ver. `PediuAjuda` fica só no diário; `PlanoRemovido` é o motor quem diz, porque aqui o plano já não está na lista.
pub fn avisos(planos: &[PlanoEsc], exec: &BTreeMap<String, EstadoEsc>) -> Vec<Frase> {
    let mut saida = Vec::new();
    for q in planos {
        let Some(e) = exec.get(&q.chave) else { continue };
        let id = q.chave.clone();
        let por = e.por.clone().unwrap_or_default();
        match e.situacao {
            Situacao::Parado => saida.push(Frase::EstaParado { id, por }),
            Situacao::Dependencia => avisos_de_dependencia(planos, q, e, &mut saida),
            Situacao::Revisao => saida.push(Frase::Mudou { id }),
            Situacao::Area => saida.push(Frase::EsperaArea { id, por, area: e.area.clone().unwrap_or_default() }),
            Situacao::Bloqueado if matches!(e.causa, Some(Causa::Sumiu | Causa::Falhou(_))) => saida.push(de_causa(id, e.causa.as_ref())),
            _ => {}
        }
    }
    saida
}

/// Dependência que não existe mais, dependência sem iniciar ou, se nada disso, a simples espera.
fn avisos_de_dependencia(planos: &[PlanoEsc], q: &PlanoEsc, e: &EstadoEsc, saida: &mut Vec<Frase>) {
    let existe = |d: &String| planos.iter().any(|p| &p.chave == d);
    let id = &q.chave;
    let sumidas: Vec<&String> = e.sem_iniciar.iter().filter(|d| !existe(d)).collect();
    let faltam: Vec<String> = e.sem_iniciar.iter().filter(|d| existe(d)).cloned().collect();
    saida.extend(sumidas.iter().map(|d| Frase::DependeFalta { id: id.clone(), dep: (*d).clone() }));
    if !faltam.is_empty() {
        saida.push(Frase::DependeSemIniciar { id: id.clone(), faltam });
    } else if sumidas.is_empty() {
        saida.push(Frase::EsperaPlano { id: id.clone(), por: e.por.clone().unwrap_or_default() });
    }
}

/// A chave `<bp>|<ref>` como a pessoa a lê.
fn curto(chave: &str) -> &str {
    chave.rsplit_once('|').map_or(chave, |(_, r)| r)
}

/// O texto para a tela, sem dizer que alguém está vendo ou conferiu: é o caso do orquestrador desligado.
pub fn texto(f: &Frase) -> String {
    texto_com(f, false)
}

/// Com o orquestrador ligado, `PediuAjuda` e `Conferiu` dizem o que ele faz; desligado, só o que o app viu.
pub fn texto_com(f: &Frase, orquestrador: bool) -> String {
    match f {
        Frase::PediuAjuda { id } if orquestrador => crate::tr!("{} asked for help: checking with the agent", "{} pediu ajuda: vendo com o agente", curto(id)),
        Frase::PediuAjuda { id } => crate::tr!("{} asked for help", "{} pediu ajuda", curto(id)),
        Frase::Chamou { id } => crate::tr!("Called an agent for {}", "Chamou um agente para {}", curto(id)),
        Frase::Segurou { id, por, area } => crate::tr!("Held {}: {} is already working on {}", "Segurou {}: {} já mexe em {}", curto(id), curto(por), area),
        Frase::Conferiu { id } if orquestrador => crate::tr!("Checked {}: completed, with evidence", "Conferiu {}: concluído, com evidência", curto(id)),
        Frase::Conferiu { id } => crate::tr!("{}: completed with evidence", "{}: concluído com evidência", curto(id)),
        Frase::EstaParado { id, por } => crate::tr!("{} is stopped: {} needs attention.", "{} está parado: {} precisa de atenção.", curto(id), curto(por)),
        Frase::DependeSemIniciar { id, faltam } => {
            let lista = faltam.iter().map(|d| curto(d)).collect::<Vec<_>>().join(", ");
            crate::tr!("{} depends on {}, not started yet.", "{} depende de {}, que ainda não foi iniciado.", curto(id), lista)
        }
        Frase::DependeFalta { id, dep } => crate::tr!("{} depends on {}, which no longer exists.", "{} depende de {}, que não existe mais.", curto(id), curto(dep)),
        Frase::EsperaPlano { id, por } => crate::tr!("{} is waiting for {} to be ready.", "{} espera {} ficar pronto.", curto(id), curto(por)),
        Frase::Mudou { id } => crate::tr!("{} changed: review it before starting.", "{} mudou: revise antes de iniciar.", curto(id)),
        Frase::EsperaArea { id, por, area } => crate::tr!("{} is waiting for {}: both work on {}.", "{} espera {}: os dois mexem em {}.", curto(id), curto(por), area),
        Frase::AgenteSumiu { id } => crate::tr!("The agent of {} disappeared: its work stayed in the copy.", "O agente de {} sumiu: o trabalho ficou na cópia.", curto(id)),
        Frase::FalhouAoIniciar { id, motivo } => crate::tr!("{} failed: {}.", "{} falhou: {}.", curto(id), motivo),
        Frase::PlanoRemovido { id } => crate::tr!("{} was removed from the blueprint: its agent was cancelled.", "{} saiu do blueprint: o agente dele foi cancelado.", curto(id)),
    }
}
