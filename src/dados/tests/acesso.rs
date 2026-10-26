//! As tabelas de acesso (005): membros, acessos, convites, sessões, chaves e atividade.
use crate::contexto::{Acesso, Papel};
use crate::dados::convites::{self, Convite};
use crate::dados::sessoes::{self, Sessao};
use crate::dados::{atividade, membros, Dados};

const AGORA: &str = "2027-01-15T10:00:00.000Z";

fn convite(id: &str, papel: &str, ws: &str, bp: &str, usos_max: Option<i64>) -> Convite {
    Convite { id: id.into(), papel: papel.into(), workspace_id: ws.into(), blueprint_id: bp.into(), abre: "folha".into(), criado_por: None, criado_em: String::new(), expira_em: "2027-01-20T00:00:00.000Z".into(), usos_max, usos: 0, revogado_em: None }
}

#[test]
fn membro_com_acessos_e_o_maior_papel_por_escopo() {
    let mut d = Dados::em_memoria().unwrap();
    d.transacao(|t| {
        membros::criar(t, "m1", "Ana")?;
        membros::somar_acesso(t, "m1", "w1", "", Papel::Colaborador, Some("c1"))?;
        membros::somar_acesso(t, "m1", "w1", "", Papel::Supervisor, None)?; // não rebaixa
        membros::somar_acesso(t, "m1", "w2", "b9", Papel::Supervisor, None)
    })
    .unwrap();
    let c = d.conexao();
    assert_eq!(membros::nome_ativo(c, "m1").unwrap().as_deref(), Some("Ana"));
    assert_eq!(membros::acessos(c, "m1").unwrap(), vec![Acesso::novo("w1", "", Papel::Colaborador), Acesso::novo("w2", "b9", Papel::Supervisor)]);
    let pessoas = membros::do_workspace(c, "w1").unwrap();
    assert_eq!((pessoas.len(), pessoas[0]["papel"].as_str()), (1, Some("colaborador")));
}

#[test]
fn o_ultimo_dono_e_contado_e_remover_de_vez_derruba_sessao_e_chave() {
    let mut d = Dados::em_memoria().unwrap();
    d.transacao(|t| {
        membros::criar(t, "dono", "Dono")?;
        membros::somar_acesso(t, "dono", "*", "", Papel::Dono, None)?;
        membros::criar(t, "m1", "Ana")?;
        membros::definir_papel(t, "m1", "w1", Papel::Dono)?;
        sessoes::criar(t, "h1", &Sessao { membro_id: Some("m1".into()), convite_id: None, expira_em: "2027-02-01T00:00:00.000Z".into() }, AGORA)?;
        sessoes::gravar_chave(t, "k1", "m1", "bpk_abcd")
    })
    .unwrap();
    let c = d.conexao();
    assert!(membros::existe_dono_geral(c).unwrap());
    assert_eq!(membros::donos_do_workspace(c, "w1", "").unwrap(), 2, "o dono de todos conta em todo workspace");
    assert_eq!(membros::donos_do_workspace(c, "w1", "m1").unwrap(), 1);
    assert_eq!(membros::do_workspace(c, "w1").unwrap().len(), 2);
    let removido = d.transacao(|t| membros::remover_do_workspace(t, "m1", "w1", AGORA)).unwrap();
    let c = d.conexao();
    assert!(removido);
    assert_eq!(membros::nome_ativo(c, "m1").unwrap(), None);
    assert_eq!(sessoes::buscar(c, "h1", AGORA).unwrap(), None);
    assert_eq!(sessoes::membro_da_chave(c, "k1").unwrap(), None);
}

#[test]
fn convite_valido_vencido_esgotado_e_revogado() {
    let mut d = Dados::em_memoria().unwrap();
    d.transacao(|t| {
        convites::criar(t, &convite("c1", "colaborador", "w1", "", Some(1)), "hash1")?;
        convites::criar(t, &convite("c2", "visitante", "w1", "b1", None), "hash2")?;
        sessoes::criar(t, "sv", &Sessao { membro_id: None, convite_id: Some("c2".into()), expira_em: "2027-01-20T00:00:00.000Z".into() }, AGORA)
    })
    .unwrap();
    let c1 = convites::por_hash(d.conexao(), "hash1").unwrap().unwrap();
    assert!(c1.valido(AGORA));
    assert!(!c1.valido("2027-01-20T00:00:00.000Z"), "vencido");
    assert_eq!(convites::por_hash(d.conexao(), "outro").unwrap(), None, "adulterado");
    d.transacao(|t| convites::usar(t, "c1")).unwrap();
    assert!(!convites::por_id(d.conexao(), "c1").unwrap().unwrap().valido(AGORA), "esgotado");
    assert!(convites::por_id(d.conexao(), "c2").unwrap().unwrap().valido(AGORA), "sem limite de usos");
    assert!(d.transacao(|t| convites::revogar(t, "c2", AGORA)).unwrap());
    assert!(!convites::por_id(d.conexao(), "c2").unwrap().unwrap().valido(AGORA), "revogado");
    assert_eq!(sessoes::buscar(d.conexao(), "sv", AGORA).unwrap(), None, "revogar derruba a sessão do visitante");
    assert_eq!(convites::do_workspace(d.conexao(), "w1").unwrap().len(), 2);
}

#[test]
fn o_convite_pendente_do_dono() {
    let mut d = Dados::em_memoria().unwrap();
    assert_eq!(convites::pendente_do_dono(d.conexao(), AGORA).unwrap(), None);
    d.transacao(|t| convites::criar(t, &convite("cd", "dono", "*", "", Some(1)), "hd")).unwrap();
    assert_eq!(convites::pendente_do_dono(d.conexao(), AGORA).unwrap().map(|c| c.id), Some("cd".into()));
    d.transacao(|t| convites::usar(t, "cd")).unwrap();
    assert_eq!(convites::pendente_do_dono(d.conexao(), AGORA).unwrap(), None, "usado não está pendente");
}

#[test]
fn sessao_vencida_some_e_a_chave_e_trocada() {
    let mut d = Dados::em_memoria().unwrap();
    d.transacao(|t| {
        membros::criar(t, "m1", "Ana")?;
        sessoes::criar(t, "h", &Sessao { membro_id: Some("m1".into()), convite_id: None, expira_em: "2027-01-15T11:00:00.000Z".into() }, AGORA)?;
        sessoes::gravar_chave(t, "k1", "m1", "bpk_aaaa")?;
        sessoes::gravar_chave(t, "k2", "m1", "bpk_bbbb")
    })
    .unwrap();
    let c = d.conexao();
    assert!(sessoes::buscar(c, "h", AGORA).unwrap().is_some());
    assert_eq!(sessoes::membro_da_chave(c, "k1").unwrap(), None, "a chave nova troca a anterior");
    assert_eq!(sessoes::membro_da_chave(c, "k2").unwrap().as_deref(), Some("m1"));
    assert_eq!(sessoes::chave_do_membro(c, "m1").unwrap().map(|k| k.0), Some("bpk_bbbb".into()));
    assert_eq!(sessoes::buscar(c, "h", "2027-01-15T11:00:00.000Z").unwrap(), None, "vencida");
    assert!(sessoes::revogar_chave(c, "m1").unwrap());
}

#[test]
fn atividade_com_retencao() {
    let mut d = Dados::em_memoria().unwrap();
    let r = |tipo: &'static str| atividade::Registro { membro_id: Some("m1"), nome: "Ana", tipo, workspace_id: "w1", blueprint_id: "", alvo: "c1" };
    let dia = 86_400_000i64;
    let hoje = 1_800_000_000_000i64;
    d.transacao(|t| {
        atividade::gravar(t, hoje - 100 * dia, &r("entrou"))?;
        atividade::gravar(t, hoje, &r("convite-criado"))
    })
    .unwrap();
    let itens = atividade::listar(d.conexao(), "w1", 10).unwrap();
    assert_eq!(itens.len(), 1, "mais de 90 dias sai");
    assert_eq!((itens[0]["tipo"].as_str(), itens[0]["membro"]["nome"].as_str(), itens[0]["alvo"].as_str()), (Some("convite-criado"), Some("Ana"), Some("c1")));
    let c = d.conexao();
    c.execute("INSERT INTO atividade (em, tipo, workspace_id) SELECT ?1, 'entrou', 'w1' FROM (WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < ?2) SELECT i FROM n)", (crate::tempo::iso_utc(hoje), atividade::MAX_LINHAS)).unwrap();
    d.transacao(|t| atividade::gravar(t, hoje, &r("saiu"))).unwrap();
    let total: i64 = d.conexao().query_row("SELECT count(*) FROM atividade", [], |l| l.get(0)).unwrap();
    assert_eq!(total, atividade::MAX_LINHAS);
}
