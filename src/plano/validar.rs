//! As regras de conteúdo de um plano (porte de `pacote.mjs:19-43`), com mensagens em inglês que o agente consegue corrigir. Só compara: quem lê figura do disco é quem chama.
//! Unicidade, ciclo e dependência valem para o plano inteiro. As regras de campo só para o que a versão tocou: blueprint antigo tolera `inicio > fim` e o agente não pode corrigir o que não mandou.
use super::formato::{AcaoDoc, Decisao, Detalhe, Pergunta, PlanoDoc, PlanoItem, FORMATO};
use super::ids::{area_valida, ref_valido};
use std::collections::{BTreeMap, BTreeSet};

const MUITAS_ACOES: usize = 200;

/// Fechamento e `salvar_plano`; `None` = todas as ações. `figura` confere a figura de um detalhe da ação e só é chamada para detalhe com figura e sem `figura_info`: o que já veio da árvore não pede linha nova em `plano_figuras`.
pub fn validar(p: &PlanoDoc, tocadas: Option<&[String]>, figura: &dyn Fn(&str, &Detalhe) -> Vec<String>) -> Vec<String> {
    let mut erros = Vec::new();
    if p.formato != FORMATO {
        erros.push(format!("formato: use \"{FORMATO}\""));
    }
    if p.blueprint.trim().is_empty() {
        erros.push("blueprint: missing the name".to_string());
    }
    // Numa escrita, o documento vem da árvore: blueprint ainda sem nada não é erro de quem só escreveu uma peça.
    if p.projetos.is_empty() && tocadas.is_none() {
        erros.push("projetos: at least one is required, with its acoes (the name is optional)".to_string());
        return erros;
    }
    let acoes = acoes_de(p);
    for a in acoes.iter().copied().filter(|a| tocadas.is_none_or(|t| t.contains(&a.id))) {
        regras_da_acao(a, !a.nome.trim().is_empty(), &mut erros);
        for d in a.detalhes.iter().filter(|d| d.figura.is_some() && d.figura_info.is_none()) {
            erros.extend(figura(&a.id, d));
        }
    }
    regras_do_conjunto(&acoes, &mut erros);
    let planos = planos_de(&acoes);
    let conhecidos: BTreeSet<&str> = planos.iter().map(|q| q.id.as_str()).collect();
    for q in &planos {
        for d in q.depende_de.iter().filter(|d| !conhecidos.contains(d.as_str())) {
            erros.push(format!("plan {}: the plan {d} does not exist", q.id));
        }
    }
    regras_de_decisoes(&p.decisoes, &mut erros);
    regras_de_perguntas(&p.perguntas, &mut erros);
    let refs: BTreeSet<&str> = acoes.iter().map(|a| a.id.as_str()).collect();
    for d in &p.decisoes {
        erros.extend(d.acoes.iter().filter(|a| !refs.contains(a.as_str())).map(|a| format!("decision {}: the action {a} does not exist", d.id)));
    }
    for q in &p.perguntas {
        erros.extend(q.acoes.iter().filter(|a| !refs.contains(a.as_str())).map(|a| format!("question {}: the action {a} does not exist", q.id)));
    }
    erros
}

/// Ao chegar: não exige que `depende_de` exista (a outra peça pode vir depois); ciclo entre os conhecidos já é erro. `acao` vale por cima da ação de mesmo id em `p`, tanto faz se já foi mesclada. `escrever` omite o nome: basta `p` já ter um. Recusar `id_gerado` é de quem chama (só `salvar_plano` aceita).
pub fn validar_peca(p: &PlanoDoc, acao: &AcaoDoc) -> Vec<String> {
    let mut erros = Vec::new();
    let mut acoes: Vec<&AcaoDoc> = acoes_de(p).into_iter().filter(|a| a.id != acao.id).collect();
    let tem_nome = !acao.nome.trim().is_empty() || acoes_de(p).iter().any(|a| a.id == acao.id && !a.nome.trim().is_empty());
    regras_da_acao(acao, tem_nome, &mut erros);
    acoes.push(acao);
    regras_do_conjunto(&acoes, &mut erros);
    regras_de_decisoes(&p.decisoes, &mut erros);
    regras_de_perguntas(&p.perguntas, &mut erros);
    erros
}

/// O primeiro ciclo de dependências, em ordem de documento, começando e terminando no mesmo id (`["a","b","a"]`). Dependência desconhecida é ignorada. Sem recursão: uma cadeia longa num plano hostil não pode estourar a pilha da thread do servidor.
pub fn achar_ciclo(planos: &[&PlanoItem]) -> Option<Vec<String>> {
    let por_id: BTreeMap<&str, &PlanoItem> = planos.iter().map(|q| (q.id.as_str(), *q)).collect();
    let mut feitos: BTreeSet<&str> = BTreeSet::new();
    let mut posicao: BTreeMap<&str, usize> = BTreeMap::new(); // id -> lugar em `caminho`, só enquanto está no caminho
    let mut caminho: Vec<(&str, usize)> = Vec::new(); // (id, próxima dependência a visitar)
    for q in planos {
        let inicio = q.id.as_str();
        if feitos.contains(inicio) {
            continue;
        }
        posicao.insert(inicio, 0);
        caminho.push((inicio, 0));
        while let Some(topo) = caminho.last_mut() {
            let (id, proxima) = (topo.0, topo.1);
            let Some(dep) = por_id[id].depende_de.get(proxima).map(String::as_str) else {
                caminho.pop();
                posicao.remove(id);
                feitos.insert(id);
                continue;
            };
            topo.1 += 1;
            if let Some(&i) = posicao.get(dep) {
                let mut ciclo: Vec<String> = caminho[i..].iter().map(|(c, _)| c.to_string()).collect();
                ciclo.push(dep.to_string());
                return Some(ciclo);
            }
            if por_id.contains_key(dep) && !feitos.contains(dep) {
                posicao.insert(dep, caminho.len());
                caminho.push((dep, 0));
            }
        }
    }
    None
}

/// O que não impede salvar, mas o agente deve saber: `id_gerado`, projeto sem ação, mais de 200 ações.
pub fn avisos(p: &PlanoDoc) -> Vec<String> {
    let mut saida = Vec::new();
    for (i, pr) in p.projetos.iter().enumerate() {
        if pr.acoes.is_empty() {
            let nome = pr.nome.clone().unwrap_or_else(|| (i + 1).to_string());
            saida.push(format!("project {nome}: no action yet"));
        }
    }
    let acoes = acoes_de(p);
    if acoes.len() > MUITAS_ACOES {
        saida.push(format!("plan: {} actions; more than {MUITAS_ACOES} is hard to read and to review", acoes.len()));
    }
    for q in planos_de(&acoes).iter().filter(|q| q.id_gerado) {
        saida.push(format!("plan {}: no id, so it got one from its position; inserting a plan before it shifts the ids and shows as a change. Write the id", q.id));
    }
    saida
}

fn acoes_de(p: &PlanoDoc) -> Vec<&AcaoDoc> {
    p.projetos.iter().flat_map(|pr| &pr.acoes).collect()
}

fn planos_de<'a>(acoes: &[&'a AcaoDoc]) -> Vec<&'a PlanoItem> {
    acoes.iter().flat_map(|a| &a.planos).collect()
}

/// Regras de campo de uma ação: id, nome, datas, planos, checkpoints, detalhes e tabelas. Figura não: quem confere é o `figura` de `validar`, no fechamento.
fn regras_da_acao(a: &AcaoDoc, tem_nome: bool, erros: &mut Vec<String>) {
    if a.id.is_empty() || !tem_nome {
        erros.push(format!("action {}: id and nome are required", if a.id.is_empty() { "?" } else { &a.id }));
    }
    if !a.id.is_empty() && !ref_valido(&a.id) {
        erros.push(erro_de_ref(&format!("action {}", a.id)));
    }
    if a.de.is_some() != a.ate.is_some() {
        erros.push(format!("action {}: use de and ate together, or neither", a.id));
    }
    let (de_ok, ate_ok) = (data_de(a.de.as_deref(), "de", &format!("action {}", a.id), erros), data_de(a.ate.as_deref(), "ate", &format!("action {}", a.id), erros));
    if let (Some(de), Some(ate), true, true) = (&a.de, &a.ate, de_ok, ate_ok) {
        if de > ate {
            erros.push(format!("action {}: de ({de}) is after ate ({ate})", a.id));
        }
    }
    for q in &a.planos {
        if q.texto.trim().is_empty() {
            erros.push(format!("plan {}: missing the texto", q.id));
        }
        if !ref_valido(&q.id) {
            erros.push(erro_de_ref(&format!("plan {}", q.id)));
        }
        // `""` é como o agente apaga um campo opcional: não é área inválida.
        if let Some(area) = q.area.as_deref().filter(|s| !s.is_empty() && !area_valida(s)) {
            erros.push(format!("plan {}: area \"{area}\" is not valid; use a relative path of letters, digits, spaces and . _ - /, up to 120 characters, with no \"..\"", q.id));
        }
    }
    for c in &a.checkpoints {
        data_de(c.data.as_deref(), "data", &format!("checkpoint {}", c.id), erros);
    }
    for d in &a.detalhes {
        if !ref_valido(&d.id) {
            erros.push(erro_de_ref(&format!("detail {}-{}", a.id, d.id)));
        }
    }
    for (i, t) in a.tabelas.iter().enumerate() {
        if t.colunas.is_empty() {
            erros.push(format!("action {}, table {}: colunas is required", a.id, i + 1));
        }
    }
}

/// Unicidade e ciclo sobre todas as ações: valem mesmo para o que a versão não tocou.
fn regras_do_conjunto(acoes: &[&AcaoDoc], erros: &mut Vec<String>) {
    unicos(acoes.iter().map(|a| a.id.as_str()), "action", "the ids link one version of the plan to the next", erros);
    let planos = planos_de(acoes);
    unicos(planos.iter().map(|q| q.id.as_str()), "plan", "each plan is one agent's work", erros);
    for a in acoes {
        unicos(a.detalhes.iter().map(|d| d.id.as_str()), &format!("action {}, detail", a.id), "each detail is reviewed on its own", erros);
        unicos(a.checkpoints.iter().map(|c| c.id.as_str()), &format!("action {}, checkpoint", a.id), "each checkpoint is marked on its own", erros);
    }
    if let Some(ciclo) = achar_ciclo(&planos) {
        erros.push(format!("plans in a cycle: {}; no agent could start", ciclo.join(" -> ")));
    }
}

fn regras_de_decisoes(decisoes: &[Decisao], erros: &mut Vec<String>) {
    unicos(decisoes.iter().map(|d| d.id.as_str()), "decision", "each decision is reviewed on its own", erros);
    for d in decisoes {
        if !ref_valido(&d.id) {
            erros.push(erro_de_ref(&format!("decision {}", d.id)));
        }
        data_de(d.data.as_deref(), "data", &format!("decision {}", d.id), erros);
    }
}

fn regras_de_perguntas(perguntas: &[Pergunta], erros: &mut Vec<String>) {
    unicos(perguntas.iter().map(|q| q.id.as_str()), "question", "each question has one answer", erros);
    for q in perguntas.iter().filter(|q| !ref_valido(&q.id)) {
        erros.push(erro_de_ref(&format!("question {}", q.id)));
    }
}

fn erro_de_ref(nome: &str) -> String {
    format!("{nome}: the id must start with a letter or digit and use only letters, digits and . _ -, up to 64 characters, with no \"..\" and not ending in \".\" or \".lock\"")
}

/// Id repetido, ou diferente só por caixa: ramo e pasta colidem no NTFS. Id vazio já é erro de campo.
fn unicos<'a>(ids: impl Iterator<Item = &'a str>, tipo: &str, porque: &str, erros: &mut Vec<String>) {
    let mut vistos: BTreeMap<String, &str> = BTreeMap::new();
    for id in ids.filter(|id| !id.is_empty()) {
        match vistos.get(&id.to_lowercase()) {
            Some(&outro) if outro == id => erros.push(format!("{tipo} {id}: repeated id; {porque}")),
            Some(&outro) => erros.push(format!("{tipo} {id}: differs from {outro} only by case; branches and folders would collide")),
            None => drop(vistos.insert(id.to_lowercase(), id)),
        }
    }
}

/// Registra o erro de data e diz se a data é boa. Sem data é bom.
fn data_de(data: Option<&str>, campo: &str, onde: &str, erros: &mut Vec<String>) -> bool {
    match data {
        Some(s) if !data_valida(s) => {
            erros.push(format!("{onde}: {campo} \"{s}\" is not a valid YYYY-MM-DD date"));
            false
        }
        _ => true,
    }
}

fn data_valida(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' || !b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit()) {
        return false;
    }
    let numero = |faixa: std::ops::Range<usize>| s[faixa].parse::<u32>().unwrap_or(0);
    let (ano, mes, dia) = (numero(0..4), numero(5..7), numero(8..10));
    let bissexto = ano % 4 == 0 && (ano % 100 != 0 || ano % 400 == 0);
    let dias = match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if bissexto => 29,
        2 => 28,
        _ => return false,
    };
    (1..=dias).contains(&dia)
}
