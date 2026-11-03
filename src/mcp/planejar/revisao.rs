//! `gesto/revisar` e `gesto/colocar`: o que o usuário decide sobre o que o agente mandou (contrato, seção 5.3). A base do diff é sempre a árvore (D1): aprovar projeta só aquela ação, reprovar não grava nela.
//! Passa de 300 linhas (teto 500): aprovar, editar e colocar dividem a mesma conferência da projeção, e a fatia é dona só deste arquivo e de `gestos.rs`; o corte natural é a projeção num módulo próprio.
//! O clique vem amarrado a `numero` e `hash` do que estava na tela: o agente pode ter fechado outra versão entre o desenho e o clique. Os `hash_*` daqui são o que a vista tem de mandar à tela.
use crate::contexto::Contexto;
use crate::dados::planejar::VersaoLinha;
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use crate::mcp::acoes::localizar;
use crate::mcp::Mcp;
use crate::plano::arvore::{do_blueprint, hash_peca, projetar, Escopo};
use crate::plano::comparar::{comparar_acao, comparar_planos, CompAcao, TipoLinha};
use crate::plano::formato::{AcaoDoc, Decisao, Detalhe, FiguraInfo, PlanoDoc, PlanoItem};
use crate::plano::frases::{self, Frase};
use crate::plano::validar::achar_ciclo;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard};

pub(super) type R<T> = Result<T, ErroHttp>;

pub(super) fn hash_decisao(d: &Decisao) -> String {
    hash_peca(&d.para_json())
}

pub(super) fn hash_detalhe(d: &Detalhe) -> String {
    hash_peca(&d.para_json())
}

/// A peça `diff:<ref>` é o par antes e depois da ação. Só ela entra: aprovar outra ação da mesma versão não derruba o clique nesta. `nova` `None` = a ação sai.
pub(super) fn hash_diff(antiga: Option<&AcaoDoc>, nova: Option<&AcaoDoc>) -> String {
    hash_peca(&json!({"antes": antiga.map(AcaoDoc::para_json), "depois": nova.map(AcaoDoc::para_json)}))
}

pub(super) fn travar(dados: &Mutex<Dados>) -> MutexGuard<'_, Dados> {
    dados.lock().unwrap_or_else(|e| e.into_inner())
}

pub(super) fn texto<'a>(v: &'a Value, campo: &str) -> &'a str {
    v.get(campo).and_then(Value::as_str).unwrap_or("")
}

pub(super) fn lista<'a>(v: &'a Value, campo: &str) -> &'a [Value] {
    v.get(campo).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn ref_de(v: &Value) -> &str {
    Some(texto(v, "ref")).filter(|r| !r.is_empty()).unwrap_or_else(|| texto(v, "id"))
}

pub(super) fn alvo(corpo: &Value) -> R<(String, String)> {
    let (ws, bp) = (texto(corpo, "workspaceId"), texto(corpo, "blueprintId"));
    if ws.is_empty() || bp.is_empty() {
        return Err(falha(400, crate::tr!("workspaceId and blueprintId are required", "workspaceId e blueprintId são obrigatórios")).codigo("faltam-ids"));
    }
    Ok((ws.to_string(), bp.to_string()))
}

/// O blueprint como está na árvore; ele some na hora de gravar se outro `PUT` o tirar, por isso tudo acontece com o `Mutex` na mão.
pub(super) fn blueprint_de(d: &mut Dados, ws: &str, bp: &str) -> R<Value> {
    let tudo = d.ler_tudo(&Contexto::local())?;
    let (_, achado) = localizar(tudo, ws, bp)?;
    achado.cloned().ok_or_else(|| falha(404, crate::tr!("Blueprint not found", "Blueprint não encontrado")).codigo("blueprint-nao-encontrado"))
}

/// O doc guardado na versão. `de_json` nunca lê `figura_info` (o agente não pode forjá-la): aqui ela volta do que o fechamento gravou ou de `plano_figuras`, senão aprovar apagaria as medidas.
pub(super) fn doc_da_versao(d: &Dados, v: &VersaoLinha) -> R<PlanoDoc> {
    let mut doc = PlanoDoc::de_json(&v.plano).map_err(|e| falha(500, e.join("; ")).codigo("versao-invalida"))?;
    let gravadas: Vec<&Value> = lista(&v.plano, "projetos").iter().flat_map(|p| lista(p, "acoes")).collect();
    for a in doc.projetos.iter_mut().flat_map(|p| p.acoes.iter_mut()) {
        let dela = gravadas.iter().find(|g| texto(g, "id") == a.id);
        for det in &mut a.detalhes {
            let gravado = dela.and_then(|g| lista(g, "detalhes").iter().find(|x| texto(x, "id") == det.id));
            det.figura_info = gravado.and_then(|g| g.get("figura_info")).and_then(FiguraInfo::de_json);
            if let (None, Some(caminho)) = (&det.figura_info, &det.figura) {
                det.figura_info = d.figura_de(&v.blueprint_id, v.numero, caminho)?.map(|f| FiguraInfo { hash: f.hash, tipo: f.tipo, largura: f.largura, altura: f.altura });
            }
        }
    }
    Ok(doc)
}

pub(super) fn acao_do<'a>(doc: &'a PlanoDoc, r: &str) -> Option<&'a AcaoDoc> {
    doc.projetos.iter().flat_map(|p| &p.acoes).find(|a| a.id == r)
}

fn plano_mudou() -> ErroHttp {
    falha(409, crate::tr!("The plan changed while you were looking at it: review it again", "O plano mudou enquanto você olhava: revise de novo")).codigo("plano-mudou")
}

fn valida_estado(estado: &str) -> R<()> {
    if !matches!(estado, "aprovada" | "revisar" | "reprovada" | "pendente") {
        return Err(falha(422, crate::tr!("Unknown review state", "Estado de revisão desconhecido")).codigo("estado-invalido"));
    }
    Ok(())
}

/// Só as ações: nome, resumo, tema, decisões e perguntas já entraram ao fechar, e aprovar uma ação depois não pode desfazer o que o usuário editou neles.
fn so_as_acoes(doc: &PlanoDoc) -> PlanoDoc {
    PlanoDoc { formato: doc.formato.clone(), projetos: doc.projetos.clone(), ..PlanoDoc::default() }
}

/// Os três pontos em que o id da árvore é global por tabela (D3); `gravar_tudo` recusaria com erro de banco.
fn id_repetido(tudo: &Value) -> Option<String> {
    let raiz = lista(tudo, "blueprints").iter();
    let blueprints = raiz.chain(lista(tudo, "workspaces").iter().flat_map(|w| lista(w, "blueprints")));
    let (mut acoes, mut planos, mut checkpoints) = (HashSet::new(), HashSet::new(), HashSet::new());
    for a in blueprints.flat_map(|b| lista(b, "acoes")) {
        let filhos = |campo, vistos: &mut HashSet<String>| lista(a, campo).iter().map(|x| texto(x, "id")).filter(|i| !i.is_empty()).find(|i| !vistos.insert(i.to_string())).map(String::from);
        let repetida = Some(texto(a, "id")).filter(|i| !i.is_empty() && !acoes.insert(i.to_string())).map(String::from);
        if let Some(i) = repetida.or_else(|| filhos("planos", &mut planos)).or_else(|| filhos("checkpoints", &mut checkpoints)) {
            return Some(i);
        }
    }
    None
}

fn trocar_blueprint(tudo: &mut Value, ws: &str, novo: Value) {
    let dono = if ws == "sm" { Some(&mut *tudo) } else { tudo.get_mut("workspaces").and_then(Value::as_array_mut).and_then(|l| l.iter_mut().find(|w| texto(w, "id") == ws)) };
    let lista = dono.and_then(|d| d.get_mut("blueprints")).and_then(Value::as_array_mut);
    if let Some(b) = lista.and_then(|l| l.iter_mut().find(|b| texto(b, "id") == texto(&novo, "id"))) {
        *b = novo;
    }
}

/// Confere a projeção e grava a árvore; só isto sobe a versão dos dados (D4). Dependência que não existe mais só avisa.
fn gravar_projecao(d: &mut Dados, ws: &str, nova: Value) -> R<Vec<String>> {
    let doc = do_blueprint(&nova);
    let planos: Vec<&PlanoItem> = doc.projetos.iter().flat_map(|p| &p.acoes).flat_map(|a| &a.planos).collect();
    if let Some(ciclo) = achar_ciclo(&planos) {
        let caminho = ciclo.join(" > ");
        return Err(falha(409, crate::tr!("This would create a dependency cycle: {caminho}", "Isto criaria um ciclo de dependências: {caminho}")).codigo("ciclo").params(json!({"ciclo": ciclo})));
    }
    let existe: HashSet<&str> = planos.iter().map(|q| q.id.as_str()).collect();
    let bp = texto(&nova, "id");
    let avisos = planos.iter().flat_map(|q| q.depende_de.iter().filter(|dep| !existe.contains(dep.as_str())).map(|dep| frases::texto(&Frase::DependeFalta { id: format!("{bp}|{}", q.id), dep: format!("{bp}|{dep}") }))).collect();

    let mut tudo = d.ler_tudo(&Contexto::local())?.clone();
    trocar_blueprint(&mut tudo, ws, nova);
    if let Some(id) = id_repetido(&tudo) {
        return Err(falha(409, crate::tr!("Two items would get the same id: {id}", "Dois itens ficariam com o mesmo id: {id}")).codigo("id-duplicado"));
    }
    d.gravar_tudo(&Contexto::local(), &tudo)?;
    d.tocar_revisao();
    Ok(avisos)
}

/// Fechou, resolve: nada mais a decidir. O que foi reprovado conta como decidido; sem isso a próxima escrita partiria de uma base com o texto recusado. `revisar` não: espera o agente.
fn resolver_se_acabou(d: &mut Dados, ws: &str, bp: &str, v: &VersaoLinha, doc: &PlanoDoc) -> R<bool> {
    let atual = do_blueprint(&blueprint_de(d, ws, bp)?);
    let c = comparar_planos(Some(&atual), doc);
    let revisoes = d.revisoes_do(bp)?;
    let reprovada = |r: &String| revisoes.iter().any(|x| x.peca == format!("diff:{r}") && x.numero == v.numero && x.estado == "reprovada");
    let novas = if v.origem == "salvar" { c.novas.as_slice() } else { &[] };
    let falta = c.alteradas.keys().chain(&c.removidas).chain(novas).any(|r| !reprovada(r));
    if !falta {
        d.plano_resolver(bp, v.numero)?;
    }
    Ok(!falta)
}

/// Editar: troca os textos no doc antes de aprovar. Só plano desta ação e nunca vazio.
fn aplicar_textos(doc: &mut PlanoDoc, r: &str, textos: Option<&Value>) -> R<()> {
    let Some(textos) = textos.and_then(Value::as_object).filter(|t| !t.is_empty()) else { return Ok(()) };
    let invalido = || falha(422, crate::tr!("The edited text does not match a plan of this action", "O texto editado não corresponde a um plano desta ação")).codigo("texto-invalido");
    let acao = doc.projetos.iter_mut().flat_map(|p| p.acoes.iter_mut()).find(|a| a.id == r).ok_or_else(invalido)?;
    for (id, valor) in textos {
        let novo = valor.as_str().map(str::trim).filter(|s| !s.is_empty()).ok_or_else(invalido)?;
        acao.planos.iter_mut().find(|q| &q.id == id).ok_or_else(invalido)?.texto = novo.to_string();
    }
    Ok(())
}

/// Gravar deixaria o agente no texto antigo e marcando `feito` no novo: com execução viva em plano alterado ou removido, 409 até cancelar ou concluir. `comp` `None` = a ação inteira sai.
fn sem_execucao_nos_mudados(d: &Dados, bp: &str, arvore: &Value, r: &str, comp: Option<&CompAcao>) -> R<()> {
    let Some(acao) = lista(arvore, "acoes").iter().find(|a| ref_de(a) == r) else { return Ok(()) };
    let mudou = |q: &Value| match comp {
        None => true,
        Some(c) => c.linhas.iter().any(|l| matches!(l.tipo, TipoLinha::Alterado | TipoLinha::Removido) && l.plano.as_ref().is_some_and(|p| p.id == ref_de(q))),
    };
    for q in lista(acao, "planos").iter().filter(|q| mudou(q)) {
        if d.execucao_viva(bp, texto(q, "id"))?.is_some() {
            let plano = ref_de(q);
            return Err(falha(409, crate::tr!("Plan {plano} is running: cancel it or wait for it before changing it", "O plano {plano} está em execução: cancele ou espere antes de mudar")).codigo("rodando"));
        }
    }
    Ok(())
}

/// Plano feito cujo texto, instruções ou área mudou volta a não feito: o que foi concluído não é mais o que está escrito. Devolve os refs reabertos.
fn reabrir_feitos(nova: &mut Value, r: &str, comp: &CompAcao) -> Vec<String> {
    let mudados: Vec<&str> = comp.linhas.iter().filter(|l| l.tipo == TipoLinha::Alterado && (!l.palavras.is_empty() || l.campos.iter().any(|c| matches!(*c, "area" | "instrucoes")))).filter_map(|l| l.plano.as_ref().map(|q| q.id.as_str())).collect();
    let mut reabertos = Vec::new();
    let acoes = nova.get_mut("acoes").and_then(Value::as_array_mut).into_iter().flatten();
    let planos = acoes.filter(|a| ref_de(a) == r).filter_map(|a| a.get_mut("planos").and_then(Value::as_array_mut)).flatten();
    for q in planos.filter(|q| q["feito"] == json!(true) && mudados.contains(&ref_de(q))) {
        reabertos.push(ref_de(q).to_string());
        let Some(o) = q.as_object_mut() else { continue };
        o.insert("feito".into(), json!(false));
        o.remove("evidencia");
        if let Some(agente) = o.get_mut("agente").and_then(Value::as_object_mut) {
            agente.insert("estado".into(), json!("pendente"));
        }
    }
    reabertos
}

/// A versão fechada guarda o texto editado: a árvore já tem esse texto e o diff precisa dar igual.
fn regravar_fechada(d: &mut Dados, v: &VersaoLinha, doc: &PlanoDoc) -> R<()> {
    d.plano_regravar_fechada(&v.blueprint_id, v.numero, &doc.para_json())?;
    Ok(())
}

struct Clique<'a> {
    ws: &'a str,
    bp: &'a str,
    peca: &'a str,
    estado: &'a str,
    observacao: &'a str,
    hash: &'a str,
}

fn acordar(d: &mut Dados, c: &Clique) -> R<()> {
    d.orq_pedir(c.ws, &format!("The user marked {} as \"{}\" in blueprint {}", c.peca, c.estado, c.bp), false)?;
    Ok(())
}

fn revisar_diff(d: &mut Dados, c: &Clique, r: &str, numero: i64, textos: Option<&Value>) -> R<Value> {
    let v = d.plano_pendente(c.bp)?.filter(|v| v.numero == numero).ok_or_else(plano_mudou)?;
    let arvore = blueprint_de(d, c.ws, c.bp)?;
    let atual = do_blueprint(&arvore);
    let mut doc = doc_da_versao(d, &v)?;
    let comp = comparar_planos(Some(&atual), &doc);
    if (!comp.alteradas.contains_key(r) && !comp.removidas.iter().any(|x| x == r)) || hash_diff(acao_do(&atual, r), acao_do(&doc, r)) != c.hash {
        return Err(plano_mudou());
    }
    let (mut avisos, mut reabertos) = (Vec::new(), Vec::new());
    match c.estado {
        "aprovada" => {
            aplicar_textos(&mut doc, r, textos)?;
            let editada = acao_do(&atual, r).zip(acao_do(&doc, r)).map(|(a, b)| comparar_acao(a, b));
            sem_execucao_nos_mudados(d, c.bp, &arvore, r, editada.as_ref())?;
            let refs = [r.to_string()];
            let mut nova = projetar(&arvore, &so_as_acoes(&doc), Escopo::So(&refs));
            if let Some(e) = &editada {
                reabertos = reabrir_feitos(&mut nova, r, e);
            }
            avisos = gravar_projecao(d, c.ws, nova)?;
            if textos.is_some() {
                regravar_fechada(d, &v, &doc)?;
            }
            d.revisao_gravar(c.bp, c.peca, numero, "aprovada", "", c.hash, crate::tempo::agora_ms())?;
        }
        "pendente" => d.revisao_limpar(c.bp, c.peca)?,
        _ => {
            d.revisao_gravar(c.bp, c.peca, numero, c.estado, c.observacao, c.hash, crate::tempo::agora_ms())?;
            acordar(d, c)?;
        }
    }
    let resolvida = resolver_se_acabou(d, c.ws, c.bp, &v, &doc)?;
    Ok(json!({"ok": true, "avisos": avisos, "reabertos": reabertos, "resolvida": resolvida}))
}

/// `d:<id>` e `det:<acao>/<id>`: só gravam, com o hash. O detalhe pode estar na árvore ou chegando na versão pendente.
fn hash_da_peca(atual: &PlanoDoc, doc: Option<&PlanoDoc>, peca: &str) -> R<Option<String>> {
    if let Some(id) = peca.strip_prefix("d:") {
        return Ok(atual.decisoes.iter().find(|x| x.id == id).map(hash_decisao));
    }
    let Some((r, id)) = peca.strip_prefix("det:").and_then(|p| p.split_once('/')) else {
        return Err(falha(422, crate::tr!("Unknown piece", "Peça desconhecida")).codigo("peca-invalida"));
    };
    let de = |p: &PlanoDoc| acao_do(p, r).and_then(|a| a.detalhes.iter().find(|x| x.id == id)).map(hash_detalhe);
    Ok(doc.and_then(de).or_else(|| de(atual)))
}

fn revisar_peca(d: &mut Dados, c: &Clique, numero: i64) -> R<Value> {
    let v = d.plano_pendente(c.bp)?;
    // sem versão pendente não há número valendo, e o hash basta; com ela, o clique tem de ser nela
    if v.as_ref().is_some_and(|v| v.numero != numero) {
        return Err(plano_mudou());
    }
    let doc = v.as_ref().map(|v| doc_da_versao(d, v)).transpose()?;
    let atual = do_blueprint(&blueprint_de(d, c.ws, c.bp)?);
    if hash_da_peca(&atual, doc.as_ref(), c.peca)?.as_deref() != Some(c.hash) {
        return Err(plano_mudou());
    }
    if c.estado == "pendente" {
        d.revisao_limpar(c.bp, c.peca)?;
    } else {
        d.revisao_gravar(c.bp, c.peca, v.as_ref().map_or(0, |v| v.numero), c.estado, c.observacao, c.hash, crate::tempo::agora_ms())?;
        if c.estado != "aprovada" {
            acordar(d, c)?;
        }
    }
    Ok(json!({"ok": true, "avisos": []}))
}

fn revisar(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let (ws, bp) = alvo(corpo)?;
    let c = Clique { ws: &ws, bp: &bp, peca: texto(corpo, "peca"), estado: texto(corpo, "estado"), observacao: texto(corpo, "observacao"), hash: texto(corpo, "hash") };
    valida_estado(c.estado)?;
    let numero = corpo["numero"].as_i64().unwrap_or(0);
    let mut d = travar(dados);
    match c.peca.strip_prefix("diff:") {
        Some(r) => revisar_diff(&mut d, &c, r, numero, corpo.get("textos")),
        None => revisar_peca(&mut d, &c, numero),
    }
}

/// As ações novas de um `salvar_plano` esperam este gesto. Na origem `escrita` elas entram ao fechar.
fn colocar(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let (ws, bp) = alvo(corpo)?;
    let refs: Vec<String> = lista(corpo, "acoes").iter().filter_map(Value::as_str).map(String::from).collect();
    if refs.is_empty() {
        return Err(falha(422, crate::tr!("Choose at least one action", "Escolha pelo menos uma ação")).codigo("faltam-acoes"));
    }
    let numero = corpo["numero"].as_i64().unwrap_or(0);
    let mut d = travar(dados);
    let v = d.plano_pendente(&bp)?.filter(|v| v.numero == numero).ok_or_else(plano_mudou)?;
    if v.origem != "salvar" {
        return Err(falha(409, crate::tr!("Only a saved plan waits to be placed", "Só um plano salvo espera ser colocado")).codigo("nao-e-salvar"));
    }
    let arvore = blueprint_de(&mut d, &ws, &bp)?;
    let doc = doc_da_versao(&d, &v)?;
    let novas = comparar_planos(Some(&do_blueprint(&arvore)), &doc).novas;
    if !refs.iter().all(|r| novas.contains(r)) {
        return Err(plano_mudou());
    }
    let nova = projetar(&arvore, &so_as_acoes(&doc), Escopo::So(&refs));
    let avisos = gravar_projecao(&mut d, &ws, nova)?;
    let resolvida = resolver_se_acabou(&mut d, &ws, &bp, &v, &doc)?;
    Ok(json!({"ok": true, "colocadas": refs, "avisos": avisos, "resolvida": resolvida}))
}

pub fn rota(mcp: &Mcp, dados: &Mutex<Dados>, _pedido: &Pedido, recurso: &str, corpo: &Value) -> R<Resposta> {
    let saida = match recurso {
        "gesto/revisar" => revisar(dados, corpo)?,
        "gesto/colocar" => colocar(dados, corpo)?,
        _ => return Err(super::nao_encontrado()),
    };
    mcp.planejar_mudou();
    Ok(Resposta::json(200, &saida))
}
