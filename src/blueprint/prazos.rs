//! Prazos no ritmo da IA: avisos (nunca erros) sobre ações longas demais, estimativas que não combinam com a duração e buracos entre ações novas.
//! A régua: um plano leva de 0,5 a 4 h de trabalho do agente; uma ação, de 1 a 3 dias; mais que isso só com motivo escrito nas instruções.
use super::datas::{dia_valido, dias_entre};
use serde_json::Value;

const DIAS_SEM_MOTIVO: i64 = 3;
const HORAS_MIN_POR_DIA: f64 = 1.0;
const HORAS_MAX_POR_DIA: f64 = 12.0;
const BURACO_MAX: i64 = 2;
/// trechos que contam como "motivo escrito" para uma ação passar de 3 dias (espera por pessoa, CI, publicação, terceiro)
const MOTIVOS: [&str; 8] = ["espera", "aguard", "aprova", "terceiro", "publica", "revisão do usuário", "revisao do usuario", " ci "];

/// Uma ação nova da proposta: início, fim e o rótulo para as mensagens.
pub struct Nova {
    pub inicio: String,
    pub fim: String,
    pub rotulo: String,
}

fn tem_motivo(observacao: &str) -> bool {
    let texto = format!(" {} ", observacao.to_lowercase());
    MOTIVOS.iter().any(|m| texto.contains(m))
}

fn horas(planos: &[Value]) -> Vec<f64> {
    planos.iter().filter_map(|p| p.get("horas").and_then(Value::as_f64)).collect()
}

/// Avisos de uma ação (nova ou existente) com as datas, os planos já normalizados (com `horas`) e as instruções finais.
pub fn avisos_da_acao(rotulo: &str, inicio: &str, fim: &str, planos: &[Value], observacao: &str, avisos: &mut Vec<String>) {
    if !(dia_valido(inicio) && dia_valido(fim)) || fim < inicio {
        return;
    }
    let dias = dias_entre(inicio, fim) + 1;
    if dias > DIAS_SEM_MOTIVO && !tem_motivo(observacao) {
        avisos.push(format!("{rotulo}: dura {dias} dias; ação de mais de {DIAS_SEM_MOTIVO} dias pede um motivo nas instruções (espera por pessoa, CI, publicação, terceiro) ou divisão em ações menores."));
    }
    let estimativas = horas(planos);
    if estimativas.is_empty() {
        return;
    }
    let soma: f64 = estimativas.iter().sum();
    if dias >= DIAS_SEM_MOTIVO && soma < dias as f64 * HORAS_MIN_POR_DIA {
        avisos.push(format!("{rotulo}: {soma} h de trabalho em {dias} dias parece frouxo (o esperado é de pelo menos {dias} h); confira as estimativas ou encurte o prazo."));
    }
    if soma > dias as f64 * HORAS_MAX_POR_DIA {
        avisos.push(format!("{rotulo}: {soma} h em {dias} dia(s) não cabe (cerca de {HORAS_MAX_POR_DIA} h por dia); alongue o prazo ou divida a ação."));
    }
}

/// Ações novas em sequência não deixam buracos de mais de 2 dias entre o fim de uma e o início da próxima.
pub fn avisos_de_buracos(mut novas: Vec<Nova>, avisos: &mut Vec<String>) {
    novas.sort_by(|a, b| a.inicio.cmp(&b.inicio));
    for par in novas.windows(2) {
        let buraco = dias_entre(&par[0].fim, &par[1].inicio) - 1;
        if buraco > BURACO_MAX {
            avisos.push(format!("{}: começa {buraco} dias depois do fim de {}; ações em sequência não deveriam deixar buracos de mais de {BURACO_MAX} dias.", par[1].rotulo, par[0].rotulo));
        }
    }
}
