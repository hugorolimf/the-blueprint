//! A entrada do escalonador, usada por `vista` e `motor` para a tela não divergir do motor. Pura: pasta e git chegam por função.
use super::escalonar::{Causa, EntradaEsc, PlanoEsc};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

/// Quanto o plano cancelado ainda segura a área, se o agente não saiu: o invólucro só encerra o filho no próximo sinal.
const CANCELADA_SEGURA_MS: i64 = 60_000;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExecEsc { pub blueprint: String, pub plano_id: String, pub plano_ref: String, pub situacao: String, pub motivo: String, pub area: String, pub ramo: String, pub terminada_em: Option<i64>, pub cancelada_em: Option<i64> }

impl ExecEsc {
    /// As mesmas situações do índice `execucao_viva`.
    fn viva(&self) -> bool {
        matches!(self.situacao.as_str(), "marcada" | "preparando" | "rodando" | "ajuda" | "sumiu" | "falhou")
    }

    /// O agente saiu, o trabalho ficou: conflita, mas não ocupa vaga.
    fn segura_area(&self, agora_ms: i64) -> bool {
        match self.situacao.as_str() {
            "sumiu" => true,
            "falhou" => !self.ramo.is_empty(),
            "cancelada" => self.terminada_em.is_none() && self.cancelada_em.is_some_and(|c| agora_ms - c < CANCELADA_SEGURA_MS),
            _ => false,
        }
    }

    fn pesa(&self, agora_ms: i64) -> bool {
        self.viva() || self.segura_area(agora_ms)
    }

    fn area(&self) -> Option<String> {
        Some(self.area.clone()).filter(|a| !a.is_empty())
    }
}

fn lista<'a>(v: &'a Value, campo: &str) -> &'a [Value] {
    v.get(campo).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn texto<'a>(v: &'a Value, campo: &str) -> &'a str {
    v.get(campo).and_then(Value::as_str).unwrap_or("")
}

/// Monta de todos os blueprints do workspace; os planos na ordem do documento, e os sintéticos no fim. `travados` já vêm em chaves (`<bp>|<ref>`), porque só quem chama sabe de que versão cada um é.
pub fn entrada_esc(ws: &Value, execs: &[ExecEsc], travados: &[String], limite: usize, tem_git: &dyn Fn(&str) -> bool, pasta_de: &dyn Fn(&str) -> Option<String>, agora_ms: i64) -> (Vec<PlanoEsc>, EntradaEsc) {
    let mut exclusivos: HashMap<String, Option<String>> = HashMap::new();
    let mut exclusivo = |bp: &str| exclusivos.entry(bp.to_string()).or_insert_with(|| pasta_de(bp).filter(|p| !tem_git(p))).clone();
    // Cancelado e marcado de novo: a viva vence a cancelada recente.
    let que_pesa = |bp: &str, plano: &str| {
        let do_plano = || execs.iter().filter(|x| x.blueprint == bp && x.plano_id == plano && x.pesa(agora_ms));
        do_plano().find(|x| x.viva()).or_else(|| do_plano().next())
    };

    let mut planos = Vec::new();
    let mut entrada = EntradaEsc { travados: travados.to_vec(), limite: Some(limite), ..EntradaEsc::default() };
    let mut chaves: HashMap<(String, String), String> = HashMap::new();
    for bp in lista(ws, "blueprints") {
        let bp_id = texto(bp, "id");
        for q in lista(bp, "acoes").iter().flat_map(|a| lista(a, "planos")) {
            let id = texto(q, "id");
            let r = Some(texto(q, "ref")).filter(|r| !r.is_empty()).unwrap_or(id);
            let chave = format!("{bp_id}|{r}");
            // Enquanto a execução vive, a área é a da linha dela: o que o agente recebeu ao ser lançado.
            let area = match que_pesa(bp_id, id) {
                Some(x) => x.area(),
                None => Some(texto(q, "area").to_string()).filter(|a| !a.is_empty()),
            };
            // Dependência que não existe mais fica como está: sem pronto, o plano espera, e `frases::avisos` fala dela.
            let depende_de = lista(q, "dependeDe").iter().filter_map(Value::as_str).map(|d| format!("{bp_id}|{d}")).collect();
            if q.get("feito").and_then(Value::as_bool) == Some(true) {
                entrada.prontos.push(chave.clone());
            }
            chaves.insert((bp_id.to_string(), id.to_string()), chave.clone());
            planos.push(PlanoEsc { chave, area, exclusivo: exclusivo(bp_id), depende_de });
        }
    }

    let mut causas = BTreeMap::new();
    for x in execs.iter().filter(|x| x.pesa(agora_ms)) {
        let par = (x.blueprint.clone(), x.plano_id.clone());
        let chave = match chaves.get(&par) {
            Some(c) => c.clone(),
            None => {
                // Plano apagado com execução viva: não sai de vaga nem de área até o motor cancelar e o agente sair.
                let r = if x.plano_ref.is_empty() { &x.plano_id } else { &x.plano_ref };
                let chave = format!("{}|{r}", x.blueprint);
                let area = que_pesa(&x.blueprint, &x.plano_id).and_then(ExecEsc::area);
                planos.push(PlanoEsc { chave: chave.clone(), area, exclusivo: exclusivo(&x.blueprint), depende_de: Vec::new() });
                chaves.insert(par, chave.clone());
                chave
            }
        };
        match x.situacao.as_str() {
            "marcada" => entrada.iniciados.push(chave.clone()),
            "preparando" | "rodando" => entrada.rodando.push(chave.clone()),
            "ajuda" => {
                entrada.rodando.push(chave.clone());
                entrada.bloqueados.push(chave.clone());
                causas.insert(chave.clone(), Causa::Ajuda);
            }
            "sumiu" => {
                entrada.bloqueados.push(chave.clone());
                causas.insert(chave.clone(), Causa::Sumiu);
            }
            "falhou" => {
                entrada.bloqueados.push(chave.clone());
                causas.insert(chave.clone(), Causa::Falhou(x.motivo.clone()));
            }
            _ => {}
        }
        if x.segura_area(agora_ms) {
            entrada.seguram_area.push(chave);
        }
    }
    entrada.causas = causas;
    (planos, entrada)
}
