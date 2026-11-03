//! A execução grava direto na árvore (D8), porque a janela nativa não consome a fila. O estado final é o mesmo de `web/js/mcp.js`. Quem chama já validou (`acoes::evento_sem_fila`) e conferiu o segredo.
//! Toda gravação é uma transação; depois, `tocar_revisao` sobe a versão dos dados (D4).
use crate::dados::formato::{CHECKPOINTS, PLANOS};
use crate::dados::item::{self, Lido};
use crate::dados::linhas::Objeto;
use crate::dados::Dados;
use crate::erro::{Erro, Resultado};
use serde_json::{json, Value};

/// Os mesmos limites de `adicionarComentarioAgente` (`web/js/rede.js`).
const MAX_TEXTO: usize = 12_000;
const MAX_COMENTARIOS: usize = 20;

fn sem_item(tipo: &str, id: &str) -> Erro {
    Erro::NaoEncontrado(format!("{tipo} \"{id}\" não existe"))
}

/// `agente` do plano como objeto (cria se faltar ou se tiver outro tipo).
fn agente_de(obj: &mut Objeto) -> &mut Objeto {
    let agente = obj.entry("agente").or_insert_with(|| json!({}));
    if !agente.is_object() {
        *agente = json!({});
    }
    agente.as_object_mut().expect("acabou de virar objeto")
}

fn comentar(agente: &mut Objeto, texto: &str, tipo: &str, em: &str) {
    if texto.is_empty() {
        return;
    }
    let lista = agente.entry("comentarios").or_insert_with(|| json!([]));
    if !lista.is_array() {
        *lista = json!([]);
    }
    let itens = lista.as_array_mut().expect("acabou de virar lista");
    let curto: String = texto.chars().take(MAX_TEXTO).collect();
    itens.push(json!({"texto": curto, "em": em, "tipo": tipo, "sessaoId": ""}));
    if itens.len() > MAX_COMENTARIOS {
        itens.drain(..itens.len() - MAX_COMENTARIOS);
    }
}

impl Dados {
    /// Plano feito, com a evidência como comentário `relatorio`.
    pub fn concluir_por_execucao(&mut self, plano_id: &str, evidencia: &str, em: &str) -> Resultado<()> {
        let tx = self.conn.transaction()?;
        let Lido { listas, mut obj, .. } = item::ler(&tx, &PLANOS, plano_id)?.ok_or_else(|| sem_item("plano", plano_id))?;
        obj.insert("feito".into(), Value::Bool(true));
        let agente = agente_de(&mut obj);
        agente.insert("estado".into(), json!("concluido"));
        agente.insert("atualizadoEm".into(), json!(em));
        comentar(agente, evidencia, "relatorio", em);
        item::escrever(&tx, &PLANOS, plano_id, listas, obj)?;
        tx.commit()?;
        self.tocar_revisao();
        Ok(())
    }

    /// O agente desfez a própria conclusão: o plano volta a pendente, com o motivo nos comentários.
    pub fn desfazer_por_execucao(&mut self, plano_id: &str, motivo: &str, em: &str) -> Resultado<()> {
        let tx = self.conn.transaction()?;
        let Lido { listas, mut obj, .. } = item::ler(&tx, &PLANOS, plano_id)?.ok_or_else(|| sem_item("plano", plano_id))?;
        obj.insert("feito".into(), Value::Bool(false));
        let agente = agente_de(&mut obj);
        agente.insert("estado".into(), json!("pendente"));
        agente.insert("atualizadoEm".into(), json!(em));
        if !motivo.is_empty() {
            comentar(agente, &crate::tr!("Not completed: {motivo}", "Não concluído: {motivo}"), "comentario", em);
        }
        item::escrever(&tx, &PLANOS, plano_id, listas, obj)?;
        tx.commit()?;
        self.tocar_revisao();
        Ok(())
    }

    /// `feito`: grava a evidência no checkpoint. Não feito: desfaz, e o motivo (`evidencia`) fica só na linha do tempo, com o resto do evento.
    pub fn checkpoint_por_execucao(&mut self, checkpoint_id: &str, feito: bool, evidencia: &str, em: &str) -> Resultado<()> {
        let tx = self.conn.transaction()?;
        let Lido { listas, mut obj, .. } = item::ler(&tx, &CHECKPOINTS, checkpoint_id)?.ok_or_else(|| sem_item("checkpoint", checkpoint_id))?;
        obj.insert("feito".into(), Value::Bool(feito));
        if feito {
            obj.insert("evidencia".into(), json!(evidencia));
        } else {
            obj.remove("evidencia");
        }
        item::escrever(&tx, &CHECKPOINTS, checkpoint_id, listas, obj)?;
        tx.commit()?;
        self.tocar_revisao();
        Ok(())
    }
}
