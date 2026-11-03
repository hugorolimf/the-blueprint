//! As oito ferramentas do Planejar (contrato, seção 6): o `--mcp` lê o que precisa do disco da sessão e compõe as rotas `plano/*`. Só existem no app local, fora do Windows (D14).
//! Aqui não há nada que aprove, marque para iniciar, integre ou ligue agente: isso é só da janela do app, em rotas que este processo não sabe chamar. Texto para o AGENTE: inglês.
use super::argumentos::{estruturado, opcional, texto};
use super::cliente::codificar;
use super::perfil::Perfil;
use super::stdio::{servidor, Via};
use crate::plano::formato::{Decisao, Pergunta, PlanoDoc};
use crate::plano::pacote::ler_pasta;
use crate::plano::svg::{LIMITE_IMAGEM, LIMITE_SVG};
use crate::plano::validar::{avisos, validar};
use serde_json::{json, Map, Value};
use std::io::Read;
use std::path::{Path, PathBuf};

pub(super) const FERRAMENTAS: [&str; 8] = ["abrir_plano", "escrever_acao", "fechar_plano", "salvar_plano", "ler_plano", "ler_respostas", "ler_workspace", "falar_com_voce"];
pub(super) const SO_LOCAL: &str = "This tool only works with the local app and its window, and the session on the same machine. Use propor_blueprint.";

type R = Result<String, String>;

/// D14: escondida e recusada no Windows e por HTTP.
pub(super) fn escondida(nome: &str, via: Via) -> bool {
    FERRAMENTAS.contains(&nome) && (cfg!(windows) || via.is_some())
}

pub(super) fn chamar(via: Via, nome: &str, a: &Value) -> R {
    if escondida(nome, via) {
        return Err(SO_LOCAL.into());
    }
    let sessao = std::env::current_dir().ok();
    ferramenta(via, nome, a, super::perfil::atual(), sessao.as_deref())
}

/// `sessao`: a pasta do `--mcp`, que é a da sessão do agente.
pub(super) fn ferramenta(via: Via, nome: &str, a: &Value, perfil: Perfil, sessao: Option<&Path>) -> R {
    match nome {
        "abrir_plano" => abrir(via, a, perfil, sessao),
        "escrever_acao" => escrever(via, a, sessao),
        "fechar_plano" => fechar(via, &texto(a, "escrita_id")?, Vec::new()),
        "salvar_plano" => salvar(via, a, perfil, sessao),
        "ler_plano" => ler(via, "plano/ler", &[("workspace", texto(a, "workspace_id")?), ("blueprint", texto(a, "blueprint_id")?)]),
        "ler_respostas" => {
            let execucao = Some(opcional(a, "execucao_id")).filter(|e| !e.trim().is_empty()).or_else(|| super::argumentos::ambiente("BLUEPRINT_EXECUCAO")).unwrap_or_default();
            ler(via, "plano/respostas", &[("workspace", texto(a, "workspace_id")?), ("blueprint", texto(a, "blueprint_id")?), ("execucao", execucao)])
        }
        "ler_workspace" => ler(via, "plano/workspace", &[("workspace", texto(a, "workspace_id")?)]),
        "falar_com_voce" => {
            let corpo = json!({"workspaceId": texto(a, "workspace_id")?, "blueprintId": opcional(a, "blueprint_id"), "acaoId": opcional(a, "acao_id"), "para": opcional(a, "para_execucao"), "texto": texto(a, "texto")?, "de": "orquestrador"});
            servidor(via, "plano/recado", "POST", Some(&corpo))?;
            Ok("Message left. The user sees it on the agents screen; an agent reads it in ler_respostas.".into())
        }
        _ => Err(format!("Unknown tool: {nome}")),
    }
}

fn ler(via: Via, rota: &str, parametros: &[(&str, String)]) -> R {
    let query: Vec<String> = parametros.iter().filter(|(_, v)| !v.is_empty()).map(|(k, v)| format!("{k}={}", codificar(v))).collect();
    Ok(servidor(via, &format!("{rota}?{}", query.join("&")), "GET", None)?.to_string()) // compacto: cada byte vai para o contexto do agente
}

fn lista_de(v: &Value, chave: &str) -> Vec<String> {
    v[chave].as_array().map_or_else(Vec::new, |l| l.iter().filter_map(Value::as_str).map(String::from).collect())
}

fn com_avisos(mut saida: String, lista: &[String]) -> String {
    if !lista.is_empty() {
        saida.push_str(&format!("\nWarnings:\n- {}", lista.join("\n- ")));
    }
    saida
}

fn nome_do_agente(perfil: Perfil) -> String {
    super::argumentos::ambiente("BLUEPRINT_AGENTE_NOME").filter(|n| !n.trim().is_empty()).unwrap_or_else(|| if perfil == Perfil::Orquestrador { "orquestrador".into() } else { String::new() })
}

/// Blueprint novo nasce ligado à pasta da sessão, menos o do orquestrador: a pasta dele é a dedicada dele, e a tela pede a certa.
fn pasta_do_novo(novo: bool, perfil: Perfil, sessao: Option<&Path>) -> Option<String> {
    (novo && perfil != Perfil::Orquestrador).then(|| sessao.map(|p| p.display().to_string())).flatten()
}

fn abrir(via: Via, a: &Value, perfil: Perfil, sessao: Option<&Path>) -> R {
    let bp = opcional(a, "blueprint_id");
    let mut corpo = Map::new();
    corpo.insert("workspaceId".into(), json!(texto(a, "workspace_id")?));
    corpo.insert("blueprintId".into(), json!(bp.trim()));
    corpo.insert("origem".into(), json!("escrita"));
    corpo.insert("agente".into(), json!(nome_do_agente(perfil)));
    for (de, para) in [("blueprint", "blueprint"), ("resumo", "resumo")] {
        if let Some(v) = a.get(de).filter(|v| v.is_string()) {
            corpo.insert(para.into(), v.clone());
        }
    }
    for (de, para) in [("tema", "tema"), ("projetos", "projetos"), ("remover_acoes", "removerAcoes")] {
        if a.get(de).is_some() {
            corpo.insert(para.into(), estruturado(a, de));
        }
    }
    if let Some(p) = pasta_do_novo(bp.trim().is_empty(), perfil, sessao) {
        corpo.insert("pasta".into(), json!(p));
    }
    let r = servidor(via, "plano/abrir", "POST", Some(&Value::Object(corpo)))?;
    let aguardando = lista_de(&r, "aguardando");
    let volta = if r["continuacao"] == true { " This is the plan that was already open: what you sent before is kept." } else { "" };
    let falta = if aguardando.is_empty() { String::new() } else { format!(" Waiting for content: {}.", aguardando.join(", ")) };
    let saida = format!("Plan open (escrita_id: {}, blueprint_id: {}, version {}).{volta}{falta} Next: escrever_acao for each action, then fechar_plano.", r["escritaId"].as_str().unwrap_or(""), r["blueprintId"].as_str().unwrap_or(""), r["numero"]);
    Ok(com_avisos(saida, &lista_de(&r, "avisos")))
}

fn tipo_da_figura(caminho: &str) -> Result<&'static str, String> {
    let extensao = caminho.rsplit(['/', '\\']).next().and_then(|n| n.rsplit_once('.')).map_or(String::new(), |(_, e)| e.to_ascii_lowercase());
    match extensao.as_str() {
        "svg" => Ok("svg"),
        "png" => Ok("png"),
        "jpg" | "jpeg" => Ok("jpg"),
        "webp" => Ok("webp"),
        _ => Err(format!("{caminho}: use SVG, PNG, JPG or WebP")),
    }
}

/// Mesmas regras de caminho de `pacote::ler_pasta`, sem seguir link. O conteúdo (SVG seguro, magic bytes) o servidor confere em `plano/figura`.
fn ler_figura_da_sessao(sessao: &Path, caminho: &str) -> Result<Vec<u8>, String> {
    let fora = || format!("{caminho}: must be a relative path inside this session's folder");
    if caminho.is_empty() || caminho.contains(['\0', ':']) || caminho.starts_with(['/', '\\']) {
        return Err(fora());
    }
    let partes: Vec<&str> = caminho.split(['/', '\\']).filter(|s| !s.is_empty() && *s != ".").collect();
    if partes.is_empty() || partes.contains(&"..") {
        return Err(fora());
    }
    let limite = if tipo_da_figura(caminho)? == "svg" { LIMITE_SVG } else { LIMITE_IMAGEM };
    let mut atual = sessao.to_path_buf();
    for (i, parte) in partes.iter().enumerate() {
        atual.push(parte);
        let tipo = std::fs::symlink_metadata(&atual).map_err(|_| format!("{caminho}: not found"))?.file_type();
        if tipo.is_symlink() {
            return Err(format!("{caminho}: symbolic links are not accepted"));
        }
        let ultimo = i + 1 == partes.len();
        if (ultimo && !tipo.is_file()) || (!ultimo && !tipo.is_dir()) {
            return Err(format!("{caminho}: not a regular file inside this session's folder"));
        }
    }
    let mut bytes = Vec::new();
    std::fs::File::open(&atual).map_err(|e| format!("{caminho}: {e}"))?.take(limite as u64 + 1).read_to_end(&mut bytes).map_err(|e| format!("{caminho}: {e}"))?;
    if bytes.len() > limite {
        return Err(format!("{caminho}: larger than {} MB", limite >> 20));
    }
    Ok(bytes)
}

fn enviar_figura(via: Via, escrita: &str, caminho: &str, bytes: &[u8]) -> Result<(), String> {
    let tipo = tipo_da_figura(caminho)?;
    let mut corpo = json!({"escritaId": escrita, "caminho": caminho, "tipo": tipo});
    if tipo == "svg" {
        corpo["svg"] = json!(std::str::from_utf8(bytes).map_err(|_| format!("{caminho}: SVG is not UTF-8"))?);
    } else {
        corpo["base64"] = json!(crate::acesso::segredo::base64url(bytes).replace('-', "+").replace('_', "/")); // `plano/figura` lê o base64 padrão
    }
    servidor(via, "plano/figura", "POST", Some(&corpo)).map(drop)
}

/// Sobe as figuras dos detalhes, primeiro, e deixa em cada um só o caminho: o conteúdo já foi.
fn figuras_dos_detalhes(via: Via, escrita: &str, acao: &mut Value, sessao: Option<&Path>) -> Result<(), String> {
    let id_acao = acao["id"].as_str().unwrap_or("").to_string();
    for d in acao.get_mut("detalhes").and_then(Value::as_array_mut).into_iter().flatten().filter_map(Value::as_object_mut) {
        let id = d.get("id").and_then(Value::as_str).unwrap_or("").to_string();
        let (arquivo, svg) = (d.remove("figura_caminho"), d.remove("figura_svg"));
        let (caminho, bytes) = match (arquivo.as_ref().and_then(Value::as_str), svg.as_ref().and_then(Value::as_str)) {
            (Some(_), Some(_)) => return Err(format!("detail {id}: send figura_caminho or figura_svg, not both")),
            (Some(c), None) => (c.to_string(), ler_figura_da_sessao(sessao.ok_or("Cannot tell which folder this session is in.")?, c)?),
            (None, Some(s)) => (format!("figuras/{id_acao}-{id}.svg"), s.as_bytes().to_vec()),
            (None, None) => continue,
        };
        enviar_figura(via, escrita, &caminho, &bytes)?;
        d.insert("figura".into(), json!(caminho));
    }
    Ok(())
}

fn escrever(via: Via, a: &Value, sessao: Option<&Path>) -> R {
    let escrita = texto(a, "escrita_id")?;
    let mut acao = estruturado(a, "acao");
    if !acao.is_object() {
        return Err("acao is required: an object with its id.".into());
    }
    figuras_dos_detalhes(via, &escrita, &mut acao, sessao)?;
    if let Some(o) = acao.as_object_mut() {
        if let Some(r) = o.remove("remover_planos") {
            o.insert("removerPlanos".into(), r);
        }
    }
    let id = acao["id"].as_str().unwrap_or("").to_string();
    let mut corpo = json!({"escritaId": escrita, "acao": acao});
    for campo in ["decisoes", "perguntas"] {
        if a.get(campo).is_some() {
            corpo[campo] = estruturado(a, campo);
        }
    }
    let r = servidor(via, "plano/escrever", "POST", Some(&corpo))?;
    let faltam = lista_de(&r, "faltam");
    let proximo = if faltam.is_empty() { "No action is waiting for content: call fechar_plano when you are done.".to_string() } else { format!("Still waiting for content: {}.", faltam.join(", ")) };
    Ok(com_avisos(format!("Action {id} is on the user's screen. {proximo}"), &lista_de(&r, "avisos")))
}

fn fechar(via: Via, escrita: &str, mut lista: Vec<String>) -> R {
    let r = servidor(via, "plano/fechar", "POST", Some(&json!({"escritaId": escrita})))?;
    lista.extend(lista_de(&r, "avisos"));
    let (novas, colocadas, removidas) = (lista_de(&r, "novas"), lista_de(&r, "colocadas"), lista_de(&r, "removidas"));
    let esperam: Vec<String> = novas.iter().filter(|n| !colocadas.contains(n)).cloned().collect();
    let alteradas: Vec<String> = r["alteradas"].as_array().map(Vec::as_slice).unwrap_or(&[]).iter().map(|x| {
        let t = &x["resumo"];
        format!("{} (+{} -{} ~{}, details {}, other {})", x["ref"].as_str().unwrap_or(""), t["mais"], t["menos"], t["alterados"], t["detalhes"], t["outros"])
    }).collect();
    let mut saida = format!("Plan closed as version {}.", r["numero"]);
    for (rotulo, itens) in [("New, already on the sheet", &colocadas), ("New, waiting for the user to place them", &esperam), ("Changed, waiting for the user's review", &alteradas), ("Removed, waiting for the user's review", &removidas)] {
        if !itens.is_empty() {
            saida.push_str(&format!("\n- {rotulo}: {}", itens.join(", ")));
        }
    }
    saida.push_str(&format!("\n- Unchanged: {}\nOnly the user approves. Do not say a change entered before ler_blueprint shows it; ler_respostas brings the answers.", r["iguais"]));
    Ok(com_avisos(saida, &lista))
}

fn erros_da_pasta(erros: &[String]) -> String {
    format!("The plan folder has errors; fix them and send again:\n- {}", erros.join("\n- "))
}

fn relativa_a_sessao(sessao: Option<&Path>, pasta: &str) -> PathBuf {
    sessao.map_or_else(|| PathBuf::from(pasta), |s| s.join(pasta))
}

/// `plano/abrir` (origem salvar) + figuras + `plano/escrever` por ação + `plano/fechar`. Confere tudo antes de abrir: a escrita não fica aberta por causa de um plano torto.
pub(super) fn salvar(via: Via, a: &Value, perfil: Perfil, sessao: Option<&Path>) -> R {
    let (pasta, ws, bp) = (texto(a, "pasta")?, texto(a, "workspace_id")?, opcional(a, "blueprint_id"));
    let pacote = ler_pasta(&relativa_a_sessao(sessao, &pasta)).map_err(|e| erros_da_pasta(&e))?;
    let doc = PlanoDoc::de_json(&pacote.plano).map_err(|e| erros_da_pasta(&e))?;
    let erros = validar(&doc, None, &|_, _| Vec::new());
    if !erros.is_empty() {
        return Err(erros_da_pasta(&erros));
    }
    let mut lista = pacote.avisos.clone();
    lista.extend(avisos(&doc));
    let projetos: Vec<Value> = doc.projetos.iter().map(|p| json!({"nome": p.nome, "acoes": p.acoes.iter().map(|x| json!({"id": x.id, "nome": x.nome})).collect::<Vec<_>>()})).collect();
    let mut abrir = json!({"workspaceId": ws, "blueprintId": bp.trim(), "blueprint": doc.blueprint, "origem": "salvar", "agente": nome_do_agente(perfil), "projetos": projetos, "tema": doc.tema});
    if let Some(r) = &doc.resumo {
        abrir["resumo"] = json!(r);
    }
    if let Some(css) = &pacote.estilo {
        abrir["estiloCss"] = json!(css);
    }
    if let Some(p) = pasta_do_novo(bp.trim().is_empty(), perfil, sessao) {
        abrir["pasta"] = json!(p);
    }
    let r = servidor(via, "plano/abrir", "POST", Some(&abrir))?;
    let escrita = r["escritaId"].as_str().ok_or("The app did not return an escrita_id.")?.to_string();
    lista.extend(lista_de(&r, "avisos"));
    for (caminho, bytes) in &pacote.figuras {
        enviar_figura(via, &escrita, caminho, bytes)?;
    }
    let mut primeira = true;
    for p in &doc.projetos {
        for acao in &p.acoes {
            let mut corpo = json!({"escritaId": escrita, "acao": acao.para_json()});
            if let Some(nome) = &p.nome {
                corpo["acao"]["projeto"] = json!(nome);
            }
            if primeira {
                corpo["decisoes"] = doc.decisoes.iter().map(Decisao::para_json).collect();
                corpo["perguntas"] = doc.perguntas.iter().map(Pergunta::para_json).collect();
                primeira = false;
            }
            servidor(via, "plano/escrever", "POST", Some(&corpo))?;
        }
    }
    if primeira && !(doc.decisoes.is_empty() && doc.perguntas.is_empty()) {
        lista.push("This plan has no action, so its decisions and questions were not sent.".into());
    }
    Ok(format!("blueprint_id: {}\n{}", r["blueprintId"].as_str().unwrap_or(""), fechar(via, &escrita, lista)?))
}

#[cfg(test)]
#[path = "ferr_plano_tests.rs"]
mod tests;
