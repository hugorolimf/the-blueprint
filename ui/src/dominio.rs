//! As regras de negócio da folha, as mesmas de `web/js/dados.js`: a situação de uma ação vem dos planos e checkpoints marcados, o progresso
//! de um blueprint é a média das ações e cada ação ocupa um passo da escada, na ordem do blueprint.

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
    /// a posição na lista de ações do blueprint: é o número da ação, `1.2`, e o degrau dela na escada
    pub idx: usize,
    pub status: Situacao,
    pub pct: f32,
    /// itens (planos e checkpoints) feitos e total
    pub itens: (usize, usize),
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

/// As ações do blueprint `p`, na ordem do arquivo (`acoesDe`).
pub fn acoes_de(p: &Value) -> Vec<Acao> {
    lista(p, "acoes").iter().enumerate().map(|(idx, f)| {
        let (status, pct, itens) = efetiva(f);
        let checkpoints = lista(f, "checkpoints").iter().map(|m| Checkpoint { id: texto(m, "id").into(), nome: texto(m, "nome").into(), feito: feito(m) }).collect();
        let planos = lista(f, "planos").iter().map(|t| Plano { id: texto(t, "id").into(), texto: texto(t, "texto").into(), feito: feito(t) }).collect();
        Acao { id: texto(f, "id").into(), nome: texto(f, "nome").into(), projeto: texto(f, "projetoId").into(), idx, status, pct, itens, planos, checkpoints }
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

/// As seções do blueprint: os projetos na ordem em que foram criados e, por último, o Geral se tiver ações. `None` se o blueprint não tem projetos (`gruposDoBlueprint` e `projetosOrdenados`).
pub fn grupos(p: &Value, acoes: &[Acao]) -> Option<Vec<Grupo>> {
    let projetos = lista(p, "projetos");
    if projetos.is_empty() {
        return None;
    }
    let do_grupo = |id: &str| -> Vec<usize> { acoes.iter().enumerate().filter(|(_, f)| projeto_da(p, f) == id).map(|(i, _)| i).collect() };
    let mut grupos: Vec<Grupo> = projetos.iter().map(|x| Grupo { id: texto(x, "id").into(), nome: texto(x, "nome").into(), descricao: texto(x, "descricao").into(), acoes: do_grupo(&texto(x, "id")) }).collect();
    let geral = do_grupo("");
    if !geral.is_empty() {
        grupos.push(Grupo { id: String::new(), nome: "Geral".into(), descricao: String::new(), acoes: geral });
    }
    Some(grupos)
}

/// O número da ação na folha (`numeroDaAcao`): `<projeto>.<posição no projeto>` com projetos (o Geral é o último), senão `<blueprint>.<posição>`.
/// A folha conta todos de uma vez (`numeros` em folha/blueprint.rs, conferido contra esta nos testes); esta fica como a regra.
#[cfg_attr(not(test), allow(dead_code))]
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

#[cfg_attr(not(test), allow(dead_code))]
fn projeto_da_json(p: &Value, a: &Value) -> String {
    let id = texto(a, "projetoId");
    if lista(p, "projetos").iter().any(|x| texto(x, "id") == id) { id.into() } else { String::new() }
}

/// O blueprint está concluído: tem ações e todas concluídas (o status efetivo).
pub fn concluido(p: &Value) -> bool {
    let fs = lista(p, "acoes");
    !fs.is_empty() && fs.iter().all(|f| efetiva(f).0 == Situacao::Concluido)
}

pub struct Resumo {
    /// 0 a 1; só chega a 1 quando toda ação está concluída
    pub geral: f32,
    pub concl: usize,
    /// todas as ações ainda planejadas
    pub futuro: bool,
}

pub fn resumo<'a>(fs: impl IntoIterator<Item = &'a Acao>) -> Resumo {
    let fs: Vec<&Acao> = fs.into_iter().collect();
    let todas = !fs.is_empty() && fs.iter().all(|f| f.status == Situacao::Concluido);
    let media = if fs.is_empty() { 0.0 } else { fs.iter().map(|f| f.pct).sum::<f32>() / fs.len() as f32 };
    Resumo { geral: if todas { 1.0 } else { media.min(0.99) }, concl: fs.iter().filter(|f| f.status == Situacao::Concluido).count(), futuro: !fs.is_empty() && fs.iter().all(|f| f.status == Situacao::Planejado) }
}

/// O `resumo` das ações do blueprint `p` e quantas são, direto do JSON: as mesmas contas, na mesma ordem, sem montar as ações nem copiar
/// texto. A lista pede isto de todo blueprint a cada quadro.
pub fn resumo_json(p: &Value) -> (Resumo, usize) {
    let fs = lista(p, "acoes");
    let (mut soma, mut concl, mut planejadas) = (0.0f32, 0, 0);
    for f in fs {
        let (status, pct, _) = efetiva(f);
        soma += pct;
        concl += usize::from(status == Situacao::Concluido);
        planejadas += usize::from(status == Situacao::Planejado);
    }
    let n = fs.len();
    let media = if n == 0 { 0.0 } else { soma / n as f32 };
    let todas = n > 0 && concl == n;
    (Resumo { geral: if todas { 1.0 } else { media.min(0.99) }, concl, futuro: n > 0 && planejadas == n }, n)
}

/// O progresso de um blueprint (`progressoDe`): 1 se concluído, senão o geral. É o `geral` do resumo: «concluído» é ter ações e todas
/// concluídas, a mesma condição que leva o geral a 1.
pub fn progresso(p: &Value) -> f32 {
    resumo_json(p).0.geral
}

/// Itens que ainda faltam nas ações (a soma de `it[1] - it[0]`).
pub fn faltam(fs: &[Acao]) -> usize {
    fs.iter().map(|f| f.itens.1 - f.itens.0).sum()
}

/// Os passos da folha: cada ação ocupa um, na ordem do blueprint, e a seguinte começa onde ela termina (a escada).
pub struct Janela {
    pub t0: i64,
    pub t1: i64,
}

pub fn janela(fs: &[Acao]) -> Janela {
    Janela { t0: fs.iter().map(|f| f.idx).min().unwrap_or(0) as i64, t1: fs.iter().map(|f| f.idx + 1).max().unwrap_or(1) as i64 }
}

/// Só as ações que ainda falta fazer e, nelas, só os planos e checkpoints pendentes (o filtro «Pendentes» de `acoesVisiveis`).
pub fn so_pendentes(fs: &[Acao]) -> Vec<Acao> {
    fs.iter().filter(|f| f.status != Situacao::Concluido).map(|f| Acao { planos: f.planos.iter().filter(|t| !t.feito).cloned().collect(), checkpoints: f.checkpoints.iter().filter(|m| !m.feito).cloned().collect(), ..f.clone() }).collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn exemplo() -> Value {
        serde_json::from_str(include_str!("../../data.exemplo.json")).unwrap()
    }

    /// Os números que a página mostra para o exemplo: o primeiro blueprint tem 1/3 ações, 44%, 8 itens faltando.
    #[test]
    fn exemplo_bate_com_a_pagina() {
        let d = exemplo();
        let p = &d["blueprints"][0];
        let fs = acoes_de(p);
        let r = resumo(&fs);
        assert_eq!((r.concl, fs.len(), (r.geral * 100.0).round() as i32, faltam(&fs)), (1, 3, 44, 8));
        let q = &d["blueprints"][1];
        let (fq, rq) = (acoes_de(q), resumo(&acoes_de(q)));
        assert_eq!((rq.concl, fq.len(), (rq.geral * 100.0).round() as i32, faltam(&fq)), (0, 2, 17, 5));
        // o carimbo da folha: média dos blueprints (31%)
        let media = d["blueprints"].as_array().unwrap().iter().map(progresso).sum::<f32>() / 2.0;
        assert_eq!((media * 100.0).round() as i32, 31);
    }

    /// As seções vêm pela data de início (Piloto 05/10 antes de Entregador 28/10, mesmo criado depois) e o Geral por último; a ação 3 é a `2.1`.
    #[test]
    fn projetos_ficam_na_ordem_de_criacao_e_o_geral_vai_por_ultimo() {
        let p = serde_json::json!({ "projetos": [{ "id": "A", "nome": "Piloto" }, { "id": "B", "nome": "Entregador" }], "acoes": [
            { "id": "a1", "nome": "x", "projetoId": "A" },
            { "id": "a2", "nome": "y", "projetoId": "A" },
            { "id": "a3", "nome": "z", "projetoId": "B" },
            { "id": "a4", "nome": "w", "projetoId": "inexistente" } ] });
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
    fn o_resumo_do_json_bate_com_o_das_acoes() {
        let d = exemplo();
        let pronto = serde_json::json!({ "acoes": [{ "status": "concluido" }, { "planos": [{ "feito": true }] }] });
        let vazio = serde_json::json!({ "acoes": [] });
        for p in d["blueprints"].as_array().unwrap().iter().chain([&pronto, &vazio]) {
            let (fs, (r, n)) = (acoes_de(p), resumo_json(p));
            let esperado = resumo(&fs);
            assert_eq!((r.geral, r.concl, r.futuro, n), (esperado.geral, esperado.concl, esperado.futuro, fs.len()));
            assert_eq!(progresso(p), if concluido(p) { 1.0 } else { esperado.geral });
        }
    }

    #[test]
    fn pendentes_tira_o_que_esta_feito() {
        let d = exemplo();
        let fs = acoes_de(&d["blueprints"][0]);
        let pend = so_pendentes(&fs);
        // a ação concluída sai; na API só restam os 3 planos e o checkpoint por fazer, e no entregador tudo
        assert_eq!(pend.len(), 2);
        assert_eq!(pend[0].planos.len(), 3);
        assert!(pend.iter().all(|f| f.planos.iter().all(|t| !t.feito) && f.checkpoints.iter().all(|m| !m.feito)));
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
    fn a_janela_tem_um_passo_por_acao() {
        let fs = acoes_de(&exemplo()["blueprints"][0]);
        let j = janela(&fs);
        assert_eq!((j.t0, j.t1), (0, fs.len() as i64));
        assert_eq!((janela(&[]).t0, janela(&[]).t1), (0, 1));
    }
}
