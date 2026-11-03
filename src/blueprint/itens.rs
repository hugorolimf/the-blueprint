//! Os planos (planos) e checkpoints de UMA ação da proposta: completa o que veio com o que já existe, valida e normaliza.
use super::mover::noutra;
use super::texto::{campo, entre_aspas, infinitivo, lista, ou, str_de, texto, truncar};
use serde_json::{json, Value};
use std::collections::HashSet;

/// Tudo o que as duas validações precisam saber da ação e do blueprint.
pub struct Cenario<'a> {
    pub acoes: &'a [Value],
    pub existente: Option<&'a Value>,
    pub movidas: &'a HashSet<String>,
    pub rotulo: &'a str,
}

impl Cenario<'_> {
    /// O item com este id: na própria ação, ou (movimento) em outra do blueprint.
    fn acha(&self, lista_nome: &str, id: &str) -> Option<Value> {
        if id.is_empty() {
            return None;
        }
        let aqui = self.existente.and_then(|e| lista(e.get(lista_nome)).iter().find(|x| str_de(x, "id") == id));
        match aqui {
            Some(x) => Some(x.clone()),
            None => noutra(self.acoes, lista_nome, id, self.existente.map(|e| str_de(e, "id")).as_deref()),
        }
    }

    /// Itens que já existem na ação e não vieram na proposta (e não foram movidos): ficam como estão.
    fn mantidos(&self, lista_nome: &str, enviados: &[Value]) -> Vec<&Value> {
        lista(self.existente.and_then(|e| e.get(lista_nome)))
            .iter()
            .filter(|x| !self.movidas.contains(&str_de(x, "id")) && !enviados.iter().any(|t| campo(t, "id") == str_de(x, "id")))
            .collect()
    }
}

enum Item<'a> {
    Enviado(&'a Value),
    Mantido(&'a Value),
}

fn com_origem(mut saida: Value, ex: &Option<Value>) -> Value {
    if let Some(de) = ex.as_ref().and_then(|e| e.get("de")).and_then(Value::as_str).filter(|d| !d.is_empty()) {
        saida["de"] = Value::String(de.to_string());
    }
    saida
}

pub fn planos(f: &Value, c: &Cenario, erros: &mut Vec<String>, avisos: &mut Vec<String>) -> Vec<Value> {
    let enviadas = lista(f.get("planos"));
    let mantidas = c.mantidos("planos", enviadas);
    let itens: Vec<Item> = enviadas.iter().map(Item::Enviado).chain(mantidas.iter().map(|x| Item::Mantido(x))).collect();
    let (rotulo, total) = (c.rotulo, itens.len());
    if total == 0 {
        avisos.push(crate::tr!("{rotulo}: no plans (fine only for a skeleton the user will detail).", "{rotulo}: sem planos (ok só se for um esqueleto que o usuário vai detalhar)."));
    }
    if total > 0 && !(3..=8).contains(&total) {
        avisos.push(crate::tr!("{rotulo}: {total} plan(s); ideally 3 to 8.", "{rotulo}: {total} plano(s); o ideal é de 3 a 8."));
    }
    if !mantidas.is_empty() && !enviadas.is_empty() {
        avisos.push(crate::tr!("{rotulo}: {} existing plan(s) were not sent and stay as they are.", "{rotulo}: {} plano(s) existente(s) não vieram e ficam como estão.", mantidas.len()));
    }
    let mut vistos: HashSet<String> = HashSet::new();
    let mut saida = Vec::new();
    for (j, item) in itens.iter().enumerate() {
        let t = match item {
            Item::Mantido(x) => {
                vistos.insert(str_de(x, "texto").to_lowercase()); // não veio: segue igual, sem revalidar
                saida.push(json!({"id": str_de(x, "id"), "texto": str_de(x, "texto"), "descricao": str_de(x, "descricao")}));
                continue;
            }
            Item::Enviado(t) => *t,
        };
        let ex = c.acha("planos", &campo(t, "id"));
        let tx = ou(campo(t, "texto"), ex.as_ref().map(|e| str_de(e, "texto")).unwrap_or_default());
        let rot = crate::tr!("{rotulo}, plan {}{}", "{rotulo}, plano {}{}", j + 1, entre_aspas(&tx));
        if tx.is_empty() {
            erros.push(crate::tr!("{rot}: the title (`texto`) is missing.", "{rot}: falta o título (`texto`)."));
        } else {
            if !infinitivo(&tx) {
                avisos.push(crate::tr!("{rot}: the title should start with a verb in the infinitive.", "{rot}: o título deveria começar com verbo no infinitivo."));
            }
            if vistos.contains(&tx.to_lowercase()) {
                erros.push(crate::tr!("{rot}: title repeated in the action.", "{rot}: título repetido na ação."));
            }
            vistos.insert(tx.to_lowercase());
        }
        let descricao = ou(campo(t, "descricao"), ex.as_ref().map(|e| str_de(e, "descricao")).unwrap_or_default());
        if descricao.is_empty() && ex.is_none() {
            erros.push(crate::tr!("{rot}: the description is missing (scope, expected result and validation).", "{rot}: falta a descrição (escopo, resultado esperado e validação)."));
        } else if !descricao.is_empty() && !campo(t, "descricao").is_empty() && !descricao.to_lowercase().contains("valida") {
            avisos.push(crate::tr!("{rot}: the description does not say how to validate (include \"Validation: ...\").", "{rot}: a descrição não diz como validar (inclua \"Validação: ...\")."));
        }
        let id = campo(t, "id");
        if !id.is_empty() && ex.is_none() {
            erros.push(crate::tr!("{rot}: id \"{id}\" does not exist in this action or another one of the blueprint (leave it empty for a new plan).", "{rot}: o id \"{id}\" não existe nesta ação nem em outra do blueprint (deixe vazio num plano novo)."));
        }
        saida.push(com_origem(json!({"id": id, "texto": tx, "descricao": descricao}), &ex));
    }
    saida
}

pub fn checkpoints(f: &Value, c: &Cenario, erros: &mut Vec<String>) -> Vec<Value> {
    let enviados = lista(f.get("checkpoints"));
    let mantidos = c.mantidos("checkpoints", enviados);
    let itens: Vec<Item> = enviados.iter().map(Item::Enviado).chain(mantidos.iter().map(|x| Item::Mantido(x))).collect();
    let rotulo = c.rotulo;
    if itens.len() > 8 {
        erros.push(crate::tr!("{rotulo}: {} checkpoints; the maximum is 8 (ideally 1 to 3).", "{rotulo}: {} checkpoints; o máximo é 8 (o ideal é de 1 a 3).", itens.len()));
    }
    let mut saida = Vec::new();
    for (j, item) in itens.iter().take(8).enumerate() {
        let m = match item {
            Item::Mantido(x) => {
                saida.push(json!({"id": str_de(x, "id"), "nome": str_de(x, "nome")}));
                continue;
            }
            Item::Enviado(m) => *m,
        };
        let ex = c.acha("checkpoints", &campo(m, "id"));
        let nm = ou(campo(m, "nome"), ex.as_ref().map(|e| str_de(e, "nome")).unwrap_or_default());
        let rot = crate::tr!("{rotulo}, checkpoint {}{}", "{rotulo}, checkpoint {}{}", j + 1, entre_aspas(&nm));
        if nm.is_empty() {
            erros.push(crate::tr!("{rot}: the name is missing.", "{rot}: falta o nome."));
        }
        let id = texto(m.get("id"));
        if !id.is_empty() && ex.is_none() {
            erros.push(crate::tr!("{rot}: id \"{id}\" does not exist in this action or another one of the blueprint (leave it empty for a new checkpoint).", "{rot}: o id \"{id}\" não existe nesta ação nem em outra do blueprint (deixe vazio num checkpoint novo)."));
        }
        saida.push(com_origem(json!({"id": id, "nome": truncar(&nm, 120)}), &ex));
    }
    saida
}
