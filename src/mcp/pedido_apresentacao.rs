//! `POST /api/mcp/apresentar`: o usuário escolheu público e nível na tela Apresentar e clicou em Gerar com IA (contrato em docs/interno/apresentacao-ia.md). Mesmo caminho
//! e mesmas regras de `iniciar`: só por clique, só loopback (rotas.rs), a sessão nova abre na pasta do blueprint (`blueprint claude "<prompt>"`) ou o pedido vai a uma
//! sessão já aberta pelo canal dela (`sessaoId`, ver `iniciar::em_sessao_de`). O prompt leva público, nível e a estimativa, para o agente não perguntar de novo.
//! Aqui nada grava nos dados.
use super::acoes;
use super::hooks::{texto, travado};
use super::iniciar::{id_valido, limpo, recusa, Lancador};
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Mutex;

const PUBLICOS: [&str; 2] = ["cliente", "interno"];
const NIVEIS: [&str; 4] = ["rapido", "padrao", "completo", "epico"];

/// A estimativa de tempo da tabela do contrato, no idioma de quem clicou.
fn estimativa(nivel: &str) -> String {
    match nivel {
        "rapido" => crate::tr!("up to about 1 h", "até cerca de 1 h"),
        "padrao" => crate::tr!("about 1 to 3 h", "cerca de 1 a 3 h"),
        "completo" => crate::tr!("about 2 to 4 h", "cerca de 2 a 4 h"),
        _ => crate::tr!("6 h or more", "6 h ou mais"),
    }
}

fn publico_por_extenso(publico: &str) -> String {
    if publico == "cliente" {
        crate::tr!("the client (a project plan, without the technical development)", "o cliente (um plano de projeto, sem o desenvolvimento técnico)")
    } else {
        crate::tr!("internal use (the full technical view)", "uso interno (a visão técnica completa)")
    }
}

/// O pedido que o Claude recebe. Sem `;` (o Windows Terminal o lê como separador de comandos); o nome do blueprint vem dos dados e passa antes por `limpo`.
pub(crate) fn prompt(nome_bp: &str, ws: &str, bp: &str, publico: &str, nivel: &str) -> String {
    let (para, tempo) = (publico_por_extenso(publico), estimativa(nivel));
    crate::tr!(
        "Build the presentation of blueprint «{nome_bp}» (workspace_id {ws}, blueprint_id {bp}) for {para}: conteudo.publico \"{publico}\", conteudo.nivel \"{nivel}\", estimate {tempo}. The user already chose audience and level in the app: do not ask for them again. Read guia_de_planejamento(\"apresentar\") and ler_blueprint, follow the flow of the guide (ask only what is still missing, say the estimate, write the content and the figures) and attach the result with anexar_apresentacao.",
        "Monte a apresentação do blueprint «{nome_bp}» (workspace_id {ws}, blueprint_id {bp}) para {para}: conteudo.publico \"{publico}\", conteudo.nivel \"{nivel}\", estimativa {tempo}. O usuário já escolheu público e nível no app: não pergunte de novo. Leia guia_de_planejamento(\"apresentar\") e ler_blueprint, siga o fluxo do guia (pergunte só o que ainda faltar, diga a estimativa, escreva o conteúdo e as figuras) e anexe o resultado com anexar_apresentacao."
    )
}

/// `lancar` abre o terminal na pasta (1º argumento) rodando o exe (2º) com `claude` e o prompt (3º), como em `iniciar::iniciar`; os testes passam um falso.
pub(crate) fn apresentar(dados: &Mutex<Dados>, corpo: &Value, lancar: &Lancador<'_>) -> Result<(u16, Value), ErroHttp> {
    let (ws, bp) = (texto(corpo, "workspaceId"), texto(corpo, "blueprintId"));
    if ![ws, bp].iter().all(|i| id_valido(i)) {
        return Err(falha(400, crate::tr!("Send workspaceId and blueprintId.", "Informe workspaceId e blueprintId.")).codigo("faltam-ids"));
    }
    let (publico, nivel) = (texto(corpo, "publico"), texto(corpo, "nivel"));
    if !PUBLICOS.contains(&publico) {
        return Err(falha(400, crate::tr!("publico must be \"cliente\" or \"interno\".", "publico deve ser \"cliente\" ou \"interno\".")).codigo("publico-invalido"));
    }
    if !NIVEIS.contains(&nivel) {
        return Err(falha(400, crate::tr!("nivel must be rapido, padrao, completo or epico.", "nivel deve ser rapido, padrao, completo ou epico.")).codigo("nivel-invalido"));
    }
    let (pasta, texto_do_pedido) = {
        let mut d = travado(dados);
        let tudo = d.ler_tudo(&Contexto::local())?;
        let estrutura = acoes::blueprint(tudo, ws, bp, None)?; // 404 de workspace ou blueprint que não existe
        let Some(pasta) = d.pasta_do_blueprint(&Contexto::local(), ws, bp)?.filter(|c| Path::new(c).is_dir()) else {
            return Ok(recusa(409, &crate::tr!("This blueprint has no linked folder (or it no longer exists).", "Este blueprint não tem uma pasta ligada (ou ela não existe mais)."), "sem-pasta"));
        };
        (pasta, prompt(&limpo(texto(&estrutura["blueprint"], "nome")), ws, bp, publico, nivel))
    };
    let exe = std::env::current_exe().map_err(|e| falha(500, crate::tr!("cannot tell where this program is: {e}", "não sei onde este programa está: {e}")))?;
    Ok(match lancar(Path::new(&pasta), &exe, &texto_do_pedido) {
        Ok(terminal) => (200, json!({"ok": true, "pasta": pasta, "terminal": terminal})),
        Err(e) => recusa(501, &e, "sem-terminal"),
    })
}

/// O que o usuário digitou no campo de conversa vira parte do prompt de um terminal: sem controle (quebra de linha vira espaço), sem `;` (o Windows Terminal o lê como separador
/// de comandos), sem as aspas «» do próprio pedido, espaços repetidos juntos e no máximo 600 caracteres.
pub(crate) fn texto_do_usuario(t: &str) -> String {
    let limpo: String = t.chars().map(|c| if c.is_control() { ' ' } else { c }).map(|c| match c { ';' => ',', '«' | '»' => '"', c => c }).collect();
    limpo.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(600).collect()
}

/// O pedido de ajuste: o agente lê a apresentação, muda SÓ o que o usuário pediu em operações pequenas e não a regera. Sem `;` (ver `prompt`).
pub(crate) fn prompt_ajuste(nome_bp: &str, ws: &str, bp: &str, id: &str, titulo: &str, pedido: &str) -> String {
    crate::tr!(
        "The user asks for a change to presentation «{titulo}» (apresentacao_id {id}) of blueprint «{nome_bp}» (workspace_id {ws}, blueprint_id {bp}). The request: «{pedido}». Read the presentation with ler_apresentacao and change ONLY what was asked, with editar_apresentacao in small operations (definir, inserir, remover, mover, figura_trecho): do not rewrite it and do not call anexar_apresentacao over it unless the user asks to regenerate it. Follow guia_de_planejamento(\"apresentacao_ajustar\"). Then say in one sentence what you changed.",
        "O usuário pede uma mudança na apresentação «{titulo}» (apresentacao_id {id}) do blueprint «{nome_bp}» (workspace_id {ws}, blueprint_id {bp}). O pedido: «{pedido}». Leia a apresentação com ler_apresentacao e mude SÓ o que foi pedido, com editar_apresentacao em operações pequenas (definir, inserir, remover, mover, figura_trecho): não a reescreva e não chame anexar_apresentacao sobre ela a menos que o usuário peça para regerar. Siga guia_de_planejamento(\"apresentacao_ajustar\"). Depois diga numa frase o que mudou."
    )
}

/// Como `apresentar`, para um ajuste: `{workspaceId, blueprintId, apresentacaoId, texto | ajuste}` (a tela manda `texto`; o terminal do app, `ajuste`). A apresentação precisa ser deste blueprint.
pub(crate) fn ajustar_com(pasta_apresentacoes: &Path, dados: &Mutex<Dados>, corpo: &Value, lancar: &Lancador<'_>) -> Result<(u16, Value), ErroHttp> {
    let (ws, bp, id) = (texto(corpo, "workspaceId"), texto(corpo, "blueprintId"), texto(corpo, "apresentacaoId"));
    if ![ws, bp].iter().all(|i| id_valido(i)) {
        return Err(falha(400, crate::tr!("Send workspaceId and blueprintId.", "Informe workspaceId e blueprintId.")).codigo("faltam-ids"));
    }
    let registro = crate::apresentacao::armazenar::ler(pasta_apresentacoes, id).filter(|r| crate::apresentacao::armazenar::alvo(r) == (ws, bp));
    let Some(registro) = registro else {
        return Err(falha(404, crate::tr!("This presentation does not exist in this blueprint.", "Esta apresentação não existe neste blueprint.")).codigo("apresentacao-nao-encontrada"));
    };
    let bruto = if texto(corpo, "texto").is_empty() { texto(corpo, "ajuste") } else { texto(corpo, "texto") };
    let pedido = texto_do_usuario(bruto);
    if pedido.is_empty() {
        return Err(falha(400, crate::tr!("Write what you want changed.", "Escreva o que você quer mudar.")).codigo("falta-texto"));
    }
    let (pasta, texto_do_pedido) = {
        let mut d = travado(dados);
        let tudo = d.ler_tudo(&Contexto::local())?;
        let estrutura = acoes::blueprint(tudo, ws, bp, None)?;
        let Some(pasta) = d.pasta_do_blueprint(&Contexto::local(), ws, bp)?.filter(|c| Path::new(c).is_dir()) else {
            return Ok(recusa(409, &crate::tr!("This blueprint has no linked folder (or it no longer exists).", "Este blueprint não tem uma pasta ligada (ou ela não existe mais)."), "sem-pasta"));
        };
        (pasta, prompt_ajuste(&limpo(texto(&estrutura["blueprint"], "nome")), ws, bp, id, &limpo(texto(&registro, "titulo")), &pedido))
    };
    let exe = std::env::current_exe().map_err(|e| falha(500, crate::tr!("cannot tell where this program is: {e}", "não sei onde este programa está: {e}")))?;
    Ok(match lancar(Path::new(&pasta), &exe, &texto_do_pedido) {
        Ok(terminal) => (200, json!({"ok": true, "pasta": pasta, "terminal": terminal})),
        Err(e) => recusa(501, &e, "sem-terminal"),
    })
}

/// `POST /api/mcp/apresentacao/pedido`: com `sessaoId` o pedido vai à sessão aberta pelo canal dela (com `semCanal` só volta o `prompt`); sem ele abre um terminal novo.
pub(crate) fn ajustar(mcp: &super::Mcp, dados: &Mutex<Dados>, corpo: &Value) -> Result<(u16, Value), ErroHttp> {
    let pasta = mcp.pasta_apresentacoes();
    let base = |d: &Mutex<Dados>, c: &Value, l: &Lancador<'_>| ajustar_com(&pasta, d, c, l);
    if corpo.get("sessaoId").is_some() {
        super::iniciar::em_sessao_de(mcp, dados, corpo, &base)
    } else {
        base(dados, corpo, &super::iniciar::abrir_terminal)
    }
}
