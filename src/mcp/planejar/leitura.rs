//! O que se lê do Planejar (contrato 5.1 e 5.2): nada aqui muda estado, nem marca nada como lido, então vários leitores não se atrapalham. O `segredo` da execução e o token do gesto nunca saem por aqui.
//! Git só depois de soltar o `Mutex<Dados>`: `destino` abre processo, e o resto do app não pode esperar por ele.
use super::git;
use super::revisao::doc_da_versao;
use super::vista::{curto, escalonado, escrita_json, exec_do_plano, folha, lista, nome_causa, nome_situacao, pendente_json, ref_de, texto, ws_de, Escalonado};
use crate::blueprint::resumo_blueprint;
use crate::contexto::Contexto;
use crate::dados::planejar::{ExecucaoLinha, RecadoLinha, SITUACOES_VIVAS};
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use crate::mcp::acoes::localizar;
use crate::mcp::Mcp;
use crate::plano::arvore::do_blueprint;
use crate::plano::comparar::comparar_planos;
use crate::plano::escalonar::{EstadoEsc, Situacao};
use crate::plano::formato::AcaoDoc;
use crate::plano::frases;
use crate::tempo::{agora_ms, hoje_local};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, MutexGuard};

/// `ajuda` sem sinal há mais que isto: o agente pode ter morrido enquanto esperava resposta. Só avisa, não muda a situação.
const SEM_SINAL_MS: i64 = 120_000;
/// Quantos recados do orquestrador entram no "Agora".
const RECADOS_NO_AGORA: usize = 3;
const RECADOS_NO_WORKSPACE: usize = 20;

fn trava(dados: &Mutex<Dados>) -> MutexGuard<'_, Dados> {
    dados.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn rota(mcp: &Mcp, dados: &Mutex<Dados>, pedido: &Pedido, recurso: &str, _corpo: &Value) -> Result<Resposta, ErroHttp> {
    let p = |nome: &str| pedido.parametro(nome).unwrap_or_default();
    let (ws, bp) = (p("workspace"), p("blueprint"));
    let corpo = match recurso {
        "planejar/pulso" => json!({"versaoDados": mcp.versao_espelho.load(Ordering::SeqCst), "revisao": mcp.revisao_planejar()}),
        "planejar/folha" => folha(mcp, &mut trava(dados), &ws, &bp, agora_ms())?,
        "planejar/chegou" => chegou(&mut trava(dados), &ws, &bp)?,
        "planejar/agentes" => agentes(mcp, dados, &ws)?,
        "planejar/config" => config(&trava(dados))?,
        "plano/ler" => plano_ler(&mut trava(dados), &ws, &bp)?,
        "plano/respostas" => respostas(&mut trava(dados), &ws, &bp, &p("execucao"))?,
        "plano/workspace" => workspace(mcp, dados, &ws)?,
        _ => return Err(super::nao_encontrado()),
    };
    Ok(Resposta::json(200, &corpo))
}

/// A árvore inteira e o blueprint pedido. Clona: quem chama ainda consulta as outras tabelas.
fn com_blueprint(d: &mut Dados, ws: &str, bp: &str) -> Result<(Value, Value), ErroHttp> {
    let tudo = d.ler_tudo(&Contexto::local())?.clone();
    let achado = localizar(&tudo, ws, bp)?.1.cloned().ok_or_else(|| falha(400, crate::tr!("Send blueprint_id.", "Informe o blueprint_id.")).codigo("falta-blueprint"))?;
    Ok((tudo, achado))
}

fn resumo_de_acao(a: &AcaoDoc) -> Value {
    json!({"ref": a.id, "nome": a.nome, "planos": a.planos.len(), "detalhes": a.detalhes.len(), "checkpoints": a.checkpoints.len()})
}

/// A tela "O que chegou". Só mostra; quem decide é `gesto/revisar` e `gesto/colocar`. Sem versão pendente, tudo vazio.
fn chegou(d: &mut Dados, ws: &str, bp_id: &str) -> Result<Value, ErroHttp> {
    let (_, bp) = com_blueprint(d, ws, bp_id)?;
    let Some(versao) = d.plano_pendente(bp_id)? else {
        return Ok(json!({"numero": 0, "agente": "", "origem": "", "novas": [], "alteradas": [], "removidas": [], "iguais": 0}));
    };
    let doc = doc_da_versao(d, &versao)?;
    let base = do_blueprint(&bp);
    let c = comparar_planos(Some(&base), &doc);
    let de = |d: &'_ crate::plano::formato::PlanoDoc, r: &str| d.projetos.iter().flat_map(|p| p.acoes.iter()).find(|a| a.id == r).cloned();
    let pode_colocar = versao.origem == "salvar";
    let novas: Vec<Value> = c.novas.iter().filter_map(|r| de(&doc, r)).map(|a| {
        let mut v = resumo_de_acao(&a);
        v["podeColocar"] = json!(pode_colocar);
        v
    }).collect();
    let alteradas: Vec<Value> = c.alteradas.iter().map(|(r, x)| {
        let nome = de(&doc, r).map_or_else(String::new, |a| a.nome);
        json!({"ref": r, "nome": nome, "resumo": {"mais": x.resumo.mais, "menos": x.resumo.menos, "alterados": x.resumo.alterados, "detalhes": x.resumo.detalhes, "outros": x.resumo.outros}})
    }).collect();
    let removidas: Vec<Value> = c.removidas.iter().map(|r| json!({"ref": r, "nome": de(&base, r).map_or_else(String::new, |a| a.nome)})).collect();
    Ok(json!({"numero": versao.numero, "agente": versao.agente, "origem": versao.origem, "novas": novas, "alteradas": alteradas, "removidas": removidas, "iguais": c.iguais.len()}))
}

fn config(d: &Dados) -> Result<Value, ErroHttp> {
    let comando = |chave: &str| -> Result<Value, ErroHttp> { Ok(d.config_ler(chave)?.and_then(|v| serde_json::from_str::<Value>(&v).ok()).filter(Value::is_array).unwrap_or_else(|| json!([]))) };
    Ok(json!({"comandoAgente": comando("comando_agente")?, "comandoOrquestrador": comando("comando_orquestrador")?}))
}

fn plano_ler(d: &mut Dados, ws: &str, bp_id: &str) -> Result<Value, ErroHttp> {
    let (_, bp) = com_blueprint(d, ws, bp_id)?;
    let aberta = d.plano_aberta(bp_id)?;
    let aguardando = aberta.as_ref().map_or(Vec::new(), |v| v.aguardando.clone());
    Ok(json!({"plano": do_blueprint(&bp).para_json(), "escrita": aberta.as_ref().map_or(Value::Null, |v| escrita_json(v, agora_ms())), "pendente": pendente_json(d.plano_pendente(bp_id)?.as_ref()), "aguardando": aguardando}))
}

fn recado_json(r: &RecadoLinha) -> Value {
    json!({"id": r.id, "blueprintId": r.blueprint_id, "acaoId": r.acao_id, "execucaoId": r.execucao_id, "de": r.de, "texto": r.texto, "em": r.em})
}

/// Sem efeito: devolve o estado atual, e ler de novo dá o mesmo. Com `execucao`, os recados dela; sem, os últimos do blueprint.
fn respostas(d: &mut Dados, ws: &str, bp_id: &str, execucao: &str) -> Result<Value, ErroHttp> {
    com_blueprint(d, ws, bp_id)?;
    let (mut revisoes, mut lidas) = (Vec::new(), Vec::new());
    for r in d.revisoes_do(bp_id)? {
        match r.peca.strip_prefix("q:") {
            Some(pergunta) => lidas.push(json!({"pergunta": pergunta, "resposta": r.observacao})),
            None => revisoes.push(json!({"peca": r.peca, "estado": r.estado, "observacao": r.observacao, "numero": r.numero})),
        }
    }
    let recados = if execucao.is_empty() {
        d.recados_do(ws, RECADOS_NO_WORKSPACE)?.into_iter().filter(|r| r.blueprint_id.is_empty() || r.blueprint_id == bp_id).collect()
    } else {
        d.recados_para(execucao)?
    };
    Ok(json!({"revisoes": revisoes, "respostas": lidas, "recados": recados.iter().map(recado_json).collect::<Vec<_>>()}))
}

/// O plano e a ação da árvore por id real, em cada blueprint do workspace.
fn indice(ws: &Value) -> HashMap<(&str, &str), (&Value, &Value, &Value)> {
    let mut saida = HashMap::new();
    for bp in lista(ws, "blueprints") {
        for a in lista(bp, "acoes") {
            for q in lista(a, "planos") {
                saida.insert((texto(bp, "id"), texto(q, "id")), (bp, a, q));
            }
        }
    }
    saida
}

fn sem_sinal(x: &ExecucaoLinha, agora: i64) -> bool {
    x.situacao == "ajuda" && x.ultimo_sinal.or(x.iniciada_em).is_some_and(|s| agora - s > SEM_SINAL_MS)
}

/// Quantos subagentes trabalham neste plano: os agentes que avisaram este plano, menos o principal.
fn subagentes(presenca: &Value, bp: &str, plano: &str, principal: &str) -> usize {
    lista(presenca, "agentes").iter().filter(|a| texto(a, "blueprintId") == bp && texto(a, "planoId") == plano && texto(a, "agente") != principal && texto(a, "estado") != "concluido").count()
}

/// Os planos da tela de agentes, em ordem de documento: só com execução viva ou pronta ainda não integrada. O que `destino` pede fica de fora: é git.
fn linhas_de_agentes(mcp: &Mcp, ws: &Value, esc: &Escalonado, agora: i64) -> (Vec<Value>, BTreeMap<String, EstadoEsc>) {
    let arvore = indice(ws);
    let presenca = mcp.agentes();
    let mut achados: Vec<(usize, Value)> = Vec::new();
    let mut estados = BTreeMap::new();
    for x in &esc.execs {
        let viva = SITUACOES_VIVAS.contains(&x.situacao.as_str());
        let achado = arvore.get(&(x.blueprint_id.as_str(), x.plano_id.as_str()));
        // A pronta cujo plano foi apagado não tem o que mostrar; a viva segue na lista até o motor cancelar.
        if !viva && achado.is_none() || exec_do_plano(&esc.execs, &x.blueprint_id, &x.plano_id).map(|e| e.id.as_str()) != Some(x.id.as_str()) {
            continue;
        }
        let r = achado.map_or(x.plano_ref.as_str(), |(_, _, q)| ref_de(q));
        let chave = format!("{}|{r}", x.blueprint_id);
        let estado = esc.estados.get(&chave);
        if let Some(e) = estado {
            estados.insert(chave.clone(), e.clone());
        }
        let (bp_nome, q) = achado.map_or(("", &Value::Null), |(bp, _, q)| (texto(bp, "nome"), *q));
        let area = if viva { x.area.as_str() } else { texto(q, "area") };
        let quieto = sem_sinal(x, agora);
        let linha = json!({"blueprintId": x.blueprint_id, "blueprintNome": bp_nome, "acaoId": x.acao_id, "id": x.plano_id, "ref": r, "texto": texto(q, "texto"),
            "situacao": estado.map_or("nao", |e| nome_situacao(e.situacao)), "causa": nome_causa(estado.and_then(|e| e.causa.as_ref())), "por": estado.and_then(|e| e.por.as_deref()).map_or("", curto),
            "dependeDe": q.get("dependeDe").cloned().unwrap_or_else(|| json!([])), "area": area, "ramo": x.ramo, "destino": "", "execucaoId": x.id, "motivo": x.motivo, "semSinal": quieto,
            "subagentes": subagentes(&presenca, &x.blueprint_id, &x.plano_id, &crate::plano::ids::nome_agente(r)),
            "podeIntegrar": x.situacao == "pronta" && x.integrada_em.is_none() && !x.ramo.is_empty(),
            "podeRetomar": matches!(x.situacao.as_str(), "sumiu" | "falhou") || quieto, "podeCancelar": viva});
        let posicao = esc.planos.iter().position(|q| q.chave == chave).unwrap_or(usize::MAX);
        achados.push((posicao, linha));
    }
    achados.sort_by_key(|(p, _)| *p);
    (achados.into_iter().map(|(_, l)| l).collect(), estados)
}

/// O ramo em que cada pasta principal está, para quem pode integrar. Fora do `Mutex<Dados>`, e uma vez por pasta.
fn preencher_destinos(linhas: &mut [Value], execs: &HashMap<String, String>) {
    let programa = git::programa();
    let mut ramos: HashMap<String, String> = HashMap::new();
    for l in linhas.iter_mut().filter(|l| l["podeIntegrar"] == true) {
        let Some(pasta) = execs.get(texto(l, "execucaoId")) else { continue };
        let ramo = ramos.entry(pasta.clone()).or_insert_with(|| git::ramo_atual(&programa, Path::new(pasta)).unwrap_or_default());
        l["destino"] = json!(ramo);
    }
}

struct LidoDoWorkspace {
    linhas: Vec<Value>,
    avisos: Vec<String>,
    agora: Vec<String>,
    numeros: Value,
    limite: usize,
    ligado: bool,
    acordado: bool,
    recados: Vec<RecadoLinha>,
    blueprints: Vec<Value>,
    pastas: HashMap<String, String>,
}

/// Lê tudo sob a trava, solta, e só então pergunta o ramo ao git.
fn ler_workspace(mcp: &Mcp, dados: &Mutex<Dados>, ws_id: &str) -> Result<LidoDoWorkspace, ErroHttp> {
    let mut lido = {
        let mut d = trava(dados);
        let tudo = d.ler_tudo(&Contexto::local())?.clone();
        localizar(&tudo, ws_id, "")?;
        let ws = ws_de(&tudo, ws_id).unwrap_or(&Value::Null);
        let agora = agora_ms();
        let esc = escalonado(&d, ws_id, ws, agora)?;
        let (linhas, listados) = linhas_de_agentes(mcp, ws, &esc, agora);
        let orq = d.orq_ler(ws_id)?;
        // Mapa só com os listados: o feito à mão, sem agente, não vira "concluído com evidência"; a lista inteira ainda diz qual dependência existe.
        let avisos = frases::avisos(&esc.planos, &listados).iter().map(frases::texto).collect();
        let mut falas: Vec<String> = frases::diario(&esc.planos, &listados).iter().map(|f| frases::texto_com(f, orq.ligado)).collect();
        let recados = d.recados_do(ws_id, RECADOS_NO_WORKSPACE)?;
        let dele: Vec<&RecadoLinha> = recados.iter().filter(|r| r.de == "orquestrador").collect();
        falas.extend(dele[dele.len().saturating_sub(RECADOS_NO_AGORA)..].iter().map(|r| r.texto.clone()));
        let conta = |nomes: &[Situacao]| listados.values().filter(|e| nomes.contains(&e.situacao)).count();
        let numeros = json!({"trabalhando": conta(&[Situacao::Trabalhando]), "fila": conta(&[Situacao::Fila]), "esperando": conta(&[Situacao::Dependencia, Situacao::Area, Situacao::Revisao]), "parados": conta(&[Situacao::Parado, Situacao::Bloqueado]), "prontos": conta(&[Situacao::Pronto])});
        let pastas = esc.execs.iter().map(|x| (x.id.clone(), x.pasta_principal.clone())).collect();
        let blueprints = lista(ws, "blueprints").iter().map(resumo_blueprint).collect();
        LidoDoWorkspace { linhas, avisos, agora: falas, numeros, limite: esc.limite, ligado: orq.ligado, acordado: orq.rodando, recados, blueprints, pastas }
    };
    preencher_destinos(&mut lido.linhas, &lido.pastas);
    Ok(lido)
}

fn agentes(mcp: &Mcp, dados: &Mutex<Dados>, ws: &str) -> Result<Value, ErroHttp> {
    let l = ler_workspace(mcp, dados, ws)?;
    Ok(json!({"limite": l.limite, "orquestrador": {"ligado": l.ligado, "acordado": l.acordado, "agora": l.agora}, "numeros": l.numeros, "avisos": l.avisos, "planos": l.linhas}))
}

fn workspace(mcp: &Mcp, dados: &Mutex<Dados>, ws: &str) -> Result<Value, ErroHttp> {
    let l = ler_workspace(mcp, dados, ws)?;
    Ok(json!({"blueprints": l.blueprints, "agentes": l.linhas, "avisos": l.avisos, "recados": l.recados.iter().map(recado_json).collect::<Vec<_>>(), "hoje": hoje_local()}))
}
