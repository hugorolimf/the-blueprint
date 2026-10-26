//! O que muda quando um evento ou um hook do Claude Code vem de um membro (modo hospedado ou convidado do compartilhamento), e não do
//! usuário local: o agente leva o nome do membro (o Claude da Bia não se junta ao Claude do dono na folha), a sessão do Claude fica separada
//! por membro (um membro não mexe no lembrete da sessão de outro) e o blueprint vem do corpo, não da tabela de pastas, que é desta máquina.
use crate::contexto::{Acao, Contexto};
use serde_json::{json, Value};

/// O limite de `agente` (o contrato de registrar_progresso).
const AGENTE_MAX: usize = 40;

/// `nome · membro`, em até 40 caracteres (o nome do agente encurta para o do membro caber). No app local, o nome como veio.
pub fn com_membro(nome: &str, ctx: &Contexto) -> String {
    if ctx.local {
        return nome.to_string();
    }
    let membro: String = ctx.nome.chars().filter(|c| !c.is_control()).take(18).collect();
    let sufixo = format!(" · {}", membro.trim());
    if nome.ends_with(&sufixo) {
        return nome.chars().take(AGENTE_MAX).collect();
    }
    let cabe = AGENTE_MAX - sufixo.chars().count();
    format!("{}{sufixo}", nome.chars().take(cabe).collect::<String>().trim_end())
}

/// O corpo de um POST de membro, antes de a rota rodar: o progresso leva o nome do membro e os hooks, a sessão separada por membro.
pub fn do_membro(mut corpo: Value, recurso: &str, ctx: &Contexto) -> Value {
    if ctx.local {
        return corpo;
    }
    let tipo = corpo.get("tipo").and_then(Value::as_str).unwrap_or("");
    if recurso == "evento" && matches!(tipo, "progresso" | "bloqueio") {
        // agente que não é texto fica como veio: a validação de sempre recusa
        let nome = match corpo.get("agente") {
            None | Some(Value::Null) => Some("Claude".to_string()),
            Some(Value::String(s)) if s.trim().is_empty() => Some("Claude".to_string()),
            Some(Value::String(s)) => Some(s.trim().to_string()),
            Some(_) => None,
        };
        if let Some(n) = nome {
            corpo["agente"] = json!(com_membro(&n, ctx));
        }
    }
    if recurso.starts_with("hook/") {
        if let Some(s) = corpo.get("session_id").and_then(Value::as_str).filter(|s| !s.is_empty()) {
            corpo["session_id"] = json!(format!("{}:{s}", ctx.usuario));
        }
    }
    corpo
}

/// O blueprint de um hook de membro: o par `workspaceId`/`blueprintId` que o `blueprint hook` do convidado leu da linha `Blueprint:` da pasta,
/// só se o papel grava nele.
pub fn alvo(corpo: &Value, ctx: &Contexto) -> Option<(String, String)> {
    let texto = |k: &str| corpo.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let (ws, bp) = (texto("workspaceId"), texto("blueprintId"));
    (!ws.is_empty() && !bp.is_empty() && ctx.pode(Acao::Gravar, &ws, Some(&bp))).then_some((ws, bp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexto::{Acesso, Papel};

    fn bia() -> Contexto {
        Contexto::membro("m7", "Bia", vec![Acesso::novo("w2", "", Papel::Colaborador)])
    }

    #[test]
    fn o_agente_do_membro_leva_o_nome_dele_e_cabe_em_40() {
        assert_eq!(com_membro("Claude", &bia()), "Claude · Bia");
        assert_eq!(com_membro("Claude · Bia", &bia()), "Claude · Bia", "não repete o sufixo");
        let longo = com_membro(&"x".repeat(40), &bia());
        assert!(longo.ends_with(" · Bia") && longo.chars().count() == 40, "{longo}");
        assert_eq!(com_membro("Claude", &Contexto::local()), "Claude");
    }

    #[test]
    fn o_corpo_do_membro_separa_a_sessao_e_nomeia_o_agente() {
        let c = do_membro(json!({"tipo": "progresso", "texto": "x"}), "evento", &bia());
        assert_eq!(c["agente"], "Claude · Bia");
        let c = do_membro(json!({"tipo": "progresso", "agente": "Revisor"}), "evento", &bia());
        assert_eq!(c["agente"], "Revisor · Bia");
        assert_eq!(do_membro(json!({"tipo": "progresso", "agente": 3}), "evento", &bia())["agente"], 3, "a validação recusa depois");
        assert_eq!(do_membro(json!({"session_id": "s1"}), "hook/parada", &bia())["session_id"], "m7:s1");
        assert_eq!(do_membro(json!({"session_id": "s1"}), "hook/parada", &Contexto::local())["session_id"], "s1");
    }

    #[test]
    fn o_alvo_do_membro_e_so_onde_ele_grava() {
        assert_eq!(alvo(&json!({"workspaceId": "w2", "blueprintId": "p9"}), &bia()), Some(("w2".into(), "p9".into())));
        assert_eq!(alvo(&json!({"workspaceId": "sm", "blueprintId": "p1"}), &bia()), None);
        assert_eq!(alvo(&json!({"workspaceId": "w2"}), &bia()), None);
        let leitor = Contexto::membro("m8", "Sup", vec![Acesso::novo("w2", "", Papel::Supervisor)]);
        assert_eq!(alvo(&json!({"workspaceId": "w2", "blueprintId": "p9"}), &leitor), None, "supervisor só lê");
    }
}
