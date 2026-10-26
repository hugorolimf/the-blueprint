//! As rotas da atualização: o estado (GET), o pedido de verificação, o clique em Atualizar e a preferência da verificação automática. Só esta máquina (Host e
//! Origin em loopback, como /api/mcp), todo POST exige JSON e nenhum corpo escolhe URL ou caminho: o que se instala é sempre o instalador que o app já baixou
//! e conferiu (src/atualizador). Instalar só acontece no POST de `instalar`, que é o clique do usuário.
use super::pedido::{Pedido, Resposta};
use super::Estado;
use crate::atualizador::{copiar_banco, lancar_instalador, Recusa};
use crate::mcp::permissao::so_loopback;
use serde_json::{json, Value};

pub fn rota(estado: &Estado, pedido: &Pedido) -> Resposta {
    if !so_loopback(&estado.mcp, pedido) {
        return Resposta::texto(403, "text/plain", "a atualização só funciona nesta máquina");
    }
    let a = &estado.atualizador;
    match (pedido.metodo.as_str(), pedido.caminho.as_str()) {
        ("GET", "/api/atualizacao") => Resposta::json(200, &a.json()).com("cache-control", "no-store"),
        ("POST", _) if !pedido.eh_json() => Resposta::texto(415, "text/plain", "envie application/json"),
        ("POST", "/api/atualizacao/verificar") => {
            a.verificar_em_segundo_plano(true);
            Resposta::json(202, &json!({"ok": true}))
        }
        ("POST", "/api/atualizacao/instalar") => match instalar_agora(estado) {
            Ok(()) => Resposta::json(200, &json!({"ok": true})),
            Err(r) => Resposta::json(r.status, &json!({"erro": r.mensagem, "codigo": r.codigo})),
        },
        ("POST", "/api/atualizacao/automatica") => match serde_json::from_slice::<Value>(&pedido.corpo).ok().and_then(|v| v["ligada"].as_bool()) {
            Some(ligada) => {
                a.definir_automatica(ligada);
                Resposta::json(200, &a.json())
            }
            None => Resposta::json(400, &json!({"erro": "envie {\"ligada\": true|false}"})),
        },
        ("GET" | "POST", _) => Resposta::texto(404, "text/plain", "não encontrado"),
        _ => Resposta::texto(405, "text/plain", "método não permitido"),
    }
}

/// Por que não fechar o app agora: a fila do MCP é só memória e some ao sair. (O que o agente registrar com o app fechado fica guardado em arquivo e entra ao
/// abrir: src/mcp/pendentes.rs, então um agente trabalhando não impede a atualização.) Nem com sessão viva no terminal do app: sair derruba o Claude que roda nela.
pub(crate) fn bloqueio(estado: &Estado) -> Option<String> {
    let n = estado.mcp.itens().len();
    if n > 0 {
        return Some(format!("Há {n} aviso(s) do agente (propostas ou marcações) que a tela ainda não aplicou: fechar o app agora os perderia. Abra o blueprint, aplique ou descarte e clique em Atualizar de novo."));
    }
    let t = estado.terminais.vivas();
    (t > 0).then(|| format!("Há {t} sessão(ões) rodando no terminal do app: atualizar fecha o app e as encerra. Feche-as no painel Terminal e clique em Atualizar de novo."))
}

/// O clique em Atualizar: confere a fila, copia o banco, roda o instalador e manda o app sair. Usado pela rota, pela bandeja e (por HTTP) pelo `blueprint atualizar --instalar`.
pub fn instalar_agora(estado: &Estado) -> Result<(), Recusa> {
    let pasta_backups = estado.config.banco.parent().filter(|p| !p.as_os_str().is_empty()).map_or_else(|| "backups".into(), |p| p.join("backups"));
    let backup = |versao: &str| {
        let dados = estado.dados.lock().unwrap_or_else(|e| e.into_inner());
        copiar_banco(dados.conexao(), &pasta_backups, versao).map(|_| ())
    };
    let sair = estado.atualizador.instalar(bloqueio(estado), &backup, &lancar_instalador)?;
    if sair {
        if let Some(sair) = estado.ao_sair.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            sair();
        }
    }
    Ok(())
}
