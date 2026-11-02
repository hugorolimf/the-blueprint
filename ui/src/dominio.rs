//! As regras de negócio da folha, as mesmas de `web/js/dados.js`: a situação de uma ação vem dos planos e checkpoints marcados, o progresso
//! de um blueprint pesa as ações pela duração e a janela de meses cobre as datas com uma semana de folga.

use crate::documento::{dia, iso_do_dia};
use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Situacao {
    Concluido,
    Andamento,
    Planejado,
}

#[derive(Clone)]
pub struct Checkpoint {
    pub id: String,
    pub nome: String,
    /// dias desde 1970
    pub data: Option<i64>,
    pub feito: bool,
}

#[derive(Clone)]
pub struct Plano {
    pub id: String,
    pub texto: String,
    pub feito: bool,
}

/// Uma ação com a situação já calculada (`acoesDe` da página).
#[derive(Clone)]
pub struct Acao {
    pub id: String,
    pub nome: String,
    /// o projeto da ação (`projetoId`); vazio se não tem
    pub projeto: String,
    /// a posição na lista de ações do blueprint (conta também as sem data): é o número da ação, `1.2`
    pub idx: usize,
    /// primeiro e último dia (desde 1970)
    pub a: i64,
    pub b: i64,
    /// sem data de verdade (`semData`, ou uma das duas faltando): `a` e `b` só guardam um lugar, e a janela de meses não conta com eles
    pub sem_data: bool,
    pub status: Situacao,
    pub pct: f32,
    /// itens (planos e checkpoints) feitos e total
    pub itens: (usize, usize),
    pub horas: f32,
    pub planos: Vec<Plano>,
    pub checkpoints: Vec<Checkpoint>,
}

fn lista<'a>(v: &'a Value, campo: &str) -> &'a [Value] {
    v.get(campo).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn feito(v: &Value) -> bool {
    v.get("feito").and_then(Value::as_bool) == Some(true)
}

fn texto<'a>(v: &'a Value, campo: &str) -> &'a str {
    v.get(campo).and_then(Value::as_str).unwrap_or("")
}

/// A situação e o progresso: só concluída quando TODOS os itens estão feitos; o status guardado só vale para ação sem item (`efetiva`).
fn efetiva(f: &Value) -> (Situacao, f32, (usize, usize)) {
    let (planos, marcos) = (lista(f, "planos"), lista(f, "checkpoints"));
    let (total, feitos) = (planos.len() + marcos.len(), planos.iter().chain(marcos).filter(|x| feito(x)).count());
    let guardado = texto(f, "status");
    let numero = f.get("progresso").and_then(Value::as_f64).unwrap_or(0.0) as f32;
    let mut pct = match guardado {
        "concluido" => 1.0,
        "andamento" => numero.clamp(0.0, 1.0),
        _ => 0.0,
    };
    let mut status = match guardado {
        "concluido" => Situacao::Concluido,
        "andamento" => Situacao::Andamento,
        _ => Situacao::Planejado,
    };
    if total > 0 {
        pct = feitos as f32 / total as f32;
        status = if pct >= 1.0 {
            Situacao::Concluido
        } else if pct > 0.0 || guardado == "andamento" {
            Situacao::Andamento
        } else {
            Situacao::Planejado
        };
    } else if status == Situacao::Andamento && pct >= 1.0 {
        status = Situacao::Concluido;
    }
    if status == Situacao::Concluido {
        pct = 1.0;
    }
    (status, pct, (feitos, total))
}

/// As ações do blueprint `p`, na ordem do arquivo (`acoesDe`). A sem data fica em hoje, com `sem_data`: o plano do agente pode não trazer datas, e ela conta no cartão e no progresso.
pub fn acoes_de(p: &Value) -> Vec<Acao> {
    lista(p, "acoes").iter().enumerate().map(|(idx, f)| {
        let (inicio, fim) = (dia(texto(f, "inicio")), dia(texto(f, "fim")));
        let sem_data = inicio.is_none() || fim.is_none() || f.get("semData").and_then(Value::as_bool) == Some(true);
        let a = inicio.or(fim).unwrap_or_else(|| dia(&crate::documento::hoje()).unwrap_or(0));
        let b = fim.unwrap_or(a);
        let (status, pct, itens) = efetiva(f);
        let horas = lista(f, "planos").iter().map(|x| x.get("horas").and_then(Value::as_f64).unwrap_or(0.0) as f32).sum();
        let checkpoints = lista(f, "checkpoints").iter().map(|m| Checkpoint { id: texto(m, "id").into(), nome: texto(m, "nome").into(), data: dia(texto(m, "data")), feito: feito(m) }).collect();
        let planos = lista(f, "planos").iter().map(|t| Plano { id: texto(t, "id").into(), texto: texto(t, "texto").into(), feito: feito(t) }).collect();
        Acao { id: texto(f, "id").into(), nome: texto(f, "nome").into(), projeto: texto(f, "projetoId").into(), idx, a, b: b.max(a), sem_data, status, pct, itens, horas, planos, checkpoints }
    }).collect()
}

/// Um grupo de ações do blueprint: um projeto (uma seção da folha) ou o «Geral» (id vazio).
pub struct Grupo {
    pub id: String,
    pub nome: String,
    pub descricao: String,
    /// índices em `acoes_de(p)`
    pub acoes: Vec<usize>,
}

/// O id do projeto da ação, ou vazio (Geral) se ela não tem projeto ou o projeto não existe mais (`projetoDaAcao`).
fn projeto_da(p: &Value, f: &Acao) -> String {
    if lista(p, "projetos").iter().any(|x| texto(x, "id") == f.projeto) { f.projeto.clone() } else { String::new() }
}

/// As seções do blueprint: os projetos pela data de início da primeira ação de cada um (empate: a ordem em que foram criados; sem ações vai depois) e,
/// por último, o Geral se tiver ações. `None` se o blueprint não tem projetos (`gruposDoBlueprint` e `projetosOrdenados`).
pub fn grupos(p: &Value, acoes: &[Acao]) -> Option<Vec<Grupo>> {
    let projetos = lista(p, "projetos");
    if projetos.is_empty() {
        return None;
    }
    let do_grupo = |id: &str| -> Vec<usize> { acoes.iter().enumerate().filter(|(_, f)| projeto_da(p, f) == id).map(|(i, _)| i).collect() };
    let mut ordem: Vec<(i64, usize, &Value)> = projetos.iter().enumerate().map(|(i, x)| (do_grupo(&texto(x, "id")).iter().map(|k| acoes[*k].a).min().unwrap_or(i64::MAX), i, x)).collect();
    ordem.sort_by_key(|(ini, i, _)| (*ini, *i));
    let mut grupos: Vec<Grupo> = ordem.into_iter().map(|(_, _, x)| Grupo { id: texto(x, "id").into(), nome: texto(x, "nome").into(), descricao: texto(x, "descricao").into(), acoes: do_grupo(&texto(x, "id")) }).collect();
    let geral = do_grupo("");
    if !geral.is_empty() {
        grupos.push(Grupo { id: String::new(), nome: "Geral".into(), descricao: String::new(), acoes: geral });
    }
    Some(grupos)
}

/// O número da ação na folha (`numeroDaAcao`): `<projeto>.<posição no projeto>` com projetos (o Geral é o último), senão `<blueprint>.<posição>`.
/// A posição conta todas as ações do blueprint, também as sem data.
pub fn numero_da_acao(p: &Value, f: &Acao, pi: usize, acoes: &[Acao]) -> String {
    match grupos(p, acoes) {
        None => format!("{}.{}", pi + 1, f.idx + 1),
        Some(gs) => {
            let id = projeto_da(p, f);
            let ordem = gs.iter().position(|g| g.id == id).map_or(gs.len() + 1, |k| k + 1);
            let posicao = lista(p, "acoes").iter().filter(|x| projeto_da_json(p, x) == id).position(|x| texto(x, "id") == f.id).map_or(1, |k| k + 1);
            format!("{ordem}.{posicao}")
        }
    }
}

fn projeto_da_json(p: &Value, a: &Value) -> String {
    let id = texto(a, "projetoId");
    if lista(p, "projetos").iter().any(|x| texto(x, "id") == id) { id.into() } else { String::new() }
}

/// O blueprint está concluído: tem ações e todas concluídas (o status efetivo, mesmo das que não têm datas).
pub fn concluido(p: &Value) -> bool {
    let fs = lista(p, "acoes");
    !fs.is_empty() && fs.iter().all(|f| efetiva(f).0 == Situacao::Concluido)
}

pub struct Resumo {
    /// 0 a 1; só chega a 1 quando toda ação está concluída
    pub geral: f32,
    pub concl: usize,
    pub horas: f32,
    /// todas as ações ainda planejadas
    pub futuro: bool,
}

pub fn resumo<'a>(fs: impl IntoIterator<Item = &'a Acao>) -> Resumo {
    let fs: Vec<&Acao> = fs.into_iter().collect();
    let duracao: i64 = fs.iter().map(|f| f.b - f.a + 1).sum();
    let todas = !fs.is_empty() && fs.iter().all(|f| f.status == Situacao::Concluido);
    let media = if duracao > 0 { fs.iter().map(|f| (f.b - f.a + 1) as f32 * f.pct).sum::<f32>() / duracao as f32 } else { 0.0 };
    Resumo { geral: if todas { 1.0 } else { media.min(0.99) }, concl: fs.iter().filter(|f| f.status == Situacao::Concluido).count(), horas: fs.iter().map(|f| f.horas).sum(), futuro: !fs.is_empty() && fs.iter().all(|f| f.status == Situacao::Planejado) }
}

/// O progresso de um blueprint (`progressoDe`): 1 se concluído, senão o geral.
pub fn progresso(p: &Value) -> f32 {
    if concluido(p) { 1.0 } else { resumo(&acoes_de(p)).geral }
}

/// Itens que ainda faltam nas ações (a soma de `it[1] - it[0]`).
pub fn faltam(fs: &[Acao]) -> usize {
    fs.iter().map(|f| f.itens.1 - f.itens.0).sum()
}

pub fn indice_mes(iso: &str) -> i64 {
    iso[..4].parse::<i64>().unwrap_or(0) * 12 + iso[5..7].parse::<i64>().unwrap_or(1) - 1
}

pub fn mes_iso(n: i64) -> String {
    format!("{}-{:02}", n.div_euclid(12), n.rem_euclid(12) + 1)
}

/// A janela de meses da folha: das datas das ações com uma semana de folga, em meses inteiros (`janelaPeriodo` sem período escolhido).
pub struct Janela {
    pub inicio: i64,
    pub fim: i64,
    pub t0: i64,
    pub t1: i64,
}

pub fn janela(fs: &[Acao], hoje: i64) -> Janela {
    janela_com(fs, hoje, None)
}

/// O mês `aaaa-mm` é válido (`mesValido`).
pub fn mes_valido(s: &str) -> bool {
    s.len() == 7 && s.as_bytes()[4] == b'-' && s[..4].bytes().all(|c| c.is_ascii_digit()) && matches!(s[5..].parse::<u32>(), Ok(1..=12))
}

/// Só as ações que ainda falta fazer e, nelas, só os planos e checkpoints pendentes (o filtro «Pendentes» de `acoesVisiveis`).
pub fn so_pendentes(fs: &[Acao]) -> Vec<Acao> {
    fs.iter().filter(|f| f.status != Situacao::Concluido).map(|f| Acao { planos: f.planos.iter().filter(|t| !t.feito).cloned().collect(), checkpoints: f.checkpoints.iter().filter(|m| !m.feito).cloned().collect(), ..f.clone() }).collect()
}

/// A janela de meses, ou o período escolhido (`inicio`, `fim` em `aaaa-mm`, em qualquer ordem) se for válido (`janelaPeriodo`).
pub fn janela_com(fs: &[Acao], hoje: i64, escolhido: Option<(&str, &str)>) -> Janela {
    let datadas = || fs.iter().filter(|f| !f.sem_data);
    let (a, b) = (datadas().map(|f| f.a).min().unwrap_or(hoje) - 7, datadas().map(|f| f.b).max().unwrap_or(hoje) + 7);
    let (mut inicio, mut fim) = (indice_mes(&iso_do_dia(a)), indice_mes(&iso_do_dia(b)));
    if let Some((x, y)) = escolhido.filter(|(x, y)| mes_valido(x) && mes_valido(y)) {
        (inicio, fim) = (indice_mes(x).min(indice_mes(y)), indice_mes(x).max(indice_mes(y)));
    }
    Janela { inicio, fim, t0: dia(&format!("{}-01", mes_iso(inicio))).unwrap_or(0), t1: dia(&format!("{}-01", mes_iso(fim + 1))).unwrap_or(0) }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn exemplo() -> Value {
        serde_json::from_str(include_str!("../../data.exemplo.json")).unwrap()
    }

    /// Os números que a página mostra para o exemplo (conferidos na tela web): o primeiro blueprint tem 1/3 ações, 35%, 8 itens faltando e 22 h.
    #[test]
    fn exemplo_bate_com_a_pagina() {
        let d = exemplo();
        let p = &d["blueprints"][0];
        let fs = acoes_de(p);
        let r = resumo(&fs);
        assert_eq!((r.concl, fs.len(), (r.geral * 100.0).round() as i32, faltam(&fs), r.horas), (1, 3, 35, 8, 22.0));
        let q = &d["blueprints"][1];
        let (fq, rq) = (acoes_de(q), resumo(&acoes_de(q)));
        assert_eq!((rq.concl, fq.len(), (rq.geral * 100.0).round() as i32, faltam(&fq), rq.horas), (0, 2, 15, 5, 8.5));
        // o carimbo da folha: média dos blueprints (25%)
        let media = d["blueprints"].as_array().unwrap().iter().map(progresso).sum::<f32>() / 2.0;
        assert_eq!((media * 100.0).round() as i32, 25);
    }

    /// As seções vêm pela data de início (Piloto 05/10 antes de Entregador 28/10, mesmo criado depois) e o Geral por último; a ação 3 é a `2.1`.
    #[test]
    fn projetos_ordenam_por_inicio_e_o_geral_vai_por_ultimo() {
        let p = serde_json::json!({ "projetos": [{ "id": "B", "nome": "Entregador" }, { "id": "A", "nome": "Piloto" }], "acoes": [
            { "id": "a1", "nome": "x", "inicio": "2026-10-05", "fim": "2026-10-12", "projetoId": "A" },
            { "id": "a2", "nome": "y", "inicio": "2026-10-13", "fim": "2026-10-27", "projetoId": "A" },
            { "id": "a3", "nome": "z", "inicio": "2026-10-28", "fim": "2026-11-10", "projetoId": "B" },
            { "id": "a4", "nome": "w", "inicio": "2026-10-20", "fim": "2026-11-02", "projetoId": "inexistente" } ] });
        let fs = acoes_de(&p);
        let gs = grupos(&p, &fs).unwrap();
        assert_eq!(gs.iter().map(|g| g.id.as_str()).collect::<Vec<_>>(), ["A", "B", ""]);
        assert_eq!(gs[2].acoes, [3], "projeto que não existe mais cai no Geral");
        assert_eq!(numero_da_acao(&p, &fs[2], 0, &fs), "2.1");
        assert_eq!(numero_da_acao(&p, &fs[1], 0, &fs), "1.2");
        assert_eq!(numero_da_acao(&p, &fs[3], 0, &fs), "3.1");
        assert!(grupos(&serde_json::json!({ "acoes": [] }), &[]).is_none());
    }

    #[test]
    fn pendentes_tira_o_que_esta_feito_e_o_periodo_escolhido_manda_na_janela() {
        let d = exemplo();
        let fs = acoes_de(&d["blueprints"][0]);
        let pend = so_pendentes(&fs);
        // a ação concluída sai; na API só restam os 3 planos e o checkpoint por fazer, e no entregador tudo
        assert_eq!(pend.len(), 2);
        assert_eq!(pend[0].planos.len(), 3);
        assert!(pend.iter().all(|f| f.planos.iter().all(|t| !t.feito) && f.checkpoints.iter().all(|m| !m.feito)));
        let hoje = dia("2026-10-31").unwrap();
        let auto = janela(&fs, hoje);
        let outubro = janela_com(&fs, hoje, Some(("2026-10", "2026-10")));
        assert_eq!((mes_iso(outubro.inicio), mes_iso(outubro.fim)), ("2026-10".to_string(), "2026-10".to_string()));
        assert!(auto.t1 - auto.t0 > outubro.t1 - outubro.t0);
        // período invertido e inválido
        assert_eq!(janela_com(&fs, hoje, Some(("2026-11", "2026-09"))).inicio, indice_mes("2026-09"));
        assert_eq!(janela_com(&fs, hoje, Some(("lixo", "2026-10"))).inicio, auto.inicio);
        assert!(mes_valido("2026-12") && !mes_valido("2026-13") && !mes_valido("2026-1"));
    }

    #[test]
    fn so_conclui_com_todos_os_itens_feitos() {
        let f = serde_json::json!({ "status": "andamento", "progresso": 0.4, "planos": [{ "feito": true }, { "feito": false }], "checkpoints": [] });
        assert_eq!(efetiva(&f).0, Situacao::Andamento);
        let g = serde_json::json!({ "status": "planejado", "planos": [{ "feito": true }], "checkpoints": [{ "feito": true }] });
        assert_eq!(efetiva(&g).0, Situacao::Concluido);
        let h = serde_json::json!({ "status": "andamento", "progresso": 1 });
        assert_eq!(efetiva(&h).0, Situacao::Concluido);
    }

    #[test]
    fn janela_cobre_as_datas_com_folga() {
        let fs = acoes_de(&exemplo()["blueprints"][0]);
        let j = janela(&fs, dia("2026-10-31").unwrap());
        assert!(j.t0 <= fs.iter().map(|f| f.a).min().unwrap() - 7 + 31);
        assert_eq!(mes_iso(indice_mes("2026-10-05")), "2026-10");
        assert_eq!(mes_iso(j.fim + 1).len(), 7);
    }
}
