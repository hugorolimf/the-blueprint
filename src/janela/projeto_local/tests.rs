//! Os pedidos da janela da equipe: o que o app recusa, o que mostra na pergunta nativa, e que nada abre sem o "sim" (pergunta, aviso e terminal falsos).
use super::*;
use serde_json::json;
use std::cell::{Cell, RefCell};

const URL: &str = "https://github.com/dono/repo";
const ORIGEM: &str = "https://abc-def.trycloudflare.com";

fn corpo(extra: Value) -> String {
    let mut v = json!({"tipo": "configurar", "workspaceId": "w2", "blueprintId": "p9", "nome": "Portal; novo", "repositorio": URL});
    v.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    v.to_string()
}

fn iniciar(extra: Value) -> String {
    let mut base = json!({"tipo": "iniciar", "acaoId": "a1", "numero": "2.5", "acao": "Tela"});
    base.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    corpo(base)
}

fn projetos(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-projeto-local-{}-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}

#[test]
fn interpreta_so_o_que_tem_ids_url_e_numero_no_formato() {
    let Ok(Pedido::Configurar { ws, bp, nome_bp, repo }) = interpretar(&corpo(json!({}))) else { panic!("devia aceitar") };
    assert_eq!((ws.as_str(), bp.as_str(), nome_bp.as_str(), repo.nome.as_str()), ("w2", "p9", "Portal, novo", "repo"), "o ; do nome vira , (o wt o leria como separador)");
    let Ok(Pedido::Iniciar { numero, acao, acao_id, .. }) = interpretar(&iniciar(json!({}))) else { panic!("devia aceitar") };
    assert_eq!((numero.as_str(), acao.as_str(), acao_id.as_str()), ("2.5", "Tela", "a1"));
    let ruins = [
        corpo(json!({"workspaceId": "w 2"})), corpo(json!({"blueprintId": ""})), corpo(json!({"workspaceId": "../x"})),
        corpo(json!({"repositorio": "https://evil.com/dono/repo"})), corpo(json!({"repositorio": "git@github.com:dono/repo.git"})),
        corpo(json!({"repositorio": ""})), corpo(json!({"tipo": "apagar"})), corpo(json!({"tipo": ""})),
        iniciar(json!({"acaoId": "a`1"})), iniciar(json!({"numero": "2.5; rm"})), iniciar(json!({"numero": ""})),
        "{".to_string(), "[]".to_string(), "x".repeat(MAX_PEDIDO + 1),
    ];
    for r in ruins {
        assert!(interpretar(&r).is_err(), "devia recusar {r}");
    }
}

#[test]
fn a_conexao_tem_de_ser_a_do_mesmo_servidor() {
    assert!(conferir_conexao(Some(ORIGEM), ORIGEM).is_ok());
    assert!(conferir_conexao(Some(&format!("{ORIGEM}/")), ORIGEM).is_ok());
    assert!(conferir_conexao(None, ORIGEM).is_err(), "sem blueprint conectar não há Claude do convidado");
    assert!(conferir_conexao(Some("https://outro.trycloudflare.com"), ORIGEM).unwrap_err().contains("outro.trycloudflare.com"));
    assert!(conferir_conexao(Some("https://abc-def.trycloudflare.com.evil.com"), ORIGEM).is_err());
}

#[test]
fn configurar_clona_na_pasta_padrao_e_pergunta_com_a_url_e_o_destino() {
    let p = projetos("cfg");
    let plano = planejar(&interpretar(&corpo(json!({}))).unwrap(), &p).unwrap();
    assert!(plano.criar_projetos);
    assert_eq!(plano.terminal_em, p);
    let destino = p.join("repo");
    assert!(plano.pergunta.contains(URL) && plano.pergunta.contains(&destino.display().to_string()) && plano.pergunta.contains("«Portal, novo»"), "{}", plano.pergunta);
    assert!(plano.prompt.contains(&format!("git clone -- {URL} \"{}\"", destino.display())), "{}", plano.prompt);
    assert!(plano.prompt.contains("workspace_id w2, blueprint_id p9") && plano.prompt.contains("vincular_pasta") && !plano.prompt.contains(';'), "{}", plano.prompt);
}

#[test]
fn iniciar_so_vale_na_pasta_ja_ligada_a_este_blueprint() {
    let p = projetos("ini");
    let pedido = interpretar(&iniciar(json!({}))).unwrap();
    let msg = planejar(&pedido, &p).unwrap_err();
    assert!(msg.contains("Set up project") || msg.contains("Configurar projeto"), "{msg}");
    let destino = p.join("repo");
    std::fs::create_dir_all(&destino).unwrap();
    assert!(planejar(&pedido, &p).is_err(), "pasta sem a linha Blueprint");
    std::fs::write(destino.join("CLAUDE.local.md"), "Blueprint: workspace_id `w2`, blueprint_id `outro`\n").unwrap();
    assert!(planejar(&pedido, &p).is_err(), "pasta ligada a outro blueprint");
    std::fs::write(p.join("CLAUDE.md"), "Blueprint: workspace_id `w2`, blueprint_id `p9`\n").unwrap();
    std::fs::remove_file(destino.join("CLAUDE.local.md")).unwrap();
    assert!(planejar(&pedido, &p).is_err(), "a linha de uma pasta acima não vale");
    std::fs::write(destino.join("CLAUDE.local.md"), "Blueprint: workspace_id `w2`, blueprint_id `p9`\n").unwrap();
    let plano = planejar(&pedido, &p).unwrap();
    assert_eq!((plano.terminal_em.as_path(), plano.criar_projetos), (destino.as_path(), false));
    assert!(plano.prompt.contains("2.5 «Tela»") && plano.prompt.contains("(workspace_id w2, blueprint_id p9, acao_id a1)"), "{}", plano.prompt);
    assert!(plano.pergunta.contains("2.5") && plano.pergunta.contains(&destino.display().to_string()), "{}", plano.pergunta);
}

struct Falsos {
    perguntas: RefCell<Vec<String>>,
    avisos: RefCell<Vec<String>>,
    abertos: RefCell<Vec<(PathBuf, String)>>,
    sim: Cell<bool>,
}

impl Falsos {
    fn novo(sim: bool) -> Falsos {
        Falsos { perguntas: RefCell::new(vec![]), avisos: RefCell::new(vec![]), abertos: RefCell::new(vec![]), sim: Cell::new(sim) }
    }
    fn rodar(&self, c: &str, origem: &str, conectado: Option<&str>, projetos: &Path) {
        atender(
            c, origem, conectado, projetos,
            &|t| { self.perguntas.borrow_mut().push(t.into()); self.sim.get() },
            &|t| self.avisos.borrow_mut().push(t.into()),
            &|pasta, prompt| { self.abertos.borrow_mut().push((pasta.into(), prompt.into())); Ok("wt") },
        );
    }
}

#[test]
fn sem_o_sim_nada_abre_e_nada_e_criado() {
    let p = projetos("nao");
    let f = Falsos::novo(false);
    f.rodar(&corpo(json!({})), ORIGEM, Some(ORIGEM), &p);
    assert_eq!((f.perguntas.borrow().len(), f.abertos.borrow().len(), f.avisos.borrow().len()), (1, 0, 0));
    assert!(!p.exists(), "recusou: a pasta de projetos nem é criada");
}

#[test]
fn com_o_sim_cria_a_pasta_de_projetos_e_abre_o_terminal_nela() {
    let p = projetos("sim");
    let f = Falsos::novo(true);
    f.rodar(&corpo(json!({})), ORIGEM, Some(ORIGEM), &p);
    assert!(p.is_dir());
    let abertos = f.abertos.borrow();
    assert_eq!(abertos.len(), 1);
    assert_eq!(abertos[0].0, p);
    assert!(abertos[0].1.contains("git clone --"), "{}", abertos[0].1);
    assert!(f.avisos.borrow().is_empty());
}

#[test]
fn pedido_ruim_ou_servidor_errado_so_avisa_e_nao_pergunta_nem_abre() {
    let p = projetos("ruim");
    for (c, conectado) in [
        (corpo(json!({"repositorio": "https://evil.com/a/b"})), Some(ORIGEM)),
        (corpo(json!({})), None),
        (corpo(json!({})), Some("https://outro.trycloudflare.com")),
        (iniciar(json!({})), Some(ORIGEM)), // sem pasta configurada
    ] {
        let f = Falsos::novo(true);
        f.rodar(&c, ORIGEM, conectado, &p);
        assert_eq!((f.perguntas.borrow().len(), f.abertos.borrow().len(), f.avisos.borrow().len()), (0, 0, 1), "{c}");
    }
    assert!(!p.exists());
}

#[test]
fn erro_do_terminal_e_avisado() {
    let p = projetos("terminal");
    let avisos = RefCell::new(vec![]);
    atender(&corpo(json!({})), ORIGEM, Some(ORIGEM), &p, &|_| true, &|t| avisos.borrow_mut().push(t.to_string()), &|_, _| Err("sem wt".into()));
    assert_eq!(avisos.borrow().len(), 1);
    assert!(avisos.borrow()[0].contains("sem wt"));
}

#[test]
fn o_wt_recebe_a_flag_equipe_e_o_prompt_como_um_argumento() {
    let args = crate::mcp::iniciar::argumentos_do_wt(Path::new("C:\\p"), Path::new("C:\\blueprint.exe"), "um prompt com espaços", FLAG);
    let args: Vec<String> = args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    assert_eq!(args, ["-d", "C:\\p", "C:\\blueprint.exe", "claude", "um prompt com espaços", "--equipe"]);
}
