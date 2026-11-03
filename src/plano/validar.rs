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
        regras_das_pecas(a, &mut erros);
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
    for d in &p.decisoes {
        erros.extend(plano_fora(d.plano.as_deref(), &d.acoes, &acoes).map(|q| format!("decision {}: the plan {q} is not in its acoes", d.id)));
    }
    for q in &p.perguntas {
        erros.extend(plano_fora(q.plano.as_deref(), &q.acoes, &acoes).map(|x| format!("question {}: the plan {x} is not in its acoes", q.id)));
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
    for a in &acoes {
        for d in &a.detalhes {
            if !a.planos.is_empty() && d.plano.as_deref().is_none_or(str::is_empty) {
                saida.push(format!("action {}, detail {}: no plano, so the app has to guess which plan shows this screen. Write the plan it belongs to", a.id, d.id));
            }
            if let Some(v) = d.variante.as_deref().filter(|v| !v.is_empty() && !VARIANTES.contains(v)) {
                saida.push(format!("action {}, detail {}: the variante \"{v}\" is not in the catalog yet, so the app chooses by the figure's proportion. Known: {}", a.id, d.id, VARIANTES.join(", ")));
            }
        }
    }
    for q in planos_de(&acoes).iter().filter(|q| q.id_gerado) {
        saida.push(format!("plan {}: no id, so it got one from its position; inserting a plan before it shifts the ids and shows as a change. Write the id", q.id));
    }
    saida
}

/// `plano` só aponta para um plano da própria ação. `""` é como o agente tira a peça do plano: volta a ser da ação inteira.
fn regras_das_pecas(a: &AcaoDoc, erros: &mut Vec<String>) {
    let existe = |q: &str| q.is_empty() || a.planos.iter().any(|x| x.id == q);
    for d in a.detalhes.iter().filter(|d| !d.plano.as_deref().is_none_or(existe)) {
        erros.push(format!("action {}, detail {}: the plan {} is not in this action", a.id, d.id, d.plano.as_deref().unwrap_or("")));
    }
    for k in a.checkpoints.iter().filter(|k| !k.plano.as_deref().is_none_or(existe)) {
        erros.push(format!("action {}, checkpoint {}: the plan {} is not in this action", a.id, k.id, k.plano.as_deref().unwrap_or("")));
    }
    for (i, t) in a.tabelas.iter().enumerate().filter(|(_, t)| !t.plano.as_deref().is_none_or(existe)) {
        erros.push(format!("action {}, table {}: the plan {} is not in this action", a.id, i + 1, t.plano.as_deref().unwrap_or("")));
    }
}

/// O plano citado por decisão ou pergunta, se não estiver em nenhuma das ações dela.
fn plano_fora<'a>(plano: Option<&'a str>, acoes_da_peca: &[String], acoes: &[&AcaoDoc]) -> Option<&'a str> {
    let q = plano.filter(|q| !q.is_empty())?;
    let achou = acoes.iter().filter(|a| acoes_da_peca.contains(&a.id)).flat_map(|a| &a.planos).any(|x| x.id == q);
    (!achou).then_some(q)
}

fn acoes_de(p: &PlanoDoc) -> Vec<&AcaoDoc> {
    p.projetos.iter().flat_map(|pr| &pr.acoes).collect()
}

fn planos_de<'a>(acoes: &[&'a AcaoDoc]) -> Vec<&'a PlanoItem> {
    acoes.iter().flat_map(|a| &a.planos).collect()
}

/// Regras de campo de uma ação: id, nome, planos, checkpoints, detalhes e tabelas. Figura não: quem confere é o `figura` de `validar`, no fechamento.
/// As figuras que a planta sabe desenhar (`ui/src/plano/acao_obra_formas.rs`).
const FORMAS: [&str; 8] = ["tela", "dados", "servico", "integracao", "ia", "teste", "documento", "infra"];
const ENTREGA_MAX: usize = 80;
/// Os moldes do componente Tela (`CATALOGO`, em `ui/src/plano/acao_obra_tela.rs`).
const VARIANTES: [&str; 5] = ["janela", "celular", "dialogo", "faixa", "quadro"];

fn numerico(id: &str) -> bool {
    !id.is_empty() && id.split('.').all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// O número do plano segue o da ação: na ação 1 são 1.1, 1.2; na 2.3 são 2.3.1, 2.3.2. Só vale entre números: id de letras é de planos antigos.
fn numero_fora_da_acao(acao: &str, plano: &str) -> Option<String> {
    let segue = plano.strip_prefix(acao).and_then(|r| r.strip_prefix('.')).is_some_and(numerico);
    (numerico(acao) && numerico(plano) && !segue).then(|| format!("plan {plano}: the number follows its action; in action {acao} use {acao}.1, {acao}.2 and so on"))
}

fn regras_da_acao(a: &AcaoDoc, tem_nome: bool, erros: &mut Vec<String>) {
    if a.id.is_empty() || !tem_nome {
        erros.push(format!("action {}: id and nome are required", if a.id.is_empty() { "?" } else { &a.id }));
    }
    if !a.id.is_empty() && !ref_valido(&a.id) {
        erros.push(erro_de_ref(&format!("action {}", a.id)));
    }
    for q in &a.planos {
        if q.texto.trim().is_empty() {
            erros.push(format!("plan {}: missing the texto", q.id));
        }
        if !ref_valido(&q.id) {
            erros.push(erro_de_ref(&format!("plan {}", q.id)));
        }
        erros.extend(numero_fora_da_acao(&a.id, &q.id));
        if let Some(forma) = q.forma.as_deref().filter(|f| !f.is_empty() && !FORMAS.contains(f)) {
            erros.push(format!("plan {}: forma \"{forma}\" is not valid; use one of {}", q.id, FORMAS.join(", ")));
        }
        if q.entrega.as_deref().is_some_and(|e| e.chars().count() > ENTREGA_MAX || e.contains('\n')) {
            erros.push(format!("plan {}: entrega is one line of up to {ENTREGA_MAX} characters: what exists when the plan is done", q.id));
        }
        // `""` é como o agente apaga um campo opcional: não é área inválida.
        if let Some(area) = q.area.as_deref().filter(|s| !s.is_empty() && !area_valida(s)) {
            erros.push(format!("plan {}: area \"{area}\" is not valid; use a relative path of letters, digits, spaces and . _ - /, up to 120 characters, with no \"..\"", q.id));
        }
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
