//! O acesso do modo hospedado (`blueprint serve --hospedar`): resolve o `Contexto` de cada pedido (cookie da sessão ou chave do MCP), confere
//! HTTPS, Host e Origin, e guarda na memória o que não precisa de banco (tentativas inválidas, sessões encerradas). No modo local nada daqui
//! roda: o despacho entrega `Contexto::local()`. Contrato: docs/colaboracao.md, seções 3 a 5.
use crate::contexto::Contexto;
use std::collections::HashSet;
use std::sync::Mutex;

/// O estado em memória do acesso (campo `acesso` do `Estado`).
#[derive(Default)]
pub struct Acesso {
    /// quem foi removido ou teve o convite revogado: o SSE dessa pessoa fecha (`sessao_ativa`)
    encerrados: Mutex<HashSet<String>>,
}

impl Acesso {
    pub fn novo() -> Acesso {
        Acesso::default()
    }

    /// A sessão de `ctx` ainda vale? Lida da memória, para um fluxo SSE aberto fechar quem foi removido. No modo local, sempre.
    pub fn sessao_ativa(&self, ctx: &Contexto) -> bool {
        ctx.local || (ctx.autenticado() && !self.encerrados.lock().unwrap_or_else(|e| e.into_inner()).contains(&ctx.usuario))
    }

    /// `usuario` (id do membro ou "visitante:<convite>") perdeu o acesso: as conexões abertas dele fecham.
    pub fn encerrar(&self, usuario: &str) {
        self.encerrados.lock().unwrap_or_else(|e| e.into_inner()).insert(usuario.to_string());
    }
}

#[cfg(test)]
mod tests;
