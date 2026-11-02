//! O motor do Planejar (contrato, seção 7): a cada tique percebe agente sem sinal, cancela o que saiu da árvore e lança no máximo um plano marcado. Só lança `marcada`, e só gesto cria `marcada`.
//! Ligado só em `nativa.rs`: `blueprint serve` não pega a trava de `instancia::garantir`, e dois motores no mesmo banco lançariam em dobro. Git e spawn correm fora do `Mutex<Dados>`.
//! Passa de 300 linhas (teto 500) porque varrer, escolher e lançar são os passos de um mesmo tique e dividem os mesmos auxiliares; o corte natural é `escolher` num módulo próprio.
use super::vista::travados_do_ws;
use super::{git, terminal};
use crate::contexto::Contexto;
use crate::dados::planejar::{ExecucaoLinha, Lancamento, Mudanca, NovoRecado};
use crate::dados::Dados;
use crate::erro::Resultado;
use crate::http::Estado;
use crate::mcp::hooks::travado;
use crate::mcp::Mcp;
use crate::plano::entrada::{entrada_esc, ExecEsc};
use crate::plano::escalonar::{areas_conflitam, escalonar, Situacao};
use crate::plano::frases::{self, Frase};
use crate::plano::ids;
use serde_json::Value;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Weak};
use std::time::Duration;

const TIQUE: Duration = Duration::from_secs(2);
const SEM_SINAL_MS: i64 = 120_000;
const CARENCIA_MS: i64 = 120_000;
/// Tique com mais atraso que isto (suspensão, git demorado): os agentes não tiveram como sinalizar, e a carência recomeça.
const ATRASO_MS: i64 = 30_000;
const LIMITE_PADRAO: usize = 4;

/// O que muda entre o app e os testes. `lancar(pasta, exe, args)` abre o invólucro.
pub struct Ambiente {
    pub lancar: Box<dyn Fn(&Path, &Path, &[OsString]) -> Result<&'static str, String> + Send + Sync>,
    pub git: String,
    pub pasta_copias: PathBuf,
}

pub struct Relogio {
    pub partida_ms: i64,
    pub ultimo_ms: i64,
}

/// O plano que este tique vai lançar, já reivindicado (`preparando`).
struct Escolha {
    x: ExecucaoLinha,
    area: String,
    ramo: String,
    ramo_nosso: bool,
    /// ramos prontos e não integrados, com o repositório de cada um (D10)
    bases: Vec<(String, String)>,
}

pub fn ligar(estado: Weak<Estado>) {
    let corpo = move || {
        let Some(e) = estado.upgrade() else { return };
        ao_ligar(&e.dados, &e.mcp);
        drop(e);
        let amb = Ambiente { lancar: Box::new(terminal::lancar), git: git::programa(), pasta_copias: crate::http::config::pasta_do_app().join("copias") };
        let agora = crate::tempo::agora_ms();
        let mut relogio = Relogio { partida_ms: agora, ultimo_ms: agora };
        loop {
            std::thread::sleep(TIQUE);
            let Some(e) = estado.upgrade() else { return };
            passo(&amb, &e.dados, &e.mcp, crate::tempo::agora_ms(), &mut relogio);
        }
    };
    if let Err(e) = std::thread::Builder::new().name("motor-planejar".into()).spawn(corpo) {
        eprintln!("planejar: não consegui ligar o motor: {e}");
    }
}

/// O app reabriu: quem estava `preparando` perdeu o preparo e volta à fila; nenhum orquestrador segue rodando.
pub fn ao_ligar(dados: &Mutex<Dados>, mcp: &Mcp) {
    let mut d = travado(dados);
    let feito = (|| -> Resultado<()> {
        for x in d.execucoes_vivas()?.iter().filter(|x| x.situacao == "preparando") {
            d.execucao_mudar(&x.id, &Mudanca { so_de: &["preparando"], situacao: Some("marcada"), ..Mudanca::default() })?;
        }
        d.orq_zerar_rodando()
    })();
    if let Err(e) = feito {
        eprintln!("planejar: ao ligar o motor: {e}");
    }
    drop(d);
    mcp.planejar_mudou();
}

/// Um tique. Testável sem thread.
pub fn passo(amb: &Ambiente, dados: &Mutex<Dados>, mcp: &Mcp, agora_ms: i64, relogio: &mut Relogio) {
    if agora_ms - relogio.ultimo_ms > ATRASO_MS {
        relogio.partida_ms = agora_ms;
    }
    relogio.ultimo_ms = agora_ms;
    let escolha = varrer(&mut travado(dados), mcp, agora_ms, relogio);
    match escolha {
        Ok(Some(e)) => lancar_uma(amb, dados, mcp, e, agora_ms),
        Ok(None) => {}
        Err(e) => eprintln!("planejar: motor: {e}"),
    }
    super::orquestrador::passo(dados, mcp, agora_ms);
}

fn texto<'a>(v: &'a Value, campo: &str) -> &'a str {
    v.get(campo).and_then(Value::as_str).unwrap_or("")
}

fn lista<'a>(v: &'a Value, campo: &str) -> &'a [Value] {
    v.get(campo).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

/// "sm" é o próprio arquivo; os outros workspaces ficam em `workspaces`.
fn ws_de<'a>(tudo: &'a Value, ws: &str) -> Option<&'a Value> {
    if ws == "sm" {
        return Some(tudo);
    }
    lista(tudo, "workspaces").iter().find(|w| texto(w, "id") == ws)
}

fn bp_de<'a>(tudo: &'a Value, ws: &str, bp: &str) -> Option<&'a Value> {
    lista(ws_de(tudo, ws)?, "blueprints").iter().find(|b| texto(b, "id") == bp)
}

fn plano_de<'a>(bp: &'a Value, plano: &str) -> Option<&'a Value> {
    lista(bp, "acoes").iter().flat_map(|a| lista(a, "planos")).find(|q| texto(q, "id") == plano)
}

fn ref_de(q: &Value) -> &str {
    Some(texto(q, "ref")).filter(|r| !r.is_empty()).unwrap_or(texto(q, "id"))
}

fn acordar(d: &mut Dados, mcp: &Mcp, ws: &str, motivo: &str, plano_ref: &str) -> Resultado<()> {
    d.orq_pedir(ws, &format!("{motivo}:{plano_ref}"), false)?;
    mcp.planejar_mudou();
    Ok(())
}

/// Tudo o que precisa do `Mutex<Dados>`: passos 1 a 3 da seção 7, mais a reivindicação do plano escolhido.
fn varrer(d: &mut Dados, mcp: &Mcp, agora: i64, relogio: &Relogio) -> Resultado<Option<Escolha>> {
    let ctx = Contexto::local();
    let vivas = d.execucoes_vivas()?;
    let passou_carencia = agora - relogio.partida_ms >= CARENCIA_MS;
    for x in vivas.iter().filter(|x| x.situacao == "rodando" && passou_carencia) {
        let visto = x.ultimo_sinal.or(x.iniciada_em).unwrap_or(agora);
        if agora - visto > SEM_SINAL_MS && d.execucao_mudar(&x.id, &Mudanca { so_de: &["rodando"], situacao: Some("sumiu"), ..Mudanca::default() })? {
            acordar(d, mcp, &x.workspace_id, "sumiu", &x.plano_ref)?;
        }
    }
    if vivas.is_empty() {
        let tudo = d.ler_tudo(&ctx)?;
        let existem = blueprints_da_arvore(tudo);
        return limpar_sumidos(d, mcp, &existem).map(|_| None);
    }
    let tudo = d.ler_tudo(&ctx)?.clone();
    let mut restam = Vec::new();
    for x in vivas {
        let existe = bp_de(&tudo, &x.workspace_id, &x.blueprint_id).is_some_and(|bp| plano_de(bp, &x.plano_id).is_some());
        if existe {
            restam.push(x);
        } else {
            cancelar_removido(d, mcp, &x, agora)?;
        }
    }
    limpar_sumidos(d, mcp, &blueprints_da_arvore(&tudo))?;
    let mut vistos = HashSet::new();
    for ws in restam.iter().map(|x| x.workspace_id.as_str()).filter(|w| vistos.insert(*w)) {
        let Some(escolha) = escolher(d, &tudo, ws, agora)? else { continue };
        if d.execucao_reivindicar(&escolha.x.id)? {
            mcp.planejar_mudou();
            return Ok(Some(escolha));
        }
    }
    Ok(None)
}

fn blueprints_da_arvore(tudo: &Value) -> HashSet<String> {
    let raiz = lista(tudo, "blueprints").iter();
    let outros = lista(tudo, "workspaces").iter().flat_map(|w| lista(w, "blueprints"));
    raiz.chain(outros).map(|b| texto(b, "id").to_string()).collect()
}

/// Exclusão comum e `PUT` não passam pelo 409 do gesto: o agente de um plano que sumiu para no próximo sinal. O recado é o aviso que fica para quem lê.
fn cancelar_removido(d: &mut Dados, mcp: &Mcp, x: &ExecucaoLinha, agora: i64) -> Resultado<()> {
    if !d.execucao_cancelar(&x.id, agora)? {
        return Ok(());
    }
    d.execucao_mudar(&x.id, &Mudanca { motivo: Some("plano-removido"), ..Mudanca::default() })?;
    let frase = frases::texto(&Frase::PlanoRemovido { id: format!("{}|{}", x.blueprint_id, x.plano_ref) });
    d.recado_gravar(&NovoRecado { workspace_id: &x.workspace_id, blueprint_id: &x.blueprint_id, acao_id: &x.acao_id, execucao_id: &x.id, de: "agente", texto: &frase, em: agora })?;
    acordar(d, mcp, &x.workspace_id, "cancelada", &x.plano_ref)
}

/// Blueprint que sumiu da árvore e não tem mais execução viva: apaga versões, figuras e revisões.
fn limpar_sumidos(d: &mut Dados, mcp: &Mcp, existem: &HashSet<String>) -> Resultado<()> {
    let sumidos: Vec<String> = d.planejar_blueprints_com_linhas()?.into_iter().filter(|b| !existem.contains(b)).collect();
    if sumidos.is_empty() {
        return Ok(());
    }
    let vivas = d.execucoes_vivas()?;
    for bp in sumidos.iter().filter(|b| !vivas.iter().any(|x| x.blueprint_id == **b)) {
        d.planejar_limpar(bp)?;
        mcp.planejar_mudou();
    }
    Ok(())
}

/// Passo 3 e 4: o primeiro plano em ordem de documento que o escalonador manda trabalhar e ainda está só `marcada`.
fn escolher(d: &Dados, tudo: &Value, ws_id: &str, agora: i64) -> Resultado<Option<Escolha>> {
    let Some(ws) = ws_de(tudo, ws_id) else { return Ok(None) };
    let execs = d.execucoes_do_ws(ws_id)?;
    let esc: Vec<ExecEsc> = execs.iter().map(|x| ExecEsc { blueprint: x.blueprint_id.clone(), plano_id: x.plano_id.clone(), plano_ref: x.plano_ref.clone(), situacao: x.situacao.clone(), motivo: x.motivo.clone(), area: x.area.clone(), ramo: x.ramo.clone(), terminada_em: x.terminada_em, cancelada_em: x.cancelada_em }).collect();
    let limite = d.config_ler(&format!("limite:{ws_id}"))?.and_then(|t| t.parse().ok()).unwrap_or(LIMITE_PADRAO);
    let travados = travados_do_ws(d, ws)?;
    let tem_git = |p: &str| git::tem_git(Path::new(p));
    let pasta_de = |bp: &str| d.pasta_real_do_blueprint(ws_id, bp).ok().flatten();
    let (planos, entrada) = entrada_esc(ws, &esc, &travados, limite, &tem_git, &pasta_de, agora);
    let estados = escalonar(&planos, &entrada);
    let chave_de = |x: &ExecucaoLinha| bp_de(tudo, ws_id, &x.blueprint_id).and_then(|bp| plano_de(bp, &x.plano_id)).map(|q| format!("{}|{}", x.blueprint_id, ref_de(q)));
    let marcada = planos.iter().filter(|q| estados.get(&q.chave).is_some_and(|e| e.situacao == Situacao::Trabalhando)).find_map(|q| execs.iter().find(|x| x.situacao == "marcada" && chave_de(x).as_deref() == Some(q.chave.as_str())));
    let Some(x) = marcada else { return Ok(None) };
    let Some(q) = bp_de(tudo, ws_id, &x.blueprint_id).and_then(|bp| plano_de(bp, &x.plano_id)) else { return Ok(None) };
    let area = texto(q, "area").to_string();
    let depende: Vec<&str> = lista(q, "dependeDe").iter().filter_map(Value::as_str).collect();
    let prontas = execs.iter().filter(|o| o.situacao == "pronta" && !o.ramo.is_empty() && o.integrada_em.is_none());
    let bases = prontas.filter(|o| (o.blueprint_id == x.blueprint_id && depende.contains(&o.plano_ref.as_str())) || areas_conflitam(Some(area.as_str()), Some(o.area.as_str()))).map(|o| (o.ramo.clone(), o.repositorio.clone())).collect();
    // o ramo que este plano já teve é nosso: retomar e remarcar voltam a ele, e nenhum outro ramo do usuário é adotado
    let registrado = execs.iter().rev().find(|o| o.blueprint_id == x.blueprint_id && o.plano_id == x.plano_id && !o.ramo.is_empty()).map(|o| o.ramo.clone());
    let ramo_nosso = registrado.is_some();
    let ramo = registrado.unwrap_or_else(|| ids::ramo_de(&x.plano_ref));
    Ok(Some(Escolha { x: x.clone(), area, ramo, ramo_nosso, bases }))
}

/// `<copias>/<12 hex do bp>/<id real do plano>`: fora do repositório, caminho curto e sem nada vindo do texto. Id antigo fora dessa forma vira hash.
fn destino_da_copia(pasta_copias: &Path, bp: &str, plano: &str) -> PathBuf {
    let hash_bp = ids::sha256_hex(bp.as_bytes());
    let seguro = plano.len() <= 32 && !plano.is_empty() && plano.bytes().all(|c| c.is_ascii_alphanumeric());
    let nome = if seguro { plano.to_string() } else { ids::sha256_hex(plano.as_bytes())[..16].to_string() };
    pasta_copias.join(&hash_bp[..12]).join(nome)
}

fn falhar(dados: &Mutex<Dados>, mcp: &Mcp, x: &ExecucaoLinha, so_de: &[&str], motivo: &str, agora: i64) {
    let mut d = travado(dados);
    let feito = (|| -> Resultado<()> {
        if d.execucao_mudar(&x.id, &Mudanca { so_de, situacao: Some("falhou"), motivo: Some(motivo), terminada_em: Some(Some(agora)), ..Mudanca::default() })? {
            acordar(&mut d, mcp, &x.workspace_id, "falhou", &x.plano_ref)?;
        }
        Ok(())
    })();
    if let Err(e) = feito {
        eprintln!("planejar: motor: {e}");
    }
}

/// Passo 5, fora do `Mutex<Dados>` exceto nas duas gravações condicionais: quem cancelou durante o git faz `execucao_lancar` não achar linha, e nada é lançado.
fn lancar_uma(amb: &Ambiente, dados: &Mutex<Dados>, mcp: &Mcp, e: Escolha, agora: i64) {
    let x = &e.x;
    let principal = Path::new(&x.pasta_principal);
    if !principal.is_dir() {
        return falhar(dados, mcp, x, &["preparando"], "pasta-nao-existe", agora);
    }
    let repositorio = git::repositorio(&amb.git, principal).unwrap_or_default();
    let bases: Vec<String> = e.bases.iter().filter(|(_, repo)| *repo == repositorio).map(|(ramo, _)| ramo.clone()).collect();
    let destino = destino_da_copia(&amb.pasta_copias, &x.blueprint_id, &x.plano_id);
    let copia = match git::preparar_copia(&amb.git, principal, &e.ramo, e.ramo_nosso, &destino, &bases) {
        Ok(c) => c,
        Err(motivo) => return falhar(dados, mcp, x, &["preparando"], &motivo, agora),
    };
    let pasta = copia.pasta_trabalho.to_string_lossy().into_owned();
    let base = if copia.reusada { x.base.as_str() } else { copia.base.as_str() };
    let lancamento = Lancamento { area: &e.area, pasta_trabalho: &pasta, repositorio: &repositorio, ramo: &copia.ramo, base, agora_ms: agora };
    let lancou = travado(dados).execucao_lancar(&x.id, &lancamento);
    mcp.planejar_mudou();
    match lancou {
        Ok(true) => {}
        Ok(false) => return, // cancelou durante o git; ramo e cópia ficam
        Err(erro) => return eprintln!("planejar: motor: {erro}"),
    }
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("blueprint"));
    if let Err(motivo) = (amb.lancar)(&copia.pasta_trabalho, &exe, &terminal::args_do_agente(&x.id)) {
        falhar(dados, mcp, x, &["rodando"], &motivo, agora);
    }
}

/// Ao concluir: alteração sem commit ou nenhum commit no ramo vira aviso na linha, sem impedir (seção 7). Quem chama já soltou o `Mutex<Dados>`.
pub fn aviso_da_copia(dados: &Mutex<Dados>, mcp: &Mcp, x: &ExecucaoLinha) {
    let (trabalho, principal) = (Path::new(&x.pasta_trabalho), Path::new(&x.pasta_principal));
    if x.ramo.is_empty() || !trabalho.is_dir() || !principal.is_dir() {
        return;
    }
    let g = git::programa();
    let aviso = if git::sujo(&g, trabalho) {
        "copia-suja"
    } else if git::adiantado_de_head(&g, principal, &x.ramo) == Some(0) {
        "sem-commit"
    } else {
        return;
    };
    let mudou = travado(dados).execucao_mudar(&x.id, &Mudanca { so_de: &["pronta"], motivo: Some(aviso), ..Mudanca::default() });
    if matches!(mudou, Ok(true)) {
        mcp.planejar_mudou();
    }
}
