//! Horas por plano e prazos no ritmo da IA: as estimativas são validadas e os prazos frouxos ou apertados só geram avisos.
use super::{dias_entre, ms_do_iso, validar_proposta};
use serde_json::{json, Value};

const HOJE: &str = "2026-10-21";

fn plano(n: u32, extra: Value) -> Value {
    let mut p = json!({"texto": format!("Criar item {n}"), "descricao": "Escopo: x. Resultado: y. Validação: z."});
    for (k, v) in extra.as_object().unwrap() {
        p[k] = v.clone();
    }
    p
}

fn acao(inicio: &str, fim: &str, observacao: &str, planos: Vec<Value>) -> Value {
    json!({"nome": "Ação", "inicio": inicio, "fim": fim, "observacao": observacao, "planos": planos, "checkpoints": []})
}

fn tres(horas: [f64; 3]) -> Vec<Value> {
    horas.iter().enumerate().map(|(i, h)| plano(i as u32 + 1, json!({"horas": h}))).collect()
}

fn novo(acoes: Vec<Value>) -> super::Validacao {
    validar_proposta(&json!({"blueprint": {"nome": "N", "descricao": "d"}, "acoes": acoes}), None, HOJE)
}

fn tem(avisos: &[String], trecho: &str) -> bool {
    avisos.iter().any(|a| a.contains(trecho))
}

#[test]
fn horas_validas_viram_numero_e_aceitam_virgula() {
    let r = novo(vec![acao("2026-10-22", "2026-10-23", "", vec![plano(1, json!({"horas": 2})), plano(2, json!({"horas": "1,5"})), plano(3, json!({}))])]);
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    let planos = &r.proposta.unwrap()["acoes"][0]["planos"];
    assert_eq!((planos[0]["horas"].as_f64(), planos[1]["horas"].as_f64(), planos[2].get("horas")), (Some(2.0), Some(1.5), None));
}

#[test]
fn horas_positivas_nao_tem_limite_e_valores_invalidos_sao_erro() {
    let r = novo(vec![acao("2026-10-22", "2026-10-23", "", vec![plano(1, json!({"horas": 0.1})), plano(2, json!({"horas": 20})), plano(3, json!({"horas": 1.25}))])]);
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    assert_eq!(r.proposta.unwrap()["acoes"][0]["planos"][2]["horas"], 1.25);
    let r = novo(vec![acao("2026-10-22", "2026-10-23", "", vec![plano(1, json!({"horas": 0})), plano(2, json!({"horas": -1})), plano(3, json!({"horas": "muito"}))])]);
    assert_eq!(r.erros.iter().filter(|e| e.contains("`horas` deve ser um número positivo")).count(), 3, "{:?}", r.erros);
    let r = novo(vec![acao("2026-10-22", "2026-10-24", "", vec![plano(1, json!({"horas": 10})), plano(2, json!({"horas": 2})), plano(3, json!({"horas": 2}))])]);
    assert!(r.erros.is_empty() && tem(&r.avisos, "10 h é muito para um plano"), "{:?} {:?}", r.erros, r.avisos);
}

#[test]
fn reenviar_so_o_texto_mantem_as_horas_do_plano_existente() {
    let bp = json!({"id": "p", "nome": "P", "descricao": "d", "obs": "", "acoes": [{"id": "a1", "nome": "Base", "inicio": "2026-10-22", "fim": "2026-10-23", "status": "andamento", "obs": "", "planos": [
        {"id": "t1", "texto": "Criar rota", "descricao": "x", "feito": false, "horas": 3.0}, {"id": "t2", "texto": "Criar tela", "descricao": "y", "feito": false}], "checkpoints": []}]});
    let r = validar_proposta(&json!({"acoes": [{"id": "a1", "planos": [{"id": "t1", "texto": "Criar rota nova"}]}]}), Some(&bp), HOJE);
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    let planos = &r.proposta.unwrap()["acoes"][0]["planos"];
    assert_eq!(planos[0]["horas"].as_f64(), Some(3.0), "o plano citado mantém a estimativa");
    assert!(planos[1].get("horas").is_none(), "o plano não citado segue sem estimativa");
}

#[test]
fn acao_de_mais_de_3_dias_pede_motivo() {
    let longa = |obs: &str| novo(vec![acao("2026-10-22", "2026-10-30", obs, tres([2.0, 2.0, 2.0]))]);
    assert!(tem(&longa("").avisos, "dura 9 dias"), "{:?}", longa("").avisos);
    assert!(!tem(&longa("Depende da espera da aprovação do usuário.").avisos, "dura 9 dias"));
    assert!(!tem(&novo(vec![acao("2026-10-22", "2026-10-24", "", tres([2.0, 2.0, 2.0]))]).avisos, "dura"), "3 dias cabem sem motivo");
}

#[test]
fn horas_que_nao_combinam_com_a_duracao_avisam() {
    let r = novo(vec![acao("2026-10-22", "2026-10-28", "Espera por revisão do usuário.", tres([2.0, 2.0, 2.0]))]);
    assert!(tem(&r.avisos, "6 h de trabalho em 7 dias parece frouxo"), "{:?}", r.avisos);
    let r = novo(vec![acao("2026-10-22", "2026-10-22", "", tres([8.0, 6.0, 6.0]))]);
    assert!(tem(&r.avisos, "20 h em 1 dia(s) não cabe"), "{:?}", r.avisos);
    let r = novo(vec![acao("2026-10-22", "2026-10-23", "", tres([2.0, 3.0, 2.0]))]);
    assert!(r.avisos.is_empty(), "sem aviso quando a estimativa combina: {:?}", r.avisos);
}

#[test]
fn buraco_de_mais_de_2_dias_entre_acoes_novas_avisa() {
    let a = acao("2026-10-22", "2026-10-23", "", tres([2.0, 3.0, 2.0]));
    let sem_buraco = acao("2026-10-25", "2026-10-26", "", tres([2.0, 3.0, 2.0]));
    let com_buraco = acao("2026-10-30", "2026-10-31", "", tres([2.0, 3.0, 2.0]));
    assert!(novo(vec![a.clone(), sem_buraco]).avisos.is_empty());
    let r = novo(vec![a, com_buraco]);
    assert!(tem(&r.avisos, "começa 6 dias depois do fim"), "{:?}", r.avisos);
}

#[test]
fn contas_com_datas_e_instantes() {
    assert_eq!(dias_entre("2026-10-22", "2026-10-30"), 8);
    assert_eq!(dias_entre("2026-02-28", "2026-03-01"), 1);
    assert_eq!(dias_entre("2028-02-28", "2028-03-01"), 2, "2028 é bissexto");
    assert_eq!(ms_do_iso("1970-01-01T00:00:01.500Z"), Some(1500));
    assert_eq!(ms_do_iso("2026-10-22T10:00:00.000Z").unwrap() - ms_do_iso("2026-10-22T07:30:00.000Z").unwrap(), 150 * 60_000);
    assert_eq!(ms_do_iso("não é data"), None);
}

fn medido(id: &str, inicio: &str, fim: &str) -> Value {
    json!({"id": id, "texto": "Criar x", "descricao": "d", "feito": true, "horas": 2, "agente": {"estado": "concluido", "inicioEm": inicio, "feitoEm": fim}})
}

fn com_planos(planos: Vec<Value>) -> Value {
    json!({"id": "p", "nome": "P", "descricao": "d", "acoes": [{"id": "a1", "nome": "A", "inicio": "2026-10-22", "fim": "2026-10-23", "status": "andamento", "planos": planos, "checkpoints": []}]})
}

#[test]
fn a_estrutura_mostra_as_horas_e_a_media_real_so_com_3_planos_medidos() {
    use super::estrutura_blueprint;
    let a = medido("t1", "2026-10-22T08:00:00.000Z", "2026-10-22T10:00:00.000Z");
    let b = medido("t2", "2026-10-22T10:00:00.000Z", "2026-10-22T11:00:00.000Z");
    let c = medido("t3", "2026-10-22T11:00:00.000Z", "2026-10-22T14:00:00.000Z");
    let dois = estrutura_blueprint(&com_planos(vec![a.clone(), b.clone()]));
    assert!(dois.get("tempoReal").is_none(), "2 planos medidos não bastam");
    assert_eq!(dois["acoes"][0]["planos"][0]["horas"], 2);
    let tres = estrutura_blueprint(&com_planos(vec![a, b, c]));
    assert_eq!(tres["tempoReal"], json!({"planosMedidos": 3, "horasPorPlano": 2.0}), "(2 + 1 + 3) / 3");
    let curtos = (1..=3).map(|i| medido(&format!("t{i}"), "2026-10-22T10:00:00.000Z", "2026-10-22T10:01:00.000Z")).collect();
    assert_eq!(estrutura_blueprint(&com_planos(curtos))["tempoReal"]["horasPorPlano"], 0.017, "medições curtas não arredondam para zero");
}

#[test]
fn a_acao_completa_mostra_os_instantes_medidos_sem_expor_o_resto_do_agente() {
    let p = com_planos(vec![medido("t1", "2026-10-22T08:00:00.000Z", "2026-10-22T10:00:00.000Z")]);
    let acao = super::contexto_acao(&p, "a1").unwrap();
    assert_eq!(acao["planos"][0]["tempoReal"], json!({"inicioEm": "2026-10-22T08:00:00.000Z", "feitoEm": "2026-10-22T10:00:00.000Z"}));
    assert!(acao["planos"][0].get("agente").is_none());
}
