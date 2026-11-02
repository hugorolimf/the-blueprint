//! Os gestos que não mudam a árvore (contrato, seção 5.3): marcar para iniciar, cancelar, retomar, integrar, responder, limite, comando, pasta, orquestrador e descartar escrita.
//! Só gesto cria execução `marcada` (seção 9). Integrar roda o git fora do `Mutex<Dados>`. O comando do agente fica em `planejar_config`, fora de `/api/dados`, e nunca passa por shell.
use super::git;
use super::revisao::{alvo,blueprint_de, doc_da_versao, lista, texto, travar, R};
use crate::contexto::Contexto;
use crate::dados::planejar::{Mudanca, NovaExecucao, NovoRecado};
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use crate::mcp::acoes::localizar;
use crate::mcp::Mcp;
use crate::plano::arvore::{do_blueprint, hash_peca};
use crate::plano::formato::Pergunta;
use crate::plano::frases::{self, Frase};
use crate::plano::ids;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Mutex;

/// Sem sinal há mais que isto, quem pediu ajuda é dado como morto: retomar não põe dois agentes na mesma cópia.
const SEM_SINAL_MS: i64 = 120_000;
const LIMITE_MAX: u64 = 16;

fn ref_de(v: &Value) -> &str {
    Some(texto(v, "ref")).filter(|r| !r.is_empty()).unwrap_or_else(|| texto(v, "id"))
}

fn workspace_existe(d: &mut Dados, ws: &str) -> R<()> {
    localizar(d.ler_tudo(&Contexto::local())?, ws, "")?;
    Ok(())
}

fn sem_execucao() -> ErroHttp {
    falha(404, crate::tr!("Run not found", "Execução não encontrada")).codigo("execucao-nao-encontrada")
}

/// `<plano>` do blueprint com a ação que o contém.
fn plano_e_acao<'a>(bp: &'a Value, id: &str) -> Option<(&'a Value, &'a Value)> {
    lista(bp, "acoes").iter().find_map(|a| lista(a, "planos").iter().find(|q| texto(q, "id") == id).map(|q| (a, q)))
}

/// Dependência sem iniciar avisa e não impede. `juntos`: o "Iniciar os N" marca os dependentes de uma vez.
fn avisos_de_dependencia(d: &Dados, bp_id: &str, bp: &Value, plano: &Value, juntos: &[String]) -> R<Vec<String>> {
    let chave = |r: &str| format!("{bp_id}|{r}");
    let todos: Vec<&Value> = lista(bp, "acoes").iter().flat_map(|a| lista(a, "planos")).collect();
    let (mut faltam, mut avisos) = (Vec::new(), Vec::new());
    for dep in lista(plano, "dependeDe").iter().filter_map(Value::as_str) {
        let Some(q) = todos.iter().find(|q| ref_de(q) == dep) else {
            avisos.push(frases::texto(&Frase::DependeFalta { id: chave(ref_de(plano)), dep: chave(dep) }));
            continue;
        };
        let id = texto(q, "id");
        let andando = q["feito"] == json!(true) || juntos.iter().any(|j| j == id) || d.execucao_viva(bp_id, id)?.is_some();
        if !andando {
            faltam.push(chave(dep));
        }
    }
    if !faltam.is_empty() {
        avisos.push(frases::texto(&Frase::DependeSemIniciar { id: chave(ref_de(plano)), faltam }));
    }
    Ok(avisos)
}

fn marcar(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let (ws, bp_id) = alvo(corpo)?;
    let pedidos: Vec<&str> = lista(corpo, "planos").iter().filter_map(Value::as_str).collect();
    if pedidos.is_empty() {
        return Err(falha(422, crate::tr!("Choose at least one plan", "Escolha pelo menos um plano")).codigo("faltam-planos"));
    }
    let mut d = travar(dados);
    let bp = blueprint_de(&mut d, &ws, &bp_id)?;
    let pasta = d.pasta_real_do_blueprint(&ws, &bp_id)?.ok_or_else(|| falha(409, crate::tr!("Link a folder to this blueprint first", "Ligue uma pasta a este blueprint primeiro")).codigo("sem-pasta"))?;
    let mut reais: Vec<String> = Vec::new();
    for pedido in pedidos {
        let real = ids::traduzir(&bp, pedido, "plano").ok_or_else(|| falha(404, crate::tr!("Plan not found", "Plano não encontrado")).codigo("plano-nao-encontrado"))?;
        let feito = plano_e_acao(&bp, &real).is_some_and(|(_, q)| q["feito"] == json!(true));
        if feito || d.execucao_viva(&bp_id, &real)?.is_some() {
            return Err(falha(409, crate::tr!("Plan {pedido} is already done or running", "O plano {pedido} já está feito ou em execução")).codigo("ja-tem-execucao"));
        }
        if !reais.contains(&real) {
            reais.push(real);
        }
    }
    let mut avisos = Vec::new();
    for real in &reais {
        let Some((acao, plano)) = plano_e_acao(&bp, real) else { continue };
        avisos.extend(avisos_de_dependencia(&d, &bp_id, &bp, plano, &reais)?);
        d.execucao_criar(&NovaExecucao { workspace_id: &ws, blueprint_id: &bp_id, acao_id: texto(acao, "id"), plano_id: real, plano_ref: ref_de(plano), area: texto(plano, "area"), pasta_principal: &pasta, marcada_em: crate::tempo::agora_ms() })?;
    }
    Ok(json!({"ok": true, "avisos": avisos}))
}

fn cancelar(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let id = texto(corpo, "execucaoId");
    let mut d = travar(dados);
    d.execucao_de(id)?.ok_or_else(sem_execucao)?;
    if !d.execucao_cancelar(id, crate::tempo::agora_ms())? {
        return Err(falha(409, crate::tr!("This run is no longer active", "Esta execução não está mais ativa")).codigo("execucao-terminada"));
    }
    Ok(json!({"ok": true}))
}

/// A cópia fica como está: a próxima tentativa reusa. `ajuda` com sinal recente é agente vivo, esperando resposta.
fn retomar(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let id = texto(corpo, "execucaoId");
    let mut d = travar(dados);
    let x = d.execucao_de(id)?.ok_or_else(sem_execucao)?;
    let nao_retoma = || falha(409, crate::tr!("This run cannot be resumed", "Esta execução não pode ser retomada")).codigo("nao-retoma");
    match x.situacao.as_str() {
        "sumiu" | "falhou" => {}
        "ajuda" => {
            let sinal = x.ultimo_sinal.or(x.iniciada_em).unwrap_or(x.marcada_em);
            if crate::tempo::agora_ms() - sinal <= SEM_SINAL_MS {
                return Err(falha(409, crate::tr!("The agent is still alive: answer it instead of starting another", "O agente ainda está vivo: responda em vez de iniciar outro")).codigo("agente-vivo"));
            }
        }
        _ => return Err(nao_retoma()),
    }
    let m = Mudanca { so_de: &["sumiu", "falhou", "ajuda"], situacao: Some("marcada"), motivo: Some(""), terminada_em: Some(None), parar: Some(false), ..Mudanca::default() };
    if !d.execucao_mudar(id, &m)? {
        return Err(nao_retoma());
    }
    Ok(json!({"ok": true}))
}

fn recusa_do_git(r: git::Recusa) -> ErroHttp {
    let codigo = r.codigo();
    let mensagem = match &r {
        git::Recusa::RamoInvalido => crate::tr!("Invalid branch name", "Nome de ramo inválido"),
        git::Recusa::RamoMudou => crate::tr!("The main folder is on another branch now", "A pasta principal agora está em outro ramo"),
        git::Recusa::Ocupada(nome) => crate::tr!("The main folder has an operation in progress ({nome})", "A pasta principal tem uma operação em andamento ({nome})"),
        git::Recusa::Suja => crate::tr!("Local changes in the main folder would be overwritten", "Alterações locais na pasta principal seriam sobrescritas"),
        git::Recusa::Conflito => crate::tr!("The branch conflicts with the main folder: nothing changed", "O ramo conflita com a pasta principal: nada mudou"),
        git::Recusa::Falhou(erro) => crate::tr!("git failed: {erro}", "o git falhou: {erro}"),
    };
    falha(409, mensagem).codigo(codigo)
}

/// Fora do `Mutex<Dados>`: o merge pode levar minutos. A falha ao remover a cópia só avisa: o merge já aconteceu.
fn integrar(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let (id, esperado) = (texto(corpo, "execucaoId"), texto(corpo, "ramoEsperado"));
    let x = travar(dados).execucao_de(id)?.ok_or_else(sem_execucao)?;
    if x.situacao != "pronta" || x.ramo.is_empty() || x.integrada_em.is_some() {
        return Err(falha(409, crate::tr!("This run has no branch waiting to be integrated", "Esta execução não tem ramo esperando integração")).codigo("nao-integravel"));
    }
    let programa = git::programa();
    let principal = git::raiz(&programa, Path::new(&x.pasta_principal)).ok_or_else(|| falha(409, crate::tr!("The main folder is not a git repository", "A pasta principal não é um repositório git")).codigo("git-indisponivel"))?;
    git::integrar(&programa, &principal, esperado, &x.ramo).map_err(recusa_do_git)?;
    let m = Mudanca { integrada_em: Some(Some(crate::tempo::agora_ms())), ..Mudanca::default() };
    travar(dados).execucao_mudar(id, &m)?;
    let copia = if x.pasta_trabalho.is_empty() { Vec::new() } else { git::remover_copia(&programa, &principal, Path::new(&x.pasta_trabalho), &x.ramo).err().into_iter().collect() };
    let avisos: Vec<String> = copia.into_iter().map(|e: String| crate::tr!("The branch was integrated, but the copy stayed: {e}", "O ramo foi integrado, mas a cópia ficou: {e}")).collect();
    Ok(json!({"ok": true, "ramo": x.ramo, "destino": esperado, "avisos": avisos}))
}

/// A pergunta está na árvore (entra ao fechar); na versão pendente, se a árvore ainda não a tem.
fn responder(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let (ws, bp) = alvo(corpo)?;
    let (id, resposta) = (texto(corpo, "pergunta"), texto(corpo, "resposta").trim());
    if resposta.is_empty() {
        return Err(falha(422, crate::tr!("The answer is empty", "A resposta está vazia")).codigo("resposta-vazia"));
    }
    let mut d = travar(dados);
    let arvore = do_blueprint(&blueprint_de(&mut d, &ws, &bp)?);
    let pendente = d.plano_pendente(&bp)?;
    let doc = pendente.as_ref().map(|v| doc_da_versao(&d, v)).transpose()?;
    let achar = |p: &crate::plano::formato::PlanoDoc| p.perguntas.iter().find(|q| q.id == id).map(Pergunta::para_json);
    let pergunta = achar(&arvore).or_else(|| doc.as_ref().and_then(achar)).ok_or_else(|| falha(404, crate::tr!("Question not found", "Pergunta não encontrada")).codigo("pergunta-nao-encontrada"))?;
    let numero = pendente.as_ref().map_or(0, |v| v.numero);
    d.revisao_gravar(&bp, &format!("q:{id}"), numero, "respondida", resposta, &hash_peca(&pergunta), crate::tempo::agora_ms())?;
    d.orq_pedir(&ws, &format!("The user answered question {id} of blueprint {bp}"), false)?;
    Ok(json!({"ok": true}))
}

fn limite(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let ws = texto(corpo, "workspaceId");
    let n = corpo["limite"].as_u64().filter(|n| (1..=LIMITE_MAX).contains(n)).ok_or_else(|| falha(422, crate::tr!("The limit goes from 1 to {LIMITE_MAX}", "O limite vai de 1 a {LIMITE_MAX}")).codigo("limite-invalido"))?;
    let mut d = travar(dados);
    workspace_existe(&mut d, ws)?;
    d.config_gravar(&format!("limite:{ws}"), &n.to_string())?;
    Ok(json!({"ok": true, "limite": n}))
}

/// Argumentos separados, que vão a `Command` sem shell. Lista vazia, argumento vazio ou caractere de controle: recusa.
fn argumentos(v: &Value) -> R<String> {
    let invalido = || falha(422, crate::tr!("The command must be a non-empty list of arguments, without control characters", "O comando precisa ser uma lista de argumentos não vazia, sem caracteres de controle")).codigo("comando-invalido");
    let itens = v.as_array().filter(|l| !l.is_empty()).ok_or_else(invalido)?;
    if !itens.iter().all(|a| a.as_str().is_some_and(|s| !s.trim().is_empty() && !s.chars().any(char::is_control))) {
        return Err(invalido());
    }
    Ok(v.to_string())
}

fn comando(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let campos = [("agente", "comando_agente"), ("orquestrador", "comando_orquestrador")];
    let mut novos = Vec::new();
    for (campo, chave) in campos {
        if let Some(v) = corpo.get(campo) {
            novos.push((chave, argumentos(v)?));
        }
    }
    if novos.is_empty() {
        return Err(falha(422, crate::tr!("Send agente or orquestrador", "Mande agente ou orquestrador")).codigo("comando-invalido"));
    }
    let mut d = travar(dados);
    for (chave, valor) in novos {
        d.config_gravar(chave, &valor)?;
    }
    Ok(json!({"ok": true}))
}

fn pasta(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let (ws, bp) = alvo(corpo)?;
    let caminho = texto(corpo, "caminho").trim();
    let ruim = cfg!(windows) && caminho.contains(';');
    if ruim || !Path::new(caminho).is_absolute() || !Path::new(caminho).is_dir() {
        return Err(falha(422, crate::tr!("This path is not a folder on this computer", "Este caminho não é uma pasta neste computador")).codigo("nao-e-pasta"));
    }
    let mut d = travar(dados);
    blueprint_de(&mut d, &ws, &bp)?;
    d.vincular_pasta(&Contexto::local(), caminho, &ws, &bp)?;
    Ok(json!({"ok": true}))
}

/// Mandar mensagem liga: o usuário pediu para ele acordar. O texto vai como recado, nunca como motivo: o motivo entra no prompt sem marcador.
fn orquestrador(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let ws = texto(corpo, "workspaceId");
    let ligado = corpo.get("ligado").and_then(Value::as_bool);
    let mensagem = texto(corpo, "mensagem").trim();
    if ligado.is_none() && mensagem.is_empty() {
        return Err(falha(422, crate::tr!("Send ligado or mensagem", "Mande ligado ou mensagem")).codigo("nada-a-fazer"));
    }
    let mut d = travar(dados);
    workspace_existe(&mut d, ws)?;
    if let Some(l) = ligado {
        d.orq_ligar(ws, l)?;
    }
    if !mensagem.is_empty() {
        d.recado_gravar(&NovoRecado { workspace_id: ws, blueprint_id: texto(corpo, "blueprintId"), acao_id: texto(corpo, "acaoId"), execucao_id: "", de: "usuario", texto: mensagem, em: crate::tempo::agora_ms() })?;
        d.orq_ligar(ws, true)?;
        d.orq_pedir(ws, "The user sent a message", true)?;
    }
    Ok(json!({"ok": true, "ligado": d.orq_ler(ws)?.ligado}))
}

/// Só a escrita aberta; a árvore não muda. Sem escrita aberta com esse id: 404 de `plano_descartar`.
fn descartar_escrita(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    travar(dados).plano_descartar(texto(corpo, "escritaId"))?;
    Ok(json!({"ok": true}))
}

pub fn rota(mcp: &Mcp, dados: &Mutex<Dados>, _pedido: &Pedido, recurso: &str, corpo: &Value) -> R<Resposta> {
    let saida = match recurso {
        "gesto/marcar" => marcar(dados, corpo)?,
        "gesto/cancelar" => cancelar(dados, corpo)?,
        "gesto/retomar" => retomar(dados, corpo)?,
        "gesto/integrar" => integrar(dados, corpo)?,
        "gesto/responder" => responder(dados, corpo)?,
        "gesto/limite" => limite(dados, corpo)?,
        "gesto/comando" => comando(dados, corpo)?,
        "gesto/pasta" => pasta(dados, corpo)?,
        "gesto/orquestrador" => orquestrador(dados, corpo)?,
        "gesto/descartar-escrita" => descartar_escrita(dados, corpo)?,
        _ => return Err(super::nao_encontrado()),
    };
    mcp.planejar_mudou();
    Ok(Resposta::json(200, &saida))
}
