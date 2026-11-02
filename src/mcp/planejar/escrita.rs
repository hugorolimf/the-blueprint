//! `plano/abrir`, `plano/escrever`, `plano/fechar` e `plano/recado`: o que chega do agente fica em `plano_versoes` e só o fechamento mexe na árvore (D1, D4; contrato 5.1).
//! A versão guarda só o que mudou em relação à árvore: nome e resumo vazios = o agente não mandou, e assim renomear o blueprint durante a escrita não é desfeito ao fechar.
//! Passa de 300 linhas: as quatro rotas dividem a mesma mescla, e a fatia dona (F31) não pode criar módulo novo nem editar `planejar/mod.rs`.
use super::super::acoes::localizar;
use super::super::hooks::{caminho_valido, travado};
use super::super::worktree::principal_ou;
use super::Mcp;
use crate::contexto::Contexto;
use crate::dados::planejar::{NovoRecado, VersaoLinha};
use crate::dados::Dados;
use crate::erro::Erro;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use crate::plano::arvore::{do_blueprint, projetar, recompor, Escopo};
use crate::plano::comparar::{comparar_planos, Resumo};
use crate::plano::formato::{AcaoDoc, Decisao, FiguraInfo, Pergunta, PlanoDoc, ProjetoDoc, FORMATO};
use crate::plano::ids::id_real;
use crate::plano::svg::{css_seguro, MAX_FIGURAS};
use crate::plano::validar::{achar_ciclo, avisos, validar, validar_peca};
use crate::tempo::{agora_ms, hoje_local};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Mutex;

/// Por que a rota não seguiu: erro comum, ou validação (`422 {erros, avisos}`), que o agente corrige de uma vez.
enum Recusa {
    Http(ErroHttp),
    Invalido(Vec<String>, Vec<String>),
}

impl From<ErroHttp> for Recusa {
    fn from(e: ErroHttp) -> Self {
        Recusa::Http(e)
    }
}

impl From<Erro> for Recusa {
    fn from(e: Erro) -> Self {
        Recusa::Http(e.into())
    }
}

fn invalido<T>(erros: Vec<String>, avisos: Vec<String>) -> Result<T, Recusa> {
    Err(Recusa::Invalido(erros, avisos))
}

pub fn rota(mcp: &Mcp, dados: &Mutex<Dados>, _pedido: &Pedido, recurso: &str, corpo: &Value) -> Result<Resposta, ErroHttp> {
    let feito = {
        let mut d = travado(dados);
        match recurso {
            "plano/abrir" => abrir(&mut d, corpo),
            "plano/escrever" => escrever(&mut d, corpo),
            "plano/fechar" => fechar(&mut d, corpo),
            "plano/recado" => recado(&mut d, corpo),
            _ => Err(super::nao_encontrado().into()),
        }
    };
    match feito {
        Ok(v) => {
            mcp.planejar_mudou();
            Ok(Resposta::json(200, &v))
        }
        Err(Recusa::Http(e)) => Err(e),
        Err(Recusa::Invalido(erros, avisos)) => {
            let erro = erros.first().cloned().unwrap_or_default();
            Ok(Resposta::json(422, &json!({"erro": erro, "codigo": "plano-invalido", "erros": erros, "avisos": avisos})))
        }
    }
}

fn txt<'a>(v: &'a Value, campo: &str) -> &'a str {
    v.get(campo).and_then(Value::as_str).unwrap_or("")
}

fn textos(v: &Value, campo: &str) -> Vec<String> {
    v.get(campo).and_then(Value::as_array).map_or_else(Vec::new, |l| l.iter().filter_map(Value::as_str).map(String::from).collect())
}

fn marca(lista: &mut Vec<String>, r: &str) {
    if !lista.iter().any(|x| x == r) {
        lista.push(r.to_string());
    }
}

fn tira(lista: &mut Vec<String>, r: &str) {
    lista.retain(|x| x != r);
}

/// A escrita ainda aberta com este id.
pub(super) fn aberta(d: &Dados, escrita_id: &str) -> Result<VersaoLinha, ErroHttp> {
    let linha = d.plano_escrita(escrita_id)?.ok_or_else(|| falha(404, crate::tr!("no open plan with this escrita_id: call abrir_plano", "não há plano aberto com este escrita_id: chame abrir_plano")).codigo("escrita-nao-encontrada"))?;
    if linha.situacao != "escrevendo" {
        return Err(falha(409, crate::tr!("this plan was already closed: call abrir_plano again", "este plano já foi fechado: chame abrir_plano de novo")).codigo("escrita-fechada"));
    }
    Ok(linha)
}

/// O doc guardado em `plano_versoes`. Esta coluna só o servidor escreve, então `figura_info` é lido dela, ao contrário do que vem do agente em `PlanoDoc::de_json`.
pub(super) fn doc_gravado(v: &Value) -> Result<PlanoDoc, ErroHttp> {
    let mut doc = PlanoDoc::de_json(v).map_err(|e| falha(500, e.join("; ")))?;
    for (p, pv) in doc.projetos.iter_mut().zip(v["projetos"].as_array().into_iter().flatten()) {
        for (a, av) in p.acoes.iter_mut().zip(pv["acoes"].as_array().into_iter().flatten()) {
            for (det, dv) in a.detalhes.iter_mut().zip(av["detalhes"].as_array().into_iter().flatten()) {
                det.figura_info = dv.get("figura_info").and_then(FiguraInfo::de_json);
            }
        }
    }
    Ok(doc)
}

fn blueprint_atual(d: &mut Dados, ws: &str, bp: &str) -> Result<Value, ErroHttp> {
    let tudo = d.ler_tudo(&Contexto::local())?;
    let (_, achado) = localizar(tudo, ws, bp)?;
    achado.cloned().ok_or_else(super::nao_encontrado)
}

/// (a árvore como doc, a base da escrita): a versão pendente por cima da árvore, sem isso a segunda escrita apagaria a mudança à espera. Só as ações: o que é do blueprint já entrou ao fechar.
pub(super) fn base_de(d: &Dados, bp: &Value) -> Result<(PlanoDoc, PlanoDoc), ErroHttp> {
    let arvore = do_blueprint(bp);
    let Some(p) = d.plano_pendente(txt(bp, "id"))? else {
        let base = arvore.clone();
        return Ok((arvore, base));
    };
    let projetos = recompor(&doc_gravado(&p.plano)?, &arvore, &p.tocadas, &p.removidas).projetos;
    let base = PlanoDoc { projetos, ..arvore.clone() };
    Ok((arvore, base))
}

fn acha<'a>(doc: &'a PlanoDoc, id: &str) -> Option<&'a AcaoDoc> {
    doc.projetos.iter().flat_map(|p| &p.acoes).find(|a| a.id == id)
}

fn chave(nome: Option<&str>) -> Option<String> {
    nome.map(|n| n.trim().to_lowercase()).filter(|n| !n.is_empty())
}

/// `projeto`: `Some` = naquele grupo (mudando, se a ação já estava em outro); `None` = onde já está, ou sem projeto, se é nova.
fn colocar(doc: &mut PlanoDoc, projeto: Option<Option<&str>>, acao: AcaoDoc) {
    let destino = projeto.map(chave);
    let lugar = doc.projetos.iter().enumerate().find_map(|(i, p)| p.acoes.iter().position(|a| a.id == acao.id).map(|j| (i, j)));
    if let Some((i, j)) = lugar {
        if destino.as_ref().is_none_or(|k| chave(doc.projetos[i].nome.as_deref()) == *k) {
            doc.projetos[i].acoes[j] = acao;
            return;
        }
        doc.projetos[i].acoes.remove(j);
    }
    let alvo = destino.flatten();
    let i = doc.projetos.iter().position(|p| chave(p.nome.as_deref()) == alvo).unwrap_or_else(|| {
        doc.projetos.push(ProjetoDoc { nome: alvo.is_some().then(|| projeto.flatten().unwrap_or("").trim().to_string()), acoes: Vec::new() });
        doc.projetos.len() - 1
    });
    doc.projetos[i].acoes.push(acao);
    doc.projetos.retain(|p| !p.acoes.is_empty() || p.nome.is_some());
}

fn sem_conteudo(a: &AcaoDoc) -> bool {
    a.texto.is_none() && a.planos.is_empty() && a.checkpoints.is_empty() && a.detalhes.is_empty() && a.tabelas.is_empty()
}

/// Nome e prazo, se vieram. `""` no prazo apaga: é como a ação fica sem data.
fn por_cabecalho(a: &mut AcaoDoc, nova: &AcaoDoc) {
    if !nova.nome.is_empty() {
        a.nome = nova.nome.clone();
    }
    if let Some(de) = &nova.de {
        a.de = Some(de.clone()).filter(|s| !s.is_empty());
    }
    if let Some(ate) = &nova.ate {
        a.ate = Some(ate.clone()).filter(|s| !s.is_empty());
    }
}

/// O reenvio substitui o item de mesmo id; o que não veio fica.
fn por_id<T>(lista: &mut Vec<T>, novos: Vec<T>, id: impl Fn(&T) -> &str) {
    for n in novos {
        match lista.iter().position(|x| id(x) == id(&n)) {
            Some(i) => lista[i] = n,
            None => lista.push(n),
        }
    }
}

/// Id repetido dentro do mesmo pedido: `por_id` engoliria o primeiro sem ninguém saber.
fn repetidos<'a>(ids: impl Iterator<Item = &'a str>, tipo: &str, erros: &mut Vec<String>) {
    let mut vistos = BTreeSet::new();
    erros.extend(ids.filter(|id| !vistos.insert(*id)).map(|id| format!("{tipo} {id}: repeated id")));
}

/// O que `plano/escrever` manda, por cima do que já há. Detalhe que cita a mesma figura mantém o `figura_info`: a figura já foi conferida numa versão anterior.
fn mesclar(antes: Option<&AcaoDoc>, nova: AcaoDoc, bruta: &Value) -> AcaoDoc {
    let mut a = antes.cloned().unwrap_or_else(|| AcaoDoc { id: nova.id.clone(), ..AcaoDoc::default() });
    por_cabecalho(&mut a, &nova);
    if nova.texto.is_some() {
        a.texto = nova.texto;
    }
    let remover = textos(bruta, "removerPlanos");
    a.planos.retain(|q| !remover.contains(&q.id));
    por_id(&mut a.planos, nova.planos, |q| q.id.as_str());
    por_id(&mut a.checkpoints, nova.checkpoints, |c| c.id.as_str());
    let mut detalhes = nova.detalhes;
    for d in &mut detalhes {
        d.figura_info = a.detalhes.iter().find(|v| v.id == d.id && v.figura == d.figura).and_then(|v| v.figura_info.clone());
    }
    por_id(&mut a.detalhes, detalhes, |d| d.id.as_str());
    if bruta.get("tabelas").is_some_and(Value::is_array) {
        a.tabelas = nova.tabelas;
    }
    a
}

/// As ações que o `abrir` declarou entram no doc: só nome e prazo, o conteúdo vem em `escrever`. Devolve os erros das que mudaram: declarar sem mudar não valida o que o agente não mandou.
fn declarar(doc: &mut PlanoDoc, tocadas: &mut Vec<String>, aguardando: &mut Vec<String>, declaradas: &PlanoDoc) -> Vec<String> {
    let mut mudadas = Vec::new();
    for pr in &declaradas.projetos {
        let nome = pr.nome.as_deref();
        if pr.acoes.is_empty() && chave(nome).is_some() && !doc.projetos.iter().any(|p| chave(p.nome.as_deref()) == chave(nome)) {
            doc.projetos.push(pr.clone());
        }
        for nova in &pr.acoes {
            let antes = acha(doc, &nova.id).cloned();
            let mudou_de_projeto = antes.is_some() && chave(nome).is_some() && !doc.projetos.iter().any(|p| chave(p.nome.as_deref()) == chave(nome) && p.acoes.iter().any(|a| a.id == nova.id));
            let mut a = antes.clone().unwrap_or_else(|| AcaoDoc { id: nova.id.clone(), ..AcaoDoc::default() });
            por_cabecalho(&mut a, nova);
            if sem_conteudo(&a) {
                marca(aguardando, &a.id);
            }
            if antes.as_ref() != Some(&a) || mudou_de_projeto {
                colocar(doc, if antes.is_none() || mudou_de_projeto { Some(nome) } else { None }, a);
                marca(tocadas, &nova.id);
                mudadas.push(nova.id.clone());
            }
        }
    }
    let mut erros = Vec::new();
    for id in &mudadas {
        for e in acha(doc, id).map(|a| validar_peca(doc, a)).unwrap_or_default() {
            marca(&mut erros, &e);
        }
    }
    erros
}

fn abrir(d: &mut Dados, corpo: &Value) -> Result<Value, Recusa> {
    let (ws, origem) = (txt(corpo, "workspaceId"), txt(corpo, "origem"));
    if ws.is_empty() {
        return Err(falha(400, crate::tr!("workspaceId is required", "falta o workspaceId")).codigo("faltam-ids").into());
    }
    if !matches!(origem, "escrita" | "salvar") {
        return Err(falha(400, crate::tr!("origem must be escrita or salvar", "origem deve ser escrita ou salvar")).codigo("origem-invalida").into());
    }
    let declaradas = PlanoDoc::de_json(&json!({"formato": FORMATO, "tema": corpo.get("tema"), "projetos": corpo.get("projetos").filter(|p| !p.is_null()).cloned().unwrap_or_else(|| json!([]))})).or_else(|e| invalido(e, Vec::new()))?;
    let estilo = corpo.get("estiloCss").and_then(Value::as_str);
    if let Some(Err(e)) = estilo.map(css_seguro) {
        return invalido(vec![format!("estiloCss: {e}")], Vec::new());
    }
    let (nome, resumo) = (txt(corpo, "blueprint").trim(), corpo.get("resumo").and_then(Value::as_str));
    let pedido_bp = txt(corpo, "blueprintId");
    localizar(d.ler_tudo(&Contexto::local())?, ws, pedido_bp)?;
    let novo = pedido_bp.is_empty();
    let pasta = txt(corpo, "pasta");
    if novo && nome.is_empty() {
        return invalido(vec!["blueprint: missing the name".to_string()], Vec::new());
    }
    if novo && !pasta.is_empty() && !(caminho_valido(pasta) && Path::new(pasta).is_dir()) {
        return invalido(vec![format!("pasta: \"{pasta}\" is not a folder")], Vec::new());
    }

    let aberta_antes = if novo { None } else { d.plano_aberta(pedido_bp)? };
    if aberta_antes.as_ref().is_some_and(|l| l.origem != origem) {
        return Err(falha(409, crate::tr!("this blueprint has a plan being written by another path: close or discard it first", "este blueprint tem um plano sendo escrito por outro caminho: feche ou descarte primeiro")).codigo("escrita-aberta").into());
    }
    let vazio = || PlanoDoc { formato: FORMATO.into(), ..PlanoDoc::default() };
    let (mut doc, mut aguardando, mut tocadas, mut removidas) = match &aberta_antes {
        Some(l) if origem == "escrita" => (doc_gravado(&l.plano)?, l.aguardando.clone(), l.tocadas.clone(), l.removidas.clone()),
        Some(_) => (vazio(), Vec::new(), Vec::new(), Vec::new()),
        None if novo || origem == "salvar" => (vazio(), Vec::new(), Vec::new(), Vec::new()),
        None => {
            let bp = blueprint_atual(d, ws, pedido_bp)?;
            let base = base_de(d, &bp)?.1;
            (PlanoDoc { blueprint: String::new(), resumo: None, ..base }, Vec::new(), Vec::new(), Vec::new())
        }
    };
    if !novo && !nome.is_empty() {
        doc.blueprint = nome.to_string();
    }
    if let (false, Some(r)) = (novo, resumo) {
        doc.resumo = Some(r.to_string());
    }
    if !declaradas.tema.is_empty() {
        doc.tema = declaradas.tema.clone();
    }
    if let Some(css) = estilo {
        doc.estilo = Some(css.to_string());
    }
    let erros = declarar(&mut doc, &mut tocadas, &mut aguardando, &declaradas);
    if !erros.is_empty() {
        return invalido(erros, avisos(&doc));
    }
    for r in textos(corpo, "removerAcoes") {
        doc.projetos.iter_mut().for_each(|p| p.acoes.retain(|a| a.id != r));
        tira(&mut tocadas, &r);
        tira(&mut aguardando, &r);
        marca(&mut removidas, &r);
    }
    let agora = agora_ms();
    let bp_id = if novo { criar_blueprint(d, ws, nome, resumo, pasta)? } else { pedido_bp.to_string() };
    let (numero, escrita_id, continuacao) = match &aberta_antes {
        Some(l) => (l.numero, l.escrita_id.clone(), true),
        None => {
            let (n, e) = d.plano_abrir(ws, &bp_id, origem, txt(corpo, "agente"), &doc.para_json(), &aguardando, agora)?;
            (n, e, false)
        }
    };
    d.plano_gravar(&escrita_id, &doc.para_json(), &aguardando, &tocadas, &removidas, agora)?;
    Ok(json!({"ok": true, "escritaId": escrita_id, "blueprintId": bp_id, "numero": numero, "aguardando": aguardando, "continuacao": continuacao, "avisos": avisos(&doc)}))
}

/// Blueprint novo: uma gravação com versão, e a pasta, se veio.
fn criar_blueprint(d: &mut Dados, ws: &str, nome: &str, resumo: Option<&str>, pasta: &str) -> Result<String, ErroHttp> {
    let id = id_real(ws, "blueprint", &super::novo_gesto());
    let mut bp = json!({"id": id, "nome": nome, "acoes": []});
    if let Some(r) = resumo {
        bp["descricao"] = json!(r);
    }
    let mut tudo = d.ler_tudo(&Contexto::local())?.clone();
    blueprints_de(&mut tudo, ws).ok_or_else(super::nao_encontrado)?.push(bp);
    d.gravar_tudo(&Contexto::local(), &tudo)?;
    d.tocar_revisao();
    if !pasta.is_empty() {
        d.vincular_pasta(&Contexto::local(), &principal_ou(pasta), ws, &id)?;
    }
    Ok(id)
}

fn blueprints_de<'a>(tudo: &'a mut Value, ws: &str) -> Option<&'a mut Vec<Value>> {
    let dono = if ws == "sm" { Some(tudo) } else { tudo.get_mut("workspaces")?.as_array_mut()?.iter_mut().find(|c| txt(c, "id") == ws) }?;
    let o = dono.as_object_mut()?;
    o.entry("blueprints").or_insert_with(|| json!([])).as_array_mut()
}

fn escrever(d: &mut Dados, corpo: &Value) -> Result<Value, Recusa> {
    let linha = aberta(d, txt(corpo, "escritaId"))?;
    let Some(bruta) = corpo.get("acao").filter(|a| a.is_object()) else {
        return invalido(vec!["acao: required, with its id".to_string()], Vec::new());
    };
    let nova = AcaoDoc::de_json(bruta).or_else(|e| invalido(e, Vec::new()))?;
    let mut erros = Vec::new();
    if linha.origem == "escrita" {
        erros.extend(nova.planos.iter().filter(|q| q.id_gerado).map(|q| format!("plan {}: id is required in escrever_acao; ids link one version of the plan to the next", q.id)));
    }
    repetidos(nova.planos.iter().map(|q| q.id.as_str()), "plan", &mut erros);
    repetidos(nova.checkpoints.iter().map(|c| c.id.as_str()), "checkpoint", &mut erros);
    repetidos(nova.detalhes.iter().map(|x| x.id.as_str()), "detail", &mut erros);
    let mut doc = doc_gravado(&linha.plano)?;
    let lista = |campo: &str| corpo.get(campo).and_then(Value::as_array).cloned().unwrap_or_default();
    let (mut decisoes, mut perguntas) = (Vec::new(), Vec::new());
    for v in lista("decisoes") {
        Decisao::de_json(&v).map_or_else(|e| erros.extend(e), |x| decisoes.push(x));
    }
    for v in lista("perguntas") {
        Pergunta::de_json(&v).map_or_else(|e| erros.extend(e), |x| perguntas.push(x));
    }
    repetidos(decisoes.iter().map(|x| x.id.as_str()), "decision", &mut erros);
    repetidos(perguntas.iter().map(|x| x.id.as_str()), "question", &mut erros);
    por_id(&mut doc.decisoes, decisoes, |q| q.id.as_str());
    por_id(&mut doc.perguntas, perguntas, |q| q.id.as_str());
    let acao = mesclar(acha(&doc, &nova.id), nova, bruta);
    let id = acao.id.clone();
    let projeto = bruta.get("projeto").and_then(Value::as_str);
    colocar(&mut doc, projeto.map(Some), acao.clone());
    erros.extend(validar_peca(&doc, &acao));
    if !erros.is_empty() {
        return invalido(erros, avisos(&doc));
    }
    let (mut aguardando, mut tocadas, mut removidas) = (linha.aguardando, linha.tocadas, linha.removidas);
    tira(&mut aguardando, &id);
    tira(&mut removidas, &id);
    marca(&mut tocadas, &id);
    d.plano_gravar(&linha.escrita_id, &doc.para_json(), &aguardando, &tocadas, &removidas, agora_ms())?;
    Ok(json!({"ok": true, "avisos": avisos(&doc), "faltam": aguardando}))
}

/// Põe nos detalhes do que esta versão mandou as medidas das figuras recebidas. Figura reenviada no mesmo caminho vale sobre a que já estava.
fn preencher_figuras(d: &Dados, linha: &VersaoLinha, doc: &mut PlanoDoc, tocadas: Option<&[String]>) -> Result<(), ErroHttp> {
    for a in doc.projetos.iter_mut().flat_map(|p| &mut p.acoes).filter(|a| tocadas.is_none_or(|t| t.contains(&a.id))) {
        for det in &mut a.detalhes {
            if let Some(f) = det.figura.as_deref().map(|c| d.figura_de(&linha.blueprint_id, linha.numero, c)).transpose()?.flatten() {
                det.figura_info = Some(FiguraInfo { hash: f.hash, tipo: f.tipo, largura: f.largura, altura: f.altura });
            }
        }
    }
    Ok(())
}

/// Id real repetido na árvore projetada: `gravar_tudo` recusaria com erro de banco; aqui o agente sabe o que corrigir (D3).
fn id_repetido(bp: &Value) -> Option<String> {
    let mut vistos = BTreeSet::new();
    let filhos = |a: &Value, campo: &str| a[campo].as_array().cloned().unwrap_or_default();
    for a in bp["acoes"].as_array().into_iter().flatten() {
        if !vistos.insert(("acao", txt(a, "id").to_string())) {
            return Some(txt(a, "id").to_string());
        }
        for (tabela, campo) in [("plano", "planos"), ("checkpoint", "checkpoints")] {
            for f in filhos(a, campo) {
                if !vistos.insert((tabela, txt(&f, "id").to_string())) {
                    return Some(txt(&f, "id").to_string());
                }
            }
        }
    }
    None
}

fn resumo_json(r: &Resumo) -> Value {
    json!({"mais": r.mais, "menos": r.menos, "alterados": r.alterados, "detalhes": r.detalhes, "outros": r.outros})
}

fn fechar(d: &mut Dados, corpo: &Value) -> Result<Value, Recusa> {
    let linha = aberta(d, txt(corpo, "escritaId"))?;
    let (ws, bp_id) = (linha.workspace_id.clone(), linha.blueprint_id.clone());
    let bp = blueprint_atual(d, &ws, &bp_id)?;
    let (arvore, base) = base_de(d, &bp)?;
    let salvar = linha.origem == "salvar";
    let mut doc = doc_gravado(&linha.plano)?;
    if !salvar {
        doc = recompor(&doc, &base, &linha.tocadas, &linha.removidas);
    }
    if doc.blueprint.is_empty() {
        doc.blueprint = arvore.blueprint.clone();
    }
    let escopo = if salvar { None } else { Some(linha.tocadas.as_slice()) };
    preencher_figuras(d, &linha, &mut doc, escopo)?;
    let sem_figura = |acao: &str, det: &crate::plano::formato::Detalhe| vec![format!("action {acao}, detail {}: the figure {} was not sent", det.id, det.figura.as_deref().unwrap_or(""))];
    let mut erros = validar(&doc, escopo, &sem_figura);
    let figuras: BTreeSet<&str> = doc.projetos.iter().flat_map(|p| &p.acoes).flat_map(|a| &a.detalhes).filter_map(|x| x.figura.as_deref()).collect();
    if figuras.len() > MAX_FIGURAS {
        erros.push(format!("detalhes: more than {MAX_FIGURAS} figures"));
    }
    let mut lista_avisos = avisos(&doc);
    lista_avisos.extend(linha.aguardando.iter().map(|r| format!("action {r}: closed with no content yet")));
    if !erros.is_empty() {
        return invalido(erros, lista_avisos);
    }

    let comp = comparar_planos(Some(&arvore), &doc);
    let colocadas = if salvar { Vec::new() } else { comp.novas.clone() };
    let projetado = projetar(&bp, &doc, Escopo::So(&colocadas), &hoje_local());
    let depois = do_blueprint(&projetado);
    let planos: Vec<_> = depois.projetos.iter().flat_map(|p| &p.acoes).flat_map(|a| &a.planos).collect();
    if let Some(ciclo) = achar_ciclo(&planos) {
        return Err(falha(409, format!("plans in a cycle: {}", ciclo.join(" -> "))).codigo("ciclo").into());
    }
    if let Some(id) = id_repetido(&projetado) {
        return Err(falha(409, format!("the id {id} is repeated in the blueprint")).codigo("id-duplicado").into());
    }
    if projetado != bp {
        let mut tudo = d.ler_tudo(&Contexto::local())?.clone();
        let lista = blueprints_de(&mut tudo, &ws).ok_or_else(super::nao_encontrado)?;
        if let Some(lugar) = lista.iter_mut().find(|b| txt(b, "id") == bp_id) {
            *lugar = projetado;
        }
        d.gravar_tudo(&Contexto::local(), &tudo)?;
        d.tocar_revisao();
    }

    // O que fica à espera, exatamente: é o que a próxima escrita põe por cima da árvore.
    let mut pendentes: Vec<String> = comp.alteradas.keys().cloned().collect();
    pendentes.extend(comp.novas.iter().filter(|r| !colocadas.contains(r)).cloned());
    let agora = agora_ms();
    d.plano_gravar(&linha.escrita_id, &doc.para_json(), &[], &pendentes, &comp.removidas, agora)?;
    let numero = d.plano_fechar(&linha.escrita_id, agora)?;
    if pendentes.is_empty() && comp.removidas.is_empty() {
        d.plano_resolver(&bp_id, numero)?;
    }
    let alteradas: Vec<Value> = comp.alteradas.iter().map(|(r, c)| json!({"ref": r, "resumo": resumo_json(&c.resumo)})).collect();
    Ok(json!({"ok": true, "numero": numero, "novas": comp.novas, "colocadas": colocadas, "alteradas": alteradas, "removidas": comp.removidas, "iguais": comp.iguais.len(), "avisos": lista_avisos}))
}

fn recado(d: &mut Dados, corpo: &Value) -> Result<Value, Recusa> {
    let (ws, texto, de) = (txt(corpo, "workspaceId"), txt(corpo, "texto").trim(), txt(corpo, "de"));
    if !matches!(de, "orquestrador" | "agente") {
        return Err(falha(400, crate::tr!("de must be orquestrador or agente", "de deve ser orquestrador ou agente")).codigo("de-invalido").into());
    }
    if texto.is_empty() {
        return Err(falha(400, crate::tr!("texto is required", "falta o texto")).codigo("texto-vazio").into());
    }
    localizar(d.ler_tudo(&Contexto::local())?, ws, "")?;
    let execucao = Some(txt(corpo, "para")).filter(|s| !s.is_empty()).unwrap_or_else(|| txt(corpo, "execucaoId"));
    d.recado_gravar(&NovoRecado { workspace_id: ws, blueprint_id: txt(corpo, "blueprintId"), acao_id: txt(corpo, "acaoId"), execucao_id: execucao, de, texto, em: agora_ms() })?;
    Ok(json!({"ok": true}))
}
