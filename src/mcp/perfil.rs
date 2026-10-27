//! Perfis de ferramentas do `--mcp` (`BLUEPRINT_MCP_PERFIL=executor|planejador|completo`, padrão completo): quem só executa não carrega no contexto a
//! descrição de `propor_blueprint`, e quem só planeja não carrega as de marcar. As instruções do servidor não mudam (os primeiros 512 caracteres
//! continuam valendo); a tabela e a medição estão em docs/mcp.md. Só o `--mcp` por stdio lê a variável; a rota HTTP do MCP é sempre completa.
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Perfil {
    Completo,
    Executor,
    Planejador,
}

/// Executar e registrar: ler o blueprint, avisar o andamento, marcar e registrar o trabalho feito fora do plano (no lugar de propor).
const EXECUTOR: &[&str] = &[
    "listar_blueprints", "guia_de_planejamento", "ler_blueprint", "vincular_pasta",
    "registrar_progresso", "reportar_bloqueio", "registrar_trabalho",
    "marcar_concluido", "marcar_nao_concluido", "marcar_checkpoint_feito", "marcar_checkpoint_nao_feito",
];

/// Planejar: ler, propor, mostrar a ideia (protótipo e apresentação) e responder à conversa do plano, que só refina e propõe.
const PLANEJADOR: &[&str] = &[
    "listar_blueprints", "guia_de_planejamento", "ler_blueprint", "vincular_pasta",
    "propor_blueprint", "anexar_prototipo", "anexar_apresentacao", "ler_apresentacao", "editar_apresentacao", "responder_conversa",
];

static ATUAL: OnceLock<Perfil> = OnceLock::new();

impl Perfil {
    /// O valor da variável; ausente ou vazio = completo. Valor desconhecido é erro (melhor não subir do que expor o perfil errado).
    pub(super) fn ler(valor: Option<&str>) -> Result<Perfil, String> {
        match valor.map(str::trim).unwrap_or("") {
            "" | "completo" => Ok(Perfil::Completo),
            "executor" => Ok(Perfil::Executor),
            "planejador" => Ok(Perfil::Planejador),
            outro => Err(crate::tr!(
                "BLUEPRINT_MCP_PERFIL={outro:?} is not a profile: use executor, planejador or completo",
                "BLUEPRINT_MCP_PERFIL={outro:?} não é um perfil: use executor, planejador ou completo"
            )),
        }
    }

    pub(super) fn nome(self) -> &'static str {
        match self {
            Perfil::Completo => "completo",
            Perfil::Executor => "executor",
            Perfil::Planejador => "planejador",
        }
    }

    pub(super) fn expoe(self, ferramenta: &str) -> bool {
        match self {
            Perfil::Completo => true,
            Perfil::Executor => EXECUTOR.contains(&ferramenta),
            Perfil::Planejador => PLANEJADOR.contains(&ferramenta),
        }
    }

    /// A lista de `tools/list` só com as ferramentas do perfil, na mesma ordem.
    pub(super) fn filtrar(self, ferramentas: Value) -> Value {
        match ferramentas {
            Value::Array(lista) => Value::Array(lista.into_iter().filter(|f| self.expoe(f["name"].as_str().unwrap_or(""))).collect()),
            outro => outro,
        }
    }
}

/// Lê a variável uma vez, ao iniciar o `--mcp`; Err com a mensagem para o valor inválido.
pub(super) fn iniciar() -> Result<(), String> {
    let perfil = Perfil::ler(std::env::var("BLUEPRINT_MCP_PERFIL").ok().as_deref())?;
    let _ = ATUAL.set(perfil);
    Ok(())
}

/// O perfil deste processo (completo se o `--mcp` não o fixou: rota HTTP e testes).
pub(super) fn atual() -> Perfil {
    ATUAL.get().copied().unwrap_or(Perfil::Completo)
}
