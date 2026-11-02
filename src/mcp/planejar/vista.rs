//! Passa de 300 linhas: a folha é um JSON só, e cada pedaço precisa do mesmo `Foto`; a fatia dona (F32) não pode criar módulo novo. Candidato a dividir (`vista/`) depois das ondas paralelas.
//! A folha como a interface a lê (contrato 5.2): tudo já calculado no servidor, um só caminho de desenho, com ou sem versão. Só lê; a base do diff é sempre a árvore (D1).
//! Os `hash` vêm de `revisao.rs`: são o que `gesto/revisar` confere, então a tela e o clique não podem calcular diferente.
use super::git;
use super::revisao::{acao_do, doc_da_versao, hash_decisao, hash_detalhe, hash_diff};
use crate::blueprint::verdade;
use crate::contexto::Contexto;
use crate::dados::planejar::{ExecucaoLinha, RevisaoLinha, VersaoLinha, SITUACOES_VIVAS};
use crate::dados::Dados;
use crate::erro::Resultado;
use crate::http::pedido::{falha, ErroHttp};
use crate::mcp::acoes::localizar;
use crate::mcp::Mcp;
use crate::plano::arvore::{do_blueprint, hash_peca};
use crate::plano::comparar::{comparar_acao, comparar_planos, planos_travados, CompAcao, CompPlano, Pedaco, TipoLinha};
use crate::plano::entrada::{entrada_esc, ExecEsc};
use crate::plano::escalonar::{escalonar, Causa, EstadoEsc, PlanoEsc, Situacao};
use crate::plano::formato::{AcaoDoc, Detalhe, PlanoDoc, PlanoItem};
use crate::plano::ids::{id_real, nome_agente};
use crate::tempo::hoje_local;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Escrita sem novidade há mais que isto está parada: a tela oferece descartar.
const PARADA_MS: i64 = 5 * 60_000;
const LIMITE_PADRAO: usize = 4;

pub(super) fn lista<'a>(v: &'a Value, campo: &str) -> &'a [Value] {
    v.get(campo).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

pub(super) fn texto<'a>(v: &'a Value, campo: &str) -> &'a str {
    v.get(campo).and_then(Value::as_str).unwrap_or("")
}

/// O `ref` do item; blueprint antigo não tem, e o ref é o próprio id.
pub(super) fn ref_de(v: &Value) -> &str {
    Some(texto(v, "ref")).filter(|r| !r.is_empty()).unwrap_or_else(|| texto(v, "id"))
}

/// A chave `<bp>|<ref>` como a pessoa a lê.
pub(super) fn curto(chave: &str) -> &str {
    chave.rsplit_once('|').map_or(chave, |(_, r)| r)
}

/// O workspace com o campo `blueprints`: "sm" é o próprio arquivo.
pub(super) fn ws_de<'a>(tudo: &'a Value, id: &str) -> Option<&'a Value> {
    if id == "sm" {
        return Some(tudo);
    }
    lista(tudo, "workspaces").iter().find(|w| texto(w, "id") == id)
}

pub(super) fn limite_de(d: &Dados, ws: &str) -> Resultado<usize> {
    Ok(d.config_ler(&format!("limite:{ws}"))?.and_then(|v| v.parse().ok()).unwrap_or(LIMITE_PADRAO))
}

/// A execução que a tela mostra para o plano: a viva ou, sem ela, a última pronta ainda não integrada.
pub(super) fn exec_do_plano<'a>(execs: &'a [ExecucaoLinha], bp: &str, plano_id: &str) -> Option<&'a ExecucaoLinha> {
    let do_plano = || execs.iter().filter(|x| x.blueprint_id == bp && x.plano_id == plano_id);
    do_plano().find(|x| SITUACOES_VIVAS.contains(&x.situacao.as_str())).or_else(|| do_plano().filter(|x| x.situacao == "pronta" && x.integrada_em.is_none()).last())
}

pub(super) fn nome_situacao(s: Situacao) -> &'static str {
    match s {
        Situacao::Nao => "nao",
        Situacao::Revisao => "revisao",
        Situacao::Trabalhando => "trabalhando",
        Situacao::Fila => "fila",
        Situacao::Dependencia => "dependencia",
        Situacao::Area => "area",
        Situacao::Parado => "parado",
        Situacao::Bloqueado => "bloqueado",
        Situacao::Pronto => "pronto",
    }
}

pub(super) fn nome_causa(c: Option<&Causa>) -> &'static str {
    match c {
        Some(Causa::Ajuda) => "ajuda",
        Some(Causa::Sumiu) => "sumiu",
        Some(Causa::Falhou(_)) => "falhou",
        None => "",
    }
}

/// O escalonamento do workspace inteiro, com as mesmas regras do motor.
pub(super) struct Escalonado {
    pub planos: Vec<PlanoEsc>,
    pub estados: BTreeMap<String, EstadoEsc>,
    pub execs: Vec<ExecucaoLinha>,
    pub limite: usize,
}

pub(super) fn escalonado(d: &Dados, ws_id: &str, ws: &Value, agora_ms: i64) -> Result<Escalonado, ErroHttp> {
    let execs = d.execucoes_do_ws(ws_id)?;
    let esc: Vec<ExecEsc> = execs.iter().map(|x| ExecEsc { blueprint: x.blueprint_id.clone(), plano_id: x.plano_id.clone(), plano_ref: x.plano_ref.clone(), situacao: x.situacao.clone(), motivo: x.motivo.clone(), area: x.area.clone(), ramo: x.ramo.clone(), terminada_em: x.terminada_em, cancelada_em: x.cancelada_em }).collect();
    let travados = travados_do_ws(d, ws)?;
    let limite = limite_de(d, ws_id)?;
    let pasta_de = |bp: &str| d.pasta_real_do_blueprint(ws_id, bp).ok().flatten();
    let (planos, entrada) = entrada_esc(ws, &esc, &travados, limite, &|p| git::tem_git(Path::new(p)), &pasta_de, agora_ms);
    let estados = escalonar(&planos, &entrada);
    Ok(Escalonado { planos, estados, execs, limite })
}

/// Em chaves: os planos que a versão pendente de cada blueprint alterou, menos o que o usuário já resolveu nela (aprovar leva à árvore; reprovar não, por isso a revisão conta). O motor usa esta mesma função: a tela e ele não divergem.
pub(super) fn travados_do_ws(d: &Dados, ws: &Value) -> Resultado<Vec<String>> {
    let mut saida = Vec::new();
    for bp in lista(ws, "blueprints") {
        let bp_id = texto(bp, "id");
        let Some(versao) = d.plano_pendente(bp_id)? else { continue };
        let Ok(doc) = doc_da_versao(d, &versao) else { continue };
        let mut comp = comparar_planos(Some(&do_blueprint(bp)), &doc);
        let revisoes = d.revisoes_do(bp_id)?;
        comp.alteradas.retain(|r, _| !revisoes.iter().any(|x| x.numero == versao.numero && x.peca == format!("diff:{r}") && matches!(x.estado.as_str(), "aprovada" | "reprovada")));
        saida.extend(planos_travados(&comp).into_iter().map(|r| format!("{bp_id}|{r}")));
    }
    Ok(saida)
}

fn acoes_de(doc: &PlanoDoc) -> impl Iterator<Item = &AcaoDoc> {
    doc.projetos.iter().flat_map(|p| p.acoes.iter())
}

fn acao_crua<'a>(bp: &'a Value, r: &str) -> Option<&'a Value> {
    lista(bp, "acoes").iter().find(|a| ref_de(a) == r)
}

fn rev_json(r: Option<&RevisaoLinha>, hash: &str) -> Value {
    match r.filter(|x| x.hash_peca == hash) {
        Some(x) => json!({"estado": x.estado, "observacao": x.observacao, "em": x.em}),
        None => Value::Null,
    }
}

fn figura_json(det: &Detalhe) -> Value {
    det.figura_info.as_ref().map_or(Value::Null, |i| json!({"hash": i.hash, "tipo": i.tipo, "largura": i.largura, "altura": i.altura}))
}

fn det_json(det: &Detalhe) -> Value {
    json!({"id": det.id, "titulo": det.titulo.clone().unwrap_or_default(), "legenda": det.legenda.clone().unwrap_or_default(), "grupo": det.grupo.clone().unwrap_or_default(), "figura": figura_json(det), "pedeAprovacao": det.pede_aprovacao, "hash": hash_detalhe(det), "revisao": Value::Null})
}

fn pedacos(p: &[Pedaco]) -> Value {
    Value::Array(p.iter().map(|x| match x {
        Pedaco::Igual(s) => json!({"t": "igual", "s": s}),
        Pedaco::Menos(s) => json!({"t": "menos", "s": s}),
        Pedaco::Mais(s) => json!({"t": "mais", "s": s}),
    }).collect())
}

fn prazo(a: &AcaoDoc) -> String {
    [a.de.as_deref(), a.ate.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" → ")
}

/// A linha do diff. `reabre`: plano feito cujo texto, instruções ou área mudou volta a não feito ao aprovar (5.3); dependência não reabre.
fn linha_json(l: &crate::plano::comparar::Linha, arvore: Option<&Value>) -> Value {
    let tipo = match l.tipo {
        TipoLinha::Prazo => "prazo",
        TipoLinha::Novo => "novo",
        TipoLinha::Alterado => "alterado",
        TipoLinha::Igual => "igual",
        TipoLinha::Removido => "removido",
    };
    let r = l.plano.as_ref().map_or("", |q| q.id.as_str());
    let feito = arvore.is_some_and(|a| lista(a, "planos").iter().any(|q| ref_de(q) == r && verdade(q.get("feito"))));
    let muda_o_feito = !l.palavras.is_empty() || l.campos.iter().any(|c| *c == "area" || *c == "instrucoes");
    let situacao = if l.tipo == TipoLinha::Alterado && feito && muda_o_feito { "reabre" } else { "" };
    json!({"tipo": tipo, "ref": r, "texto": l.plano.as_ref().map_or("", |q| q.texto.as_str()), "antes": l.antes.as_ref().map_or("", |q| q.texto.as_str()), "palavras": pedacos(&l.palavras), "campos": l.campos, "situacao": situacao})
}

/// O DIFF da ação; a revisão entra depois, por quem sabe qual vale. `nova` `None`: a ação sai da árvore se aprovada.
fn diff_json(versao: &VersaoLinha, antiga: &AcaoDoc, nova: Option<&AcaoDoc>, c: &CompAcao, arvore: Option<&Value>) -> Value {
    let tem_prazo = c.linhas.iter().any(|l| l.tipo == TipoLinha::Prazo);
    let prazo_json = if tem_prazo { json!({"antes": prazo(antiga), "depois": nova.map(prazo).unwrap_or_default()}) } else { Value::Null };
    json!({"numero": versao.numero, "agente": versao.agente, "hash": hash_diff(Some(antiga), nova), "resumo": {"mais": c.resumo.mais, "menos": c.resumo.menos, "alterados": c.resumo.alterados, "detalhes": c.resumo.detalhes, "outros": c.resumo.outros}, "removida": nova.is_none(), "prazo": prazo_json, "linhas": c.linhas.iter().map(|l| linha_json(l, arvore)).collect::<Vec<_>>(), "detalhesNovos": c.detalhes_novos.iter().map(det_json).collect::<Vec<_>>(), "outros": c.outros, "revisao": Value::Null})
}

/// O que a montagem da folha consulta o tempo todo.
struct Foto<'a> {
    d: &'a Dados,
    bp: &'a Value,
    bp_id: &'a str,
    base: PlanoDoc,
    revisoes: HashMap<String, RevisaoLinha>,
    esc: Escalonado,
    aberta: Option<VersaoLinha>,
    pendente: Option<VersaoLinha>,
    doc_pendente: Option<PlanoDoc>,
    comp: Option<CompPlano>,
    recados: Vec<crate::dados::planejar::RecadoLinha>,
}

impl Foto<'_> {
    /// A revisão que ainda vale para a peça; `null` se não há ou se a peça mudou desde então.
    fn revisao(&self, peca: &str, hash: &str) -> Value {
        rev_json(self.revisoes.get(peca), hash)
    }

    fn det(&self, acao: &str, x: &Detalhe) -> Value {
        let mut v = det_json(x);
        v["revisao"] = self.revisao(&format!("det:{acao}/{}", x.id), v["hash"].as_str().unwrap_or(""));
        v
    }

    fn diff(&self, r: &str, arvore: Option<&Value>) -> Value {
        let (Some(versao), Some(doc), Some(comp)) = (&self.pendente, &self.doc_pendente, &self.comp) else { return Value::Null };
        let Some(antiga) = acao_do(&self.base, r) else { return Value::Null };
        let nova = acao_do(doc, r);
        let mut v = match (comp.alteradas.get(r), nova) {
            (Some(c), Some(_)) => diff_json(versao, antiga, nova, c, arvore),
            (_, None) => diff_json(versao, antiga, None, &comparar_acao(antiga, &AcaoDoc { id: antiga.id.clone(), nome: antiga.nome.clone(), de: antiga.de.clone(), ate: antiga.ate.clone(), ..AcaoDoc::default() }), arvore),
            _ => return Value::Null,
        };
        v["revisao"] = self.revisao(&format!("diff:{r}"), v["hash"].as_str().unwrap_or(""));
        v
    }

    fn estado_escrita(&self, r: &str) -> &'static str {
        match &self.aberta {
            Some(e) if e.aguardando.iter().any(|x| x == r) => "aguardando",
            Some(e) if e.tocadas.last().is_some_and(|x| x == r) => "escrevendo",
            _ => "pronta",
        }
    }

    fn plano(&self, a: &AcaoDoc, q: &PlanoItem, arvore: Option<&Value>) -> (Value, bool) {
        let cru = arvore.and_then(|c| lista(c, "planos").iter().find(|x| ref_de(x) == q.id));
        let id = cru.map_or_else(|| id_real(self.bp_id, "plano", &format!("{}/{}", a.id, q.id)), |x| texto(x, "id").to_string());
        let feito = cru.is_some_and(|x| verdade(x.get("feito")));
        let estado = self.esc.estados.get(&format!("{}|{}", self.bp_id, q.id));
        let exec = exec_do_plano(&self.esc.execs, self.bp_id, &id);
        // O que ainda não entrou na árvore não pode começar: a caixinha fica tracejada, sem clique.
        let situacao = match (estado, cru) {
            (Some(e), _) => nome_situacao(e.situacao),
            (None, Some(_)) => "nao",
            (None, None) => "revisao",
        };
        let agente = exec.map_or_else(|| cru.map_or("", |x| texto(x, "agente")).to_string(), |x| nome_agente(&x.plano_ref));
        let sem_iniciar: Vec<&str> = estado.map_or(Vec::new(), |e| e.sem_iniciar.iter().map(|s| curto(s)).collect());
        let v = json!({"id": id, "ref": q.id, "texto": q.texto, "instrucoes": q.instrucoes.clone().unwrap_or_default(), "area": q.area.clone().unwrap_or_default(), "dependeDe": q.depende_de, "feito": feito, "situacao": situacao, "causa": nome_causa(estado.and_then(|e| e.causa.as_ref())), "por": estado.and_then(|e| e.por.as_deref()).map_or("", curto), "semIniciar": sem_iniciar, "execucaoId": exec.map_or("", |x| x.id.as_str()), "ramo": exec.map_or("", |x| x.ramo.as_str()), "agente": agente, "motivo": exec.map_or("", |x| x.motivo.as_str())});
        (v, feito)
    }

    fn acao(&self, a: &AcaoDoc) -> Value {
        let arvore = acao_crua(self.bp, &a.id);
        let id = arvore.map_or_else(|| id_real(self.bp_id, "acao", &a.id), |x| texto(x, "id").to_string());
        let mut prontos = 0;
        let planos: Vec<Value> = a.planos.iter().map(|q| {
            let (v, feito) = self.plano(a, q, arvore);
            prontos += feito as usize;
            v
        }).collect();
        let checkpoints: Vec<Value> = a.checkpoints.iter().map(|k| {
            let cru = arvore.and_then(|c| lista(c, "checkpoints").iter().find(|x| ref_de(x) == k.id));
            let id = cru.map_or_else(|| id_real(self.bp_id, "checkpoint", &format!("{}/{}", a.id, k.id)), |x| texto(x, "id").to_string());
            json!({"id": id, "nome": k.texto, "data": k.data.clone().unwrap_or_default(), "feito": cru.is_some_and(|x| verdade(x.get("feito")))})
        }).collect();
        let detalhes: Vec<Value> = a.detalhes.iter().map(|x| self.det(&a.id, x)).collect();
        let diff = self.diff(&a.id, arvore);
        let decisoes: Vec<_> = self.base.decisoes.iter().filter(|x| x.acoes.contains(&a.id)).collect();
        let espera_dec = decisoes.iter().filter(|x| x.pede_aprovacao && self.revisao(&format!("d:{}", x.id), &hash_decisao(x)).is_null()).count();
        let espera_det = detalhes.iter().filter(|x| x["pedeAprovacao"] == true && x["revisao"].is_null()).count();
        let espera_diff = (!diff.is_null() && diff["revisao"].is_null()) as usize;
        let recados: Vec<Value> = self.recados.iter().filter(|r| r.blueprint_id == self.bp_id && (r.acao_id == id || r.acao_id == a.id)).map(|r| json!({"de": r.de, "texto": r.texto, "em": r.em})).collect();
        let agentes = self.esc.execs.iter().filter(|x| x.blueprint_id == self.bp_id && x.acao_id == id && matches!(x.situacao.as_str(), "preparando" | "rodando" | "ajuda")).count();
        let links = arvore.and_then(|c| c.get("links")).cloned().unwrap_or_else(|| json!([]));
        json!({"id": id, "ref": a.id, "nome": a.nome, "de": a.de.clone().unwrap_or_default(), "ate": a.ate.clone().unwrap_or_default(), "semData": a.de.is_none() && a.ate.is_none(), "texto": a.texto.clone().unwrap_or_default(), "status": arvore.map_or("planejado", |c| texto(c, "status")), "links": links, "temNota": arvore.is_some_and(|c| !texto(c, "obs").is_empty()), "escrita": self.estado_escrita(&a.id), "nova": arvore.is_none(), "agentes": agentes, "planos": planos, "checkpoints": checkpoints, "tabelas": a.tabelas.iter().map(|t| t.para_json()).collect::<Vec<_>>(), "detalhes": detalhes, "diff": diff, "recados": recados, "numeros": {"planos": a.planos.len(), "prontos": prontos, "detalhes": a.detalhes.len(), "decisoes": decisoes.len(), "aprovar": espera_dec + espera_det + espera_diff}})
    }
}

/// Os projetos da árvore, com as ações que só existem na versão aberta ou pendente acrescentadas no fim do projeto de mesmo nome.
fn projetos_json(f: &Foto) -> Vec<Value> {
    let id_do_projeto = |nome: &Option<String>| lista(f.bp, "projetos").iter().find(|p| nome.as_deref().is_some_and(|n| texto(p, "nome").trim().to_lowercase() == n.trim().to_lowercase())).map_or("", |p| texto(p, "id"));
    let mut saida: Vec<(Option<String>, Vec<Value>)> = f.base.projetos.iter().map(|p| (p.nome.clone(), p.acoes.iter().map(|a| f.acao(a)).collect())).collect();
    let mut vistas: Vec<String> = acoes_de(&f.base).map(|a| a.id.clone()).collect();
    for versao in [&f.aberta, &f.pendente].into_iter().flatten() {
        let Ok(doc) = doc_da_versao(f.d, versao) else { continue };
        for p in &doc.projetos {
            for a in p.acoes.iter().filter(|a| !vistas.contains(&a.id)).collect::<Vec<_>>() {
                let v = f.acao(a);
                match saida.iter_mut().find(|(nome, _)| *nome == p.nome) {
                    Some((_, acoes)) => acoes.push(v),
                    None => saida.push((p.nome.clone(), vec![v])),
                }
                vistas.push(a.id.clone());
            }
        }
    }
    saida.into_iter().map(|(nome, acoes)| json!({"id": id_do_projeto(&nome), "nome": nome.unwrap_or_default(), "acoes": acoes})).collect()
}

/// `escrita` da folha e de `plano/ler`.
pub(super) fn escrita_json(v: &VersaoLinha, agora_ms: i64) -> Value {
    let parada_ms = (agora_ms - v.atualizada_em).max(0);
    json!({"escritaId": v.escrita_id, "agente": v.agente, "origem": v.origem, "paradaMs": parada_ms, "parada": parada_ms > PARADA_MS})
}

pub(super) fn pendente_json(v: Option<&VersaoLinha>) -> Value {
    v.map_or(Value::Null, |p| json!({"numero": p.numero, "agente": p.agente}))
}

/// Toda a folha de um blueprint, com ou sem versão.
pub(super) fn folha(mcp: &Mcp, d: &mut Dados, ws_id: &str, bp_id: &str, agora_ms: i64) -> Result<Value, ErroHttp> {
    let tudo = d.ler_tudo(&Contexto::local())?.clone();
    let d: &Dados = d;
    let (_, bp) = localizar(&tudo, ws_id, bp_id)?;
    let bp = bp.ok_or_else(|| falha(400, crate::tr!("Send blueprint_id.", "Informe o blueprint_id.")).codigo("falta-blueprint"))?;
    let ws = ws_de(&tudo, ws_id).unwrap_or(&Value::Null);
    let base = do_blueprint(bp);
    let (aberta, pendente) = (d.plano_aberta(bp_id)?, d.plano_pendente(bp_id)?);
    let doc_pendente = pendente.as_ref().and_then(|v| doc_da_versao(d, v).ok());
    let comp = doc_pendente.as_ref().map(|doc| comparar_planos(Some(&base), doc));
    let revisoes = d.revisoes_do(bp_id)?.into_iter().map(|r| (r.peca.clone(), r)).collect();
    let foto = Foto { d, bp, bp_id, base, revisoes, esc: escalonado(d, ws_id, ws, agora_ms)?, aberta, pendente, doc_pendente, comp, recados: d.recados_do(ws_id, 200)? };

    let projetos = projetos_json(&foto);
    let decisoes: Vec<Value> = foto.base.decisoes.iter().map(|x| {
        let hash = hash_decisao(x);
        json!({"id": x.id, "rotulo": x.rotulo.clone().unwrap_or_default(), "texto": x.texto, "data": x.data.clone().unwrap_or_default(), "acoes": x.acoes, "pedeAprovacao": x.pede_aprovacao, "revisao": foto.revisao(&format!("d:{}", x.id), &hash), "hash": hash})
    }).collect();
    let perguntas: Vec<Value> = foto.base.perguntas.iter().map(|q| {
        let resposta = foto.revisoes.get(&format!("q:{}", q.id)).filter(|r| r.estado == "respondida" && r.hash_peca == hash_peca(&q.para_json())).map_or("", |r| r.observacao.as_str());
        json!({"id": q.id, "texto": q.texto, "opcoes": q.opcoes, "recomendada": q.recomendada.clone().unwrap_or_default(), "acoes": q.acoes, "resposta": resposta})
    }).collect();
    let (mut diffs, mut sem_revisao, mut dets) = (0, 0, 0);
    for a in projetos.iter().flat_map(|p| lista(p, "acoes")) {
        if !a["diff"].is_null() {
            diffs += 1;
            sem_revisao += a["diff"]["revisao"].is_null() as usize;
        }
        dets += lista(a, "detalhes").iter().filter(|x| x["pedeAprovacao"] == true && x["revisao"].is_null()).count();
    }
    let dec = decisoes.iter().filter(|x| x["pedeAprovacao"] == true && x["revisao"].is_null()).count();
    let novas = foto.comp.as_ref().map_or(0, |c| c.novas.len());
    Ok(json!({"versaoDados": mcp.versao_espelho.load(std::sync::atomic::Ordering::SeqCst), "revisao": mcp.revisao_planejar(), "hoje": hoje_local(),
        "blueprint": {"id": bp_id, "nome": texto(bp, "nome"), "resumo": texto(bp, "descricao"), "links": bp.get("links").cloned().unwrap_or_else(|| json!([])), "temNota": !texto(bp, "obs").is_empty()},
        "escrita": foto.aberta.as_ref().map_or(Value::Null, |v| escrita_json(v, agora_ms)), "pendente": pendente_json(foto.pendente.as_ref()),
        "semPasta": d.pasta_real_do_blueprint(ws_id, bp_id)?.is_none(), "limite": foto.esc.limite,
        "projetos": projetos, "decisoes": decisoes, "perguntas": perguntas, "numeros": {"aprovar": dec + dets + sem_revisao, "diffs": diffs, "novas": novas}}))
}
