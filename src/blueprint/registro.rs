//! Registro do trabalho feito fora do plano (`registrar_trabalho`): o que o agente já fez, com os planos e checkpoints já marcados como feitos (e a evidência),
//! mais o que ficou pendente. NÃO é proposta (proposta é trabalho futuro): a validação é a mesma, mas aceita `feito` + `evidencia` e não reclama de datas passadas.
//! Nada aqui altera dados: devolve erros, avisos e o registro normalizado; quem adiciona ao blueprint é o usuário, na tela.
use super::texto::{campo, lista, truncar};
use super::validar::{validar, Validacao};
use serde_json::Value;

const EVIDENCIA_MAX: usize = 1000;

/// Valida o registro contra o blueprint existente (um registro nunca cria blueprint, remove nem move nada).
pub fn validar_registro(entrada: &Value, blueprint: &Value, hoje: &str) -> Validacao {
    let mut erros = Vec::new();
    if !lista(entrada.get("remover")).is_empty() || !lista(entrada.get("mover")).is_empty() {
        erros.push("Um registro só relata o que foi feito: `remover` e `mover` não valem aqui (use propor_blueprint).".into());
    }
    let enviadas = lista(entrada.get("acoes"));
    let mut feitos = 0;
    for (i, f) in enviadas.iter().take(20).enumerate() {
        for (lista_nome, rotulo) in [("planos", "plano"), ("checkpoints", "checkpoint")] {
            for (j, item) in lista(f.get(lista_nome)).iter().enumerate() {
                let onde = format!("Ação {}, {rotulo} {}", i + 1, j + 1);
                match item.get("feito") {
                    None | Some(Value::Null) | Some(Value::Bool(false)) => {}
                    Some(Value::Bool(true)) => {
                        feitos += 1;
                        let evidencia = campo(item, "evidencia");
                        if evidencia.is_empty() {
                            erros.push(format!("{onde}: `feito: true` exige a `evidencia` (o comando ou teste rodado e o resultado)."));
                        } else if evidencia.chars().count() > EVIDENCIA_MAX {
                            erros.push(format!("{onde}: `evidencia` com {} caracteres; o máximo é {EVIDENCIA_MAX}.", evidencia.chars().count()));
                        }
                    }
                    Some(_) => erros.push(format!("{onde}: `feito` deve ser true ou false.")),
                }
            }
        }
    }
    if feitos == 0 && erros.is_empty() {
        erros.push("O registro não tem nada feito: marque `feito: true` (com `evidencia`) no que já foi feito. Trabalho futuro vai em propor_blueprint.".into());
    }
    let mut r = validar(entrada, Some(blueprint), hoje, true);
    r.erros.splice(0..0, erros);
    if !r.erros.is_empty() {
        r.proposta = None;
        return r;
    }
    if let Some(Value::Object(proposta)) = r.proposta.as_mut() {
        // as ações saem na ordem enviada e, em cada uma, os itens enviados vêm antes dos que já existiam e não vieram: o índice do enviado é o do normalizado
        if let Some(Value::Array(acoes)) = proposta.get_mut("acoes") {
            for (acao, enviada) in acoes.iter_mut().zip(enviadas) {
                for lista_nome in ["planos", "checkpoints"] {
                    let normalizados = acao[lista_nome].as_array_mut().into_iter().flatten();
                    for (normalizado, item) in normalizados.zip(lista(enviada.get(lista_nome))) {
                        if item.get("feito") == Some(&Value::Bool(true)) {
                            normalizado["feito"] = Value::Bool(true);
                            normalizado["evidencia"] = Value::String(truncar(&campo(item, "evidencia"), EVIDENCIA_MAX));
                        }
                    }
                }
            }
        }
        proposta.insert("registro".into(), Value::Bool(true));
    }
    r
}
