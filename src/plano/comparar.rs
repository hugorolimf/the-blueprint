//! A diferença entre duas versões de um plano, como um git diff: por ação, por plano (pelos ids) e, dentro de um plano alterado, por palavra.
//! Porte de `design/planejar/comparar.mjs`, com os buracos fechados: `area`, `depende_de` e `instrucoes` também alteram (e travam) o plano; nome, texto, checkpoints e tabelas da ação contam em `outros`; figura com hash novo conta como detalhe mudado; ação que sumiu vai em `removidas`.
//! Reordenar não é mudança (limite assumido). Campo opcional ausente na versão nova não é mudança, porque `arvore::projetar` não o apaga; apagar é mandar `""`.
use super::formato::{AcaoDoc, CheckpointDoc, Detalhe, PlanoDoc, PlanoItem};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Acima disto (células da tabela de subsequência comum) o trecho diferente sai inteiro: o corpo aceita 2 MiB de texto, e a tabela de dois textos desse tamanho não cabe na memória.
const MAX_CELULAS: usize = 4_000_000;

#[derive(Debug, Clone, PartialEq)]
pub enum Pedaco { Igual(String), Menos(String), Mais(String) }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoLinha { Prazo, Novo, Alterado, Igual, Removido }
/// `campos`: `area`, `depende_de`, `instrucoes`.
#[derive(Debug, Clone, PartialEq)]
pub struct Linha { pub tipo: TipoLinha, pub plano: Option<PlanoItem>, pub antes: Option<PlanoItem>, pub palavras: Vec<Pedaco>, pub campos: Vec<&'static str> }
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resumo { pub mais: usize, pub menos: usize, pub alterados: usize, pub detalhes: usize, pub outros: usize }
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompAcao { pub linhas: Vec<Linha>, pub detalhes_novos: Vec<Detalhe>, pub detalhes_mudados: Vec<String>, pub detalhes_removidos: Vec<String>, pub outros: Vec<&'static str>, pub mudou: bool, pub resumo: Resumo }
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompPlano { pub novas: Vec<String>, pub alteradas: BTreeMap<String, CompAcao>, pub iguais: Vec<String>, pub removidas: Vec<String> }

/// Palavras e espaços, em ordem: juntar tudo devolve o texto original.
fn partes(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut comeco = 0;
    let mut espaco = None;
    for (i, c) in s.char_indices() {
        let agora = c.is_whitespace();
        if espaco.is_some_and(|e| e != agora) {
            out.push(&s[comeco..i]);
            comeco = i;
        }
        espaco = Some(agora);
    }
    if comeco < s.len() {
        out.push(&s[comeco..]);
    }
    out
}

#[derive(Clone, Copy, PartialEq)]
enum Lado { Igual, Menos, Mais }

/// Junta com o vizinho do mesmo lado.
fn poe(out: &mut Vec<Pedaco>, lado: Lado, s: &str) {
    match (out.last_mut(), lado) {
        (Some(Pedaco::Igual(t)), Lado::Igual) | (Some(Pedaco::Menos(t)), Lado::Menos) | (Some(Pedaco::Mais(t)), Lado::Mais) => t.push_str(s),
        (_, Lado::Igual) => out.push(Pedaco::Igual(s.into())),
        (_, Lado::Menos) => out.push(Pedaco::Menos(s.into())),
        (_, Lado::Mais) => out.push(Pedaco::Mais(s.into())),
    }
}

/// Maior subsequência comum do miolo; o empate favorece `Mais`.
fn miolo(a: &[&str], b: &[&str], out: &mut Vec<Pedaco>) {
    let (m, n) = (a.len(), b.len());
    if m.saturating_mul(n) > MAX_CELULAS {
        a.iter().for_each(|s| poe(out, Lado::Menos, s));
        b.iter().for_each(|s| poe(out, Lado::Mais, s));
        return;
    }
    let largura = n + 1;
    let mut d = vec![0u32; (m + 1) * largura];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            d[i * largura + j] = if a[i] == b[j] { d[(i + 1) * largura + j + 1] + 1 } else { d[(i + 1) * largura + j].max(d[i * largura + j + 1]) };
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < m || j < n {
        if i < m && j < n && a[i] == b[j] {
            poe(out, Lado::Igual, a[i]);
            i += 1;
            j += 1;
        } else if j < n && (i == m || d[i * largura + j + 1] >= d[(i + 1) * largura + j]) {
            poe(out, Lado::Mais, b[j]);
            j += 1;
        } else {
            poe(out, Lado::Menos, a[i]);
            i += 1;
        }
    }
}

/// Split preservando espaços; empate favorece `Mais`; junta vizinhos. Sem os `Mais` lê-se o texto antigo; sem os `Menos`, o novo.
pub fn diff_palavras(a: &str, b: &str) -> Vec<Pedaco> {
    let (a, b) = (partes(a), partes(b));
    // O começo e o fim iguais ficam fora da tabela: o caso comum, uma palavra trocada num texto longo, sai sem custo.
    let inicio = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let fim = a[inicio..].iter().rev().zip(b[inicio..].iter().rev()).take_while(|(x, y)| x == y).count();
    let mut out = Vec::new();
    a[..inicio].iter().for_each(|s| poe(&mut out, Lado::Igual, s));
    miolo(&a[inicio..a.len() - fim], &b[inicio..b.len() - fim], &mut out);
    a[a.len() - fim..].iter().for_each(|s| poe(&mut out, Lado::Igual, s));
    out
}

/// Ausente na versão nova não muda nada, e vazio é o mesmo que nunca ter tido.
fn opcional_mudou(antes: &Option<String>, depois: &Option<String>) -> bool {
    depois.as_deref().is_some_and(|d| antes.as_deref().unwrap_or("") != d)
}

fn conjunto(l: &[String]) -> BTreeSet<&str> {
    l.iter().map(String::as_str).collect()
}

fn campos_mudados(antes: &PlanoItem, depois: &PlanoItem) -> Vec<&'static str> {
    let mut campos = Vec::new();
    if opcional_mudou(&antes.area, &depois.area) {
        campos.push("area");
    }
    if conjunto(&antes.depende_de) != conjunto(&depois.depende_de) {
        campos.push("depende_de");
    }
    if opcional_mudou(&antes.instrucoes, &depois.instrucoes) {
        campos.push("instrucoes");
    }
    campos
}

fn linha(tipo: TipoLinha, plano: Option<&PlanoItem>) -> Linha {
    Linha { tipo, plano: plano.cloned(), antes: None, palavras: Vec::new(), campos: Vec::new() }
}

fn linha_do_plano(antes: Option<&PlanoItem>, q: &PlanoItem) -> Linha {
    let Some(v) = antes else { return linha(TipoLinha::Novo, Some(q)) };
    let campos = campos_mudados(v, q);
    if v.texto == q.texto && campos.is_empty() {
        return linha(TipoLinha::Igual, Some(q));
    }
    let palavras = if v.texto == q.texto { Vec::new() } else { diff_palavras(&v.texto, &q.texto) };
    Linha { tipo: TipoLinha::Alterado, plano: Some(q.clone()), antes: Some(v.clone()), palavras, campos }
}

/// O hash, quando os dois lados já têm, decide; a figura renomeada com o mesmo conteúdo não muda. Sem hash, vale o caminho.
fn detalhe_mudou(a: &Detalhe, b: &Detalhe) -> bool {
    let figura = match (&a.figura_info, &b.figura_info) {
        (Some(x), Some(y)) => x.hash != y.hash,
        _ => a.figura != b.figura,
    };
    figura || a.titulo != b.titulo || a.legenda != b.legenda || a.grupo != b.grupo || a.pede_aprovacao != b.pede_aprovacao
}

fn por_id(c: &[CheckpointDoc]) -> Vec<&CheckpointDoc> {
    let mut l: Vec<_> = c.iter().collect();
    l.sort_by(|x, y| x.id.cmp(&y.id));
    l
}

fn outros_mudados(antiga: &AcaoDoc, nova: &AcaoDoc) -> Vec<&'static str> {
    let mut outros = Vec::new();
    if antiga.nome != nova.nome {
        outros.push("nome");
    }
    if opcional_mudou(&antiga.texto, &nova.texto) {
        outros.push("texto");
    }
    if por_id(&antiga.checkpoints) != por_id(&nova.checkpoints) {
        outros.push("checkpoints");
    }
    if antiga.tabelas != nova.tabelas {
        outros.push("tabelas");
    }
    outros
}

/// As linhas do diff de uma ação: prazo, planos na ordem da versão nova, removidos no fim.
pub fn comparar_acao(antiga: &AcaoDoc, nova: &AcaoDoc) -> CompAcao {
    let mut linhas = Vec::new();
    if (&antiga.de, &antiga.ate) != (&nova.de, &nova.ate) {
        linhas.push(linha(TipoLinha::Prazo, None));
    }
    let antes: HashMap<&str, &PlanoItem> = antiga.planos.iter().map(|q| (q.id.as_str(), q)).collect();
    linhas.extend(nova.planos.iter().map(|q| linha_do_plano(antes.get(q.id.as_str()).copied(), q)));
    let ids: HashSet<&str> = nova.planos.iter().map(|q| q.id.as_str()).collect();
    linhas.extend(antiga.planos.iter().filter(|q| !ids.contains(q.id.as_str())).map(|q| linha(TipoLinha::Removido, Some(q))));

    let velhos: HashMap<&str, &Detalhe> = antiga.detalhes.iter().map(|d| (d.id.as_str(), d)).collect();
    let detalhes_novos: Vec<Detalhe> = nova.detalhes.iter().filter(|d| !velhos.contains_key(d.id.as_str())).cloned().collect();
    let detalhes_mudados: Vec<String> = nova.detalhes.iter().filter(|d| velhos.get(d.id.as_str()).is_some_and(|v| detalhe_mudou(v, d))).map(|d| d.id.clone()).collect();
    let atuais: HashSet<&str> = nova.detalhes.iter().map(|d| d.id.as_str()).collect();
    let detalhes_removidos: Vec<String> = antiga.detalhes.iter().filter(|d| !atuais.contains(d.id.as_str())).map(|d| d.id.clone()).collect();
    let outros = outros_mudados(antiga, nova);

    let conta = |t: TipoLinha| linhas.iter().filter(|l| l.tipo == t).count();
    let resumo = Resumo {
        mais: conta(TipoLinha::Novo),
        menos: conta(TipoLinha::Removido),
        alterados: conta(TipoLinha::Alterado) + conta(TipoLinha::Prazo),
        detalhes: detalhes_novos.len() + detalhes_mudados.len() + detalhes_removidos.len(),
        outros: outros.len(),
    };
    let mudou = linhas.iter().any(|l| l.tipo != TipoLinha::Igual) || resumo.detalhes > 0 || resumo.outros > 0;
    CompAcao { linhas, detalhes_novos, detalhes_mudados, detalhes_removidos, outros, mudou, resumo }
}

fn acoes(p: &PlanoDoc) -> impl Iterator<Item = &AcaoDoc> {
    p.projetos.iter().flat_map(|pr| pr.acoes.iter())
}

/// O plano inteiro: quais ações são novas, quais mudaram (com o diff), quais continuam iguais e quais saíram. Sem versão anterior, tudo é novo.
pub fn comparar_planos(anterior: Option<&PlanoDoc>, atual: &PlanoDoc) -> CompPlano {
    let antes: HashMap<&str, &AcaoDoc> = anterior.into_iter().flat_map(acoes).map(|a| (a.id.as_str(), a)).collect();
    let mut c = CompPlano::default();
    for a in acoes(atual) {
        let Some(v) = antes.get(a.id.as_str()) else {
            c.novas.push(a.id.clone());
            continue;
        };
        let comp = comparar_acao(v, a);
        if comp.mudou {
            c.alteradas.insert(a.id.clone(), comp);
        } else {
            c.iguais.push(a.id.clone());
        }
    }
    let ids: HashSet<&str> = acoes(atual).map(|a| a.id.as_str()).collect();
    c.removidas = anterior.into_iter().flat_map(acoes).filter(|a| !ids.contains(a.id.as_str())).map(|a| a.id.clone()).collect();
    c
}

/// Os planos que não podem começar enquanto a mudança da ação não for revisada: `Novo` e `Alterado` das ações alteradas.
pub fn planos_travados(c: &CompPlano) -> Vec<String> {
    c.alteradas.values().flat_map(|a| a.linhas.iter()).filter(|l| matches!(l.tipo, TipoLinha::Novo | TipoLinha::Alterado)).filter_map(|l| l.plano.as_ref().map(|q| q.id.clone())).collect()
}
