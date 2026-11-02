//! Perfis de ferramentas do `--mcp` (`BLUEPRINT_MCP_PERFIL=executor|planejador|orquestrador|completo`, padrão completo): quem só executa não carrega no contexto a
//! descrição de `propor_blueprint`, e quem só planeja não carrega as de marcar. As instruções do servidor não mudam (os primeiros 512 caracteres
//! continuam valendo); a tabela e a medição estão em docs/mcp.md. Só o `--mcp` por stdio lê a variável; a rota HTTP do MCP é sempre completa, menos as ferramentas do Planejar (D14).
//! Nenhum perfil tem ferramenta de aprovar, marcar para iniciar, integrar ou ligar agente: isso é gesto do usuário na janela do app (D7).
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Perfil {
    Completo,
    Executor,
    Planejador,
    Orquestrador,
}

/// Executar e registrar: ler o blueprint, avisar o andamento, marcar e registrar o trabalho feito fora do plano (no lugar de propor).
const EXECUTOR: &[&str] = &[
    "listar_blueprints", "guia_de_planejamento", "ler_blueprint", "vincular_pasta",
    "registrar_progresso", "reportar_bloqueio", "registrar_trabalho",
    "marcar_concluido", "marcar_nao_concluido", "marcar_checkpoint_feito", "marcar_checkpoint_nao_feito",
    "ler_plano", "ler_respostas",
];

/// Planejar: ler, propor, mostrar a ideia (protótipo e apresentação) e responder à conversa do plano, que só refina e propõe.
const PLANEJADOR: &[&str] = &[
    "listar_blueprints", "guia_de_planejamento", "ler_blueprint", "vincular_pasta",
    "propor_blueprint", "anexar_prototipo", "anexar_apresentacao", "ler_apresentacao", "editar_apresentacao", "responder_conversa",
    "abrir_plano", "escrever_acao", "fechar_plano", "salvar_plano", "ler_plano", "ler_respostas",
];

/// O agente mestre do workspace (D12): lê, escreve plano e fala. Não conclui, não executa, não aprova.
const ORQUESTRADOR: &[&str] = &[
    "listar_blueprints", "guia_de_planejamento", "ler_blueprint", "ler_plano", "ler_respostas", "ler_workspace",
    "abrir_plano", "escrever_acao", "fechar_plano", "salvar_plano", "falar_com_voce",
];

static ATUAL: OnceLock<Perfil> = OnceLock::new();

impl Perfil {
    /// O valor da variável; ausente ou vazio = completo. Valor desconhecido é erro (melhor não subir do que expor o perfil errado).
    pub(super) fn ler(valor: Option<&str>) -> Result<Perfil, String> {
        match valor.map(str::trim).unwrap_or("") {
            "" | "completo" => Ok(Perfil::Completo),
            "executor" => Ok(Perfil::Executor),
            "planejador" => Ok(Perfil::Planejador),
            "orquestrador" => Ok(Perfil::Orquestrador),
            outro => Err(crate::tr!(
                "BLUEPRINT_MCP_PERFIL={outro:?} is not a profile: use executor, planejador, orquestrador or completo",
                "BLUEPRINT_MCP_PERFIL={outro:?} não é um perfil: use executor, planejador, orquestrador ou completo"
            )),
        }
    }

    pub(super) fn nome(self) -> &'static str {
        match self {
            Perfil::Completo => "completo",
            Perfil::Executor => "executor",
            Perfil::Planejador => "planejador",
            Perfil::Orquestrador => "orquestrador",
        }
    }

    pub(super) fn expoe(self, ferramenta: &str) -> bool {
        match self {
            Perfil::Completo => true,
            Perfil::Executor => EXECUTOR.contains(&ferramenta),
            Perfil::Planejador => PLANEJADOR.contains(&ferramenta),
            Perfil::Orquestrador => ORQUESTRADOR.contains(&ferramenta),
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
