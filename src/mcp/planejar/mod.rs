//! As rotas do Planejar (`plano/*`, `planejar/*`, `gesto/*`, `execucao/*`), só no app local com janela nativa. Aqui fica o que vale para todas: quem alcança e quem atende cada uma.
//! Contrato: D7 (gesto com token), D14 (planejar só no app local). `POST /mcp` chama `rota_estado` direto, sem passar por `acesso::so_local`: por isso o `!ctx.local` aqui.
use super::permissao::so_loopback;
use super::Mcp;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

mod escrita;
mod execucao;
mod figuras;
mod gestos;
mod git;
mod leitura;
pub mod motor;
mod orquestrador;
mod prompt;
mod revisao;
mod terminal;
mod vista;

#[cfg(test)]
mod escrita_tests;
#[cfg(test)]
mod execucao_tests;
#[cfg(test)]
mod figuras_tests;
#[cfg(test)]
mod fluxo_tests;
#[cfg(test)]
mod gestos_tests;
#[cfg(test)]
mod git_tests;
#[cfg(test)]
mod leitura_tests;
#[cfg(test)]
mod motor_tests;
#[cfg(test)]
mod orquestrador_tests;
#[cfg(test)]
mod portas_tests;
#[cfg(test)]
mod prompt_tests;
#[cfg(test)]
mod terminal_tests;

/// O cabeçalho que prova o gesto do usuário: o valor só vai à janela, por chamada de função (`Mcp::ligar_janela`), nunca por env, arquivo, GET ou log.
pub const CABECALHO_GESTO: &str = "x-blueprint-gesto";

pub fn eh_rota(recurso: &str) -> bool {
    ["plano/", "planejar/", "gesto/", "execucao/"].iter().any(|p| recurso.starts_with(p))
}

/// O pedido traz o token desta abertura do app? Sem janela o token nunca saiu daqui, então ninguém acerta.
pub fn gesto_ok(mcp: &Mcp, pedido: &Pedido) -> bool {
    pedido.cabecalho(CABECALHO_GESTO).is_some_and(|t| t == mcp.gesto)
}

pub fn gesto_negado() -> Resposta {
    falha(403, crate::tr!("this only works from the app window", "isto só funciona pela janela do app")).codigo("gesto-negado").resposta()
}

/// 32 hexadecimais do gerador do sistema.
pub(super) fn novo_gesto() -> String {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).expect("o gerador aleatório do sistema falhou");
    b.iter().map(|x| format!("{x:02x}")).collect()
}

impl Mcp {
    /// Onde ficam as figuras dos planos (`<sha256>.<ext>`), com a mesma regra de `pasta_apresentacoes`.
    pub fn pasta_figuras(&self) -> PathBuf {
        match self.prototipos.file_name().is_some_and(|n| n == "prototipos") {
            true => self.prototipos.with_file_name("figuras"),
            false => self.prototipos.join("figuras"),
        }
    }

    /// A janela nativa abriu: devolve o token do gesto, por chamada de função, e passa a exigi-lo.
    pub fn ligar_janela(&self) -> String {
        self.tem_janela.store(true, Ordering::SeqCst);
        self.gesto.clone()
    }

    pub fn tem_janela(&self) -> bool {
        self.tem_janela.load(Ordering::SeqCst)
    }

    pub fn revisao_planejar(&self) -> u64 {
        self.revisao_planejar.load(Ordering::SeqCst)
    }

    /// Algo do Planejar mudou fora da árvore: o próximo `planejar/pulso` mostra.
    pub fn planejar_mudou(&self) {
        self.revisao_planejar.fetch_add(1, Ordering::SeqCst);
    }
}

fn nao_encontrado() -> ErroHttp {
    falha(404, crate::tr!("not found", "não encontrado")).codigo("nao-encontrado")
}

pub fn rota(mcp: &Mcp, dados: &Mutex<Dados>, pedido: &Pedido, recurso: &str, ctx: &Contexto) -> Resposta {
    if !ctx.local {
        return falha(404, crate::tr!("this route only exists in the local app", "esta rota só existe no app local")).codigo("so-local").resposta();
    }
    if recurso.starts_with("gesto/") && !(so_loopback(mcp, pedido) && gesto_ok(mcp, pedido)) {
        return gesto_negado();
    }
    if recurso.starts_with("plano/") && !mcp.tem_janela() {
        return falha(409, crate::tr!("no app window is open: use propor_blueprint", "nenhuma janela do app está aberta: use propor_blueprint")).codigo("sem-janela").resposta();
    }
    atender(mcp, dados, pedido, recurso).unwrap_or_else(|e| e.resposta())
}

fn atender(mcp: &Mcp, dados: &Mutex<Dados>, pedido: &Pedido, recurso: &str) -> Result<Resposta, ErroHttp> {
    let corpo: Value = if pedido.corpo.is_empty() { json!({}) } else { serde_json::from_slice(&pedido.corpo).map_err(|e| falha(400, crate::tr!("invalid JSON: {e}", "JSON inválido: {e}")).codigo("json-invalido"))? };
    let dono = match (pedido.metodo.as_str(), recurso) {
        ("POST", "plano/abrir" | "plano/escrever" | "plano/fechar" | "plano/recado") => escrita::rota,
        ("POST", "plano/figura") | ("GET", "planejar/figura") => figuras::rota,
        ("GET", "plano/ler" | "plano/respostas" | "plano/workspace" | "planejar/pulso" | "planejar/folha" | "planejar/chegou" | "planejar/agentes" | "planejar/config") => leitura::rota,
        ("POST", "gesto/revisar" | "gesto/colocar") => revisao::rota,
        ("POST", "gesto/marcar" | "gesto/cancelar" | "gesto/retomar" | "gesto/integrar" | "gesto/responder" | "gesto/limite" | "gesto/comando" | "gesto/pasta" | "gesto/orquestrador" | "gesto/descartar-escrita") => gestos::rota,
        ("POST", "execucao/partida" | "execucao/sinal" | "execucao/fim") => execucao::rota,
        _ => return Err(nao_encontrado()),
    };
    dono(mcp, dados, pedido, recurso, &corpo)
}
