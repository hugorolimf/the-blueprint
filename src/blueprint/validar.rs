//! `validar_proposta`: porte de `validarPlano` do antigo plano.mjs. `blueprint` é o blueprint atual (None num blueprint novo);
//! com erros a proposta não é aceita (`proposta` fica None).
use super::datas::dia_valido;
use super::itens::{checkpoints, planos, Cenario};
use super::mover::movidas;
use super::prazos::{avisos_da_acao, avisos_de_buracos, Nova};
use super::projetos::{escolher, projetos_novos, Escolha};
use super::remocoes::remocoes;
use super::texto::{campo, entre_aspas, lista, ou, str_de, truncar, verdade};
use serde_json::{json, Map, Value};

pub struct Validacao {
    pub erros: Vec<String>,
    pub avisos: Vec<String>,
    pub proposta: Option<Value>,
}

pub fn validar_proposta(entrada: &Value, blueprint: Option<&Value>, hoje: &str) -> Validacao {
    validar(entrada, blueprint, hoje, false)
}

/// `registro`: a mesma validação para o registro do que já foi feito (`registro.rs`): sem os avisos de datas passadas, prazos e quantidade de planos
/// (o trabalho já aconteceu), e sem recusar `feito`, que só vale ali.
pub(super) fn validar(entrada: &Value, blueprint: Option<&Value>, hoje: &str, registro: bool) -> Validacao {
    let (mut erros, mut avisos) = (Vec::new(), Vec::new());
    if !(entrada.is_object() || entrada.is_array()) {
        return Validacao { erros: vec!["A proposta precisa ser um objeto com a lista `acoes`.".into()], avisos, proposta: None };
    }
    let mut proposta = Map::new();
    let dados_blueprint = entrada.get("blueprint");
    match blueprint {
        None => {
            let nome = campo(dados_blueprint.unwrap_or(&Value::Null), "nome");
            if nome.is_empty() {
                erros.push("Blueprint novo: informe `blueprint: { nome, descricao }`.".into());
            } else {
                proposta.insert("blueprint".into(), json!({"nome": truncar(&nome, 120), "descricao": truncar(&campo(dados_blueprint.unwrap_or(&Value::Null), "descricao"), 3000)}));
            }
        }
        Some(_) if verdade(dados_blueprint.and_then(|p| p.get("nome"))) => avisos.push("`blueprint` é ignorado em blueprint existente (o nome só se define na criação).".into()),
        Some(_) => {}
    }
    let projetos_novos = projetos_novos(dados_blueprint, blueprint, &mut erros);
    if !projetos_novos.is_empty() {
        proposta.insert("projetos".into(), Value::Array(projetos_novos.clone()));
    }
    proposta.insert("mensagem".into(), Value::String(truncar(&campo(entrada, "mensagem"), 3000)));
    let pedidos = lista(entrada.get("remover"));
    let enviadas = lista(entrada.get("acoes"));
    if enviadas.is_empty() && pedidos.is_empty() && lista(entrada.get("mover")).is_empty() {
        erros.push("`acoes` é obrigatório: uma lista com ao menos uma ação (ou `remover`/`mover`).".into());
        return Validacao { erros, avisos, proposta: None };
    }
    if enviadas.len() > 20 {
        erros.push("Ações demais (máximo 20 por proposta).".into());
    }
    let acoes_proj: &[Value] = blueprint.map(|p| lista(p.get("acoes"))).unwrap_or(&[]);
    let movidas = movidas(acoes_proj, enviadas);

    // ação existente: o que não vier fica como está (campos, planos e checkpoints); assim uma proposta parcial nunca apaga conteúdo
    let mut acoes_ok: Vec<Value> = Vec::new();
    let mut novas: Vec<Nova> = Vec::new();
    for (i, f) in enviadas.iter().take(20).enumerate() {
        let id_f = campo(f, "id");
        let existente = if id_f.is_empty() { None } else { acoes_proj.iter().find(|x| str_de(x, "id") == id_f) };
        let ex = |chave: &str| existente.map(|e| str_de(e, chave)).unwrap_or_default();
        let nome = ou(campo(f, "nome"), ex("nome"));
        let rotulo = format!("Ação {}{}", i + 1, entre_aspas(&nome));
        if !id_f.is_empty() && existente.is_none() {
            let cru = f.get("id").and_then(Value::as_str).unwrap_or(&id_f);
            erros.push(format!("{rotulo}: o id \"{cru}\" não existe neste blueprint (deixe vazio numa ação nova; use só ids de ler_blueprint)."));
        }
        if nome.is_empty() {
            erros.push(format!("{rotulo}: falta o nome."));
        }
        let (inicio, fim) = (ou(campo(f, "inicio"), ex("inicio")), ou(campo(f, "fim"), ex("fim")));
        if !dia_valido(&inicio) {
            erros.push(format!("{rotulo}: `inicio` deve ser uma data aaaa-mm-dd válida (recebido \"{inicio}\")."));
        }
        if !dia_valido(&fim) {
            erros.push(format!("{rotulo}: `fim` deve ser uma data aaaa-mm-dd válida (recebido \"{fim}\")."));
        }
        if dia_valido(&inicio) && dia_valido(&fim) && fim < inicio {
            erros.push(format!("{rotulo}: `fim` ({fim}) é anterior a `inicio` ({inicio})."));
        }
        if existente.is_none() && !registro && dia_valido(&inicio) && inicio.as_str() < hoje {
            avisos.push(format!("{rotulo}: começa em {inicio}, antes de hoje ({hoje}); só faça isso se o usuário pediu."));
        }
        let status = match f.get("status") {
            None if existente.is_some() => if ex("status") == "planejado" { "planejado".to_string() } else { "andamento".to_string() },
            None => "planejado".to_string(),
            Some(Value::String(s)) => s.clone(),
            Some(outro) => outro.to_string(),
        };
        let status_ok = status == "planejado" || status == "andamento";
        if !status_ok {
            erros.push(format!("{rotulo}: `status` deve ser \"planejado\" ou \"andamento\" (o status concluído não pode ser proposto; recebido \"{status}\")."));
        }
        if !registro {
            for item in lista(f.get("planos")).iter().chain(lista(f.get("checkpoints"))) {
                if item.get("feito") == Some(&Value::Bool(true)) {
                    erros.push(format!("{rotulo}: `feito` não vale numa proposta (plano é trabalho futuro); o que já foi feito vai em registrar_trabalho."));
                    break;
                }
            }
        }
        let cenario = Cenario { acoes: acoes_proj, existente, movidas: &movidas, rotulo: &rotulo, inicio: &inicio, fim: &fim };
        let mut calados = Vec::new(); // no registro os avisos dos planos (quantidade, estimativa, "validação") só atrapalham
        let planos_ok = planos(f, &cenario, &mut erros, if registro { &mut calados } else { &mut avisos });
        let checkpoints_ok = checkpoints(f, &cenario, &mut erros);
        let observacao = ou(campo(f, "observacao"), ex("obs"));
        if !registro {
            avisos_da_acao(&rotulo, &inicio, &fim, &planos_ok, &observacao, &mut avisos);
        }
        if existente.is_none() {
            novas.push(Nova { inicio: inicio.clone(), fim: fim.clone(), rotulo: rotulo.clone() });
        }
        let mut acao_ok = json!({
            "id": existente.map(|e| str_de(e, "id")).unwrap_or_default(), "nome": truncar(&nome, 120), "inicio": inicio, "fim": fim,
            "status": if status_ok { status } else { "planejado".to_string() }, "observacao": truncar(&observacao, 20000),
            "planos": planos_ok, "checkpoints": checkpoints_ok,
        });
        match escolher(f, blueprint, &projetos_novos, &rotulo, &mut erros) { // ausente = a ação fica onde está; null = sai do projeto
            Escolha::Mantem => {}
            Escolha::Nenhum => acao_ok["projeto"] = Value::Null,
            Escolha::Projeto(projeto) => acao_ok["projeto"] = projeto,
        }
        acoes_ok.push(acao_ok);
    }

    if !registro {
        avisos_de_buracos(novas.into_iter().filter(|n| dia_valido(&n.inicio) && dia_valido(&n.fim)).collect(), &mut avisos);
    }
    let mut remover = Vec::new();
    if !pedidos.is_empty() && blueprint.is_none() {
        avisos.push("`remover` é ignorado em blueprint novo.".into());
    } else {
        let ids_na_proposta: Vec<String> = acoes_ok.iter().map(|f| str_de(f, "id")).collect();
        remover = remocoes(pedidos, acoes_proj, &movidas, &ids_na_proposta, &mut erros, &mut avisos);
    }
    proposta.insert("acoes".into(), Value::Array(acoes_ok));
    proposta.insert("remover".into(), Value::Array(remover));
    let proposta = if erros.is_empty() { Some(Value::Object(proposta)) } else { None };
    Validacao { erros, avisos, proposta }
}
