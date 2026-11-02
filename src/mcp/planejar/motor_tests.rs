//! Testes de mcp::planejar::motor com lançador falso e git de verdade, em pasta temporária.
//! Passa de 300 linhas (teto 500): são os 16 cenários do contrato em cima de uma única cena, e o arquivo de teste da fatia é um só.
use super::motor::{ao_ligar, aviso_da_copia, passo, Ambiente, Relogio};
use crate::contexto::Contexto;
use crate::dados::planejar::{ExecucaoLinha, Lancamento, Mudanca, NovaExecucao};
use crate::dados::Dados;
use crate::mcp::Mcp;
use crate::plano::arvore::do_blueprint;
use serde_json::{json, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const AGORA: i64 = 10_000_000;
static SEQUENCIA: AtomicUsize = AtomicUsize::new(0);

type Lancados = Arc<Mutex<Vec<(PathBuf, Vec<OsString>)>>>;

struct Cena {
    base: PathBuf,
    d: Mutex<Dados>,
    m: Mcp,
    amb: Ambiente,
    lancados: Lancados,
}

impl Drop for Cena {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

fn g(pasta: &Path, args: &[&str]) -> String {
    let saida = Command::new("git").args(args).current_dir(pasta).env("GIT_TERMINAL_PROMPT", "0").output().unwrap();
    assert!(saida.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&saida.stderr));
    String::from_utf8_lossy(&saida.stdout).trim().to_string()
}

fn commit(pasta: &Path, nome: &str, texto: &str) {
    std::fs::write(pasta.join(nome), texto).unwrap();
    g(pasta, &["add", "-A"]);
    g(pasta, &["commit", "-q", "-m", nome]);
}

fn plano(id: &str, area: &str, depende: &[&str]) -> Value {
    json!({"id": format!("r{id}"), "ref": id, "texto": format!("Plano {id}"), "area": area, "dependeDe": depende})
}

fn blueprint(id: &str, planos: Vec<Value>) -> Value {
    json!({"id": id, "nome": id, "acoes": [{"id": format!("{id}a"), "ref": "1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-30", "planos": planos}]})
}

impl Cena {
    fn nova(blueprints: Vec<Value>) -> Cena {
        let n = SEQUENCIA.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!("blueprint-motor-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let mut d = Dados::em_memoria().unwrap();
        d.gravar_tudo(&Contexto::local(), &json!({"empresa": "Eu", "blueprints": blueprints})).unwrap();
        let lancados: Lancados = Default::default();
        let anotados = Arc::clone(&lancados);
        let amb = Ambiente {
            lancar: Box::new(move |pasta, _exe, args| {
                anotados.lock().unwrap().push((pasta.to_path_buf(), args.to_vec()));
                Ok("falso")
            }),
            git: "git".into(),
            pasta_copias: base.join("copias"),
        };
        Cena { base, d: Mutex::new(d), m: Mcp::novo("127.0.0.1", false, "blueprint", "http://127.0.0.1:3000", std::env::temp_dir()), amb, lancados }
    }

    /// Um repositório com um commit, ligado ao blueprint.
    fn repo(&self, nome: &str, bp: &str) -> PathBuf {
        let repo = self.base.join(nome);
        std::fs::create_dir_all(&repo).unwrap();
        g(&repo, &["init", "-q", "-b", "main"]);
        for (chave, valor) in [("user.name", "Teste"), ("user.email", "teste@exemplo.invalid"), ("commit.gpgsign", "false"), ("core.autocrlf", "false")] {
            g(&repo, &["config", chave, valor]);
        }
        commit(&repo, "base.txt", "base\n");
        self.d.lock().unwrap().vincular_pasta(&Contexto::local(), &repo.to_string_lossy(), "sm", bp).unwrap();
        repo
    }

    fn marcar(&self, bp: &str, r: &str, pasta: &Path) -> String {
        let (acao, plano, principal) = (format!("{bp}a"), format!("r{r}"), pasta.to_string_lossy().into_owned());
        // como `gesto/marcar`: a área da árvore é copiada ao marcar
        let tudo = self.arvore();
        let area = tudo["blueprints"].as_array().unwrap().iter().flat_map(|b| b["acoes"].as_array().unwrap()).flat_map(|a| a["planos"].as_array().unwrap()).find(|q| q["id"] == plano.as_str()).and_then(|q| q["area"].as_str()).unwrap_or("").to_string();
        self.d.lock().unwrap().execucao_criar(&NovaExecucao { workspace_id: "sm", blueprint_id: bp, acao_id: &acao, plano_id: &plano, plano_ref: r, area: &area, pasta_principal: &principal, marcada_em: AGORA, ..NovaExecucao::default() }).unwrap()
    }

    /// `rodando` sem passar pelo git, para os cenários de sinal e de área.
    fn rodando(&self, bp: &str, r: &str, area: &str, iniciada: i64) -> String {
        let id = self.marcar(bp, r, Path::new("/nao/importa"));
        let mut d = self.d.lock().unwrap();
        assert!(d.execucao_reivindicar(&id).unwrap());
        assert!(d.execucao_lancar(&id, &Lancamento { area, pasta_trabalho: "/copia", ramo: "agente/x", agora_ms: iniciada, ..Lancamento::default() }).unwrap());
        id
    }

    fn linha(&self, id: &str) -> ExecucaoLinha {
        self.d.lock().unwrap().execucao_de(id).unwrap().unwrap()
    }

    fn mudar(&self, id: &str, m: Mudanca) {
        assert!(self.d.lock().unwrap().execucao_mudar(id, &m).unwrap());
    }

    /// Um tique, sem carência: `partida_ms` lá atrás e o tique anterior há 2 s.
    fn tique(&self) {
        let mut r = Relogio { partida_ms: AGORA - 10_000_000, ultimo_ms: AGORA - 2_000 };
        passo(&self.amb, &self.d, &self.m, AGORA, &mut r);
    }

    fn quantos(&self) -> usize {
        self.lancados.lock().unwrap().len()
    }

    /// O plano fica pronto na árvore, com o ramo registrado na execução.
    fn pronta(&self, id: &str, r: &str) {
        let x = self.linha(id);
        let mut d = self.d.lock().unwrap();
        d.concluir_por_execucao(&format!("r{r}"), "evidência", "2026-11-01T00:00:00.000Z").unwrap();
        d.execucao_mudar(&x.id, &Mudanca { situacao: Some("pronta"), ..Mudanca::default() }).unwrap();
    }

    fn arvore(&self) -> Value {
        self.d.lock().unwrap().ler_tudo(&Contexto::local()).unwrap().clone()
    }
}

#[test]
fn motor_nao_cria_execucao() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[]), plano("1.2", "", &[])])]);
    c.repo("repo", "bp1");
    for _ in 0..3 {
        c.tique();
    }
    assert!(c.d.lock().unwrap().execucoes_do_ws("sm").unwrap().is_empty());
    assert_eq!(c.quantos(), 0);
}

#[test]
fn marcada_lanca_um_por_tique() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "a", &[]), plano("1.2", "b", &[])])]);
    let repo = c.repo("repo", "bp1");
    let (a, b) = (c.marcar("bp1", "1.1", &repo), c.marcar("bp1", "1.2", &repo));
    c.tique();
    assert_eq!(c.quantos(), 1, "um lançamento por tique");
    let x = c.linha(&a);
    assert_eq!((x.situacao.as_str(), x.ramo.as_str(), x.tentativa, x.base.as_str(), x.area.as_str()), ("rodando", "agente/1.1", 1, "HEAD", "a"));
    assert_eq!(x.segredo.len(), 32);
    assert!(x.pasta_trabalho.starts_with(&*c.amb.pasta_copias.to_string_lossy()), "fora do repositório: {}", x.pasta_trabalho);
    assert!(Path::new(&x.pasta_trabalho).join("base.txt").exists());
    assert_eq!(c.linha(&b).situacao, "marcada");
    {
        let l = c.lancados.lock().unwrap();
        assert_eq!(l[0].0, PathBuf::from(&x.pasta_trabalho));
        assert_eq!(l[0].1, vec![OsString::from("agente-plano"), OsString::from(&a)], "só o id na linha de comando");
    }
    c.tique();
    assert_eq!(c.quantos(), 2);
    assert_eq!(c.linha(&b).ramo, "agente/1.2");
    c.tique();
    assert_eq!(c.quantos(), 2, "quem já roda não é lançado de novo");
}

#[test]
fn limite_segura_fila() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "a", &[]), plano("1.2", "b", &[])])]);
    let repo = c.repo("repo", "bp1");
    c.d.lock().unwrap().config_gravar("limite:sm", "1").unwrap();
    let (a, b) = (c.marcar("bp1", "1.1", &repo), c.marcar("bp1", "1.2", &repo));
    for _ in 0..3 {
        c.tique();
    }
    assert_eq!((c.quantos(), c.linha(&a).situacao.as_str(), c.linha(&b).situacao.as_str()), (1, "rodando", "marcada"));
    c.pronta(&a, "1.1");
    c.tique();
    assert_eq!((c.quantos(), c.linha(&b).situacao.as_str()), (2, "rodando"), "a vaga abriu");
}

#[test]
fn mesma_area_espera_entre_blueprints() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "app/barra", &[])]), blueprint("bp2", vec![plano("2.1", "app", &[])])]);
    let (r1, r2) = (c.repo("repo1", "bp1"), c.repo("repo2", "bp2"));
    let (a, b) = (c.marcar("bp1", "1.1", &r1), c.marcar("bp2", "2.1", &r2));
    for _ in 0..3 {
        c.tique();
    }
    assert_eq!((c.quantos(), c.linha(&a).situacao.as_str(), c.linha(&b).situacao.as_str()), (1, "rodando", "marcada"));
}

#[test]
fn pronta_libera_e_base_e_o_ramo() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "app", &[]), plano("1.2", "app", &[]), plano("1.3", "outra", &["1.1"]), plano("1.4", "livre", &[])])]);
    let repo = c.repo("repo", "bp1");
    let (a, b, d, e) = (c.marcar("bp1", "1.1", &repo), c.marcar("bp1", "1.2", &repo), c.marcar("bp1", "1.3", &repo), c.marcar("bp1", "1.4", &repo));
    c.d.lock().unwrap().config_gravar("limite:sm", "1").unwrap();
    c.tique();
    assert_eq!(c.linha(&a).situacao, "rodando");
    let copia = PathBuf::from(c.linha(&a).pasta_trabalho);
    commit(&copia, "do-1.txt", "feito no ramo\n");
    c.tique();
    assert_eq!(c.linha(&b).situacao, "marcada", "segue a área, o plano e a dependência");
    c.pronta(&a, "1.1");
    c.d.lock().unwrap().config_gravar("limite:sm", "4").unwrap();
    c.tique();
    let x = c.linha(&b);
    assert_eq!((x.situacao.as_str(), x.base.as_str()), ("rodando", "agente/1.1"), "mesma área: parte do ramo pronto, que não está no principal");
    assert!(Path::new(&x.pasta_trabalho).join("do-1.txt").exists());
    assert!(!repo.join("do-1.txt").exists());
    c.tique();
    let y = c.linha(&d);
    assert_eq!((y.situacao.as_str(), y.base.as_str()), ("rodando", "agente/1.1"), "dependência também é base");
    c.tique();
    let z = c.linha(&e);
    assert_eq!((z.situacao.as_str(), z.base.as_str()), ("rodando", "HEAD"), "sem relação: parte do HEAD");
}

#[test]
fn alterado_nao_inicia() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "a", &[]), plano("1.2", "b", &[])])]);
    let repo = c.repo("repo", "bp1");
    let bp = c.arvore()["blueprints"][0].clone();
    let mut doc = do_blueprint(&bp);
    doc.projetos[0].acoes[0].planos[0].texto = "Plano 1.1 reescrito".into();
    let numero = {
        let mut d = c.d.lock().unwrap();
        let (_, escrita) = d.plano_abrir("sm", "bp1", "escrita", "", &doc.para_json(), &[], AGORA).unwrap();
        d.plano_fechar(&escrita, AGORA).unwrap()
    };
    let (a, b) = (c.marcar("bp1", "1.1", &repo), c.marcar("bp1", "1.2", &repo));
    c.tique();
    c.tique();
    assert_eq!((c.linha(&a).situacao.as_str(), c.linha(&b).situacao.as_str()), ("marcada", "rodando"), "só o alterado espera a revisão");
    c.d.lock().unwrap().revisao_gravar("bp1", "diff:1", numero, "reprovada", "", "", AGORA).unwrap();
    c.tique();
    assert_eq!(c.linha(&a).situacao, "rodando", "reprovada resolve: o plano segue como está na árvore");
}

#[test]
fn sem_sinal_vira_sumiu() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[]), plano("1.2", "", &[])])]);
    let (a, b) = (c.rodando("bp1", "1.1", "", AGORA - 500_000), c.rodando("bp1", "1.2", "", AGORA - 500_000));
    c.mudar(&a, Mudanca { ultimo_sinal: Some(AGORA - 120_001), ..Mudanca::default() });
    c.mudar(&b, Mudanca { ultimo_sinal: Some(AGORA - 60_000), ..Mudanca::default() });
    c.tique();
    assert_eq!((c.linha(&a).situacao.as_str(), c.linha(&b).situacao.as_str()), ("sumiu", "rodando"));
    assert!(c.d.lock().unwrap().orq_ler("sm").unwrap().motivos.iter().any(|m| m == "sumiu:1.1"), "acorda o orquestrador");
    assert_eq!(c.quantos(), 0);
}

#[test]
fn nunca_sinalizou_vira_sumiu() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])])]);
    let a = c.rodando("bp1", "1.1", "", AGORA - 120_001);
    c.tique();
    assert_eq!(c.linha(&a).situacao, "sumiu", "o invólucro nunca falou: vale a hora do lançamento");
}

#[test]
fn carencia_ao_reiniciar() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])])]);
    let a = c.rodando("bp1", "1.1", "", AGORA - 600_000);
    let mut r = Relogio { partida_ms: AGORA, ultimo_ms: AGORA };
    let mut t = AGORA;
    while t < AGORA + 118_000 {
        t += 2_000;
        passo(&c.amb, &c.d, &c.m, t, &mut r);
        assert_eq!(c.linha(&a).situacao, "rodando", "o app acabou de abrir: o agente ainda não teve como sinalizar ({t})");
    }
    passo(&c.amb, &c.d, &c.m, t + 2_000, &mut r);
    assert_eq!(c.linha(&a).situacao, "sumiu", "passada a carência, vale a regra");
}

#[test]
fn tique_atrasado_renova_carencia() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])])]);
    let a = c.rodando("bp1", "1.1", "", AGORA - 600_000);
    let mut r = Relogio { partida_ms: AGORA - 900_000, ultimo_ms: AGORA - 30_001 };
    passo(&c.amb, &c.d, &c.m, AGORA, &mut r);
    assert_eq!(r.partida_ms, AGORA, "suspensão: a carência recomeça");
    assert_eq!(r.ultimo_ms, AGORA);
    assert_eq!(c.linha(&a).situacao, "rodando");
    let mut r = Relogio { partida_ms: AGORA - 900_000, ultimo_ms: AGORA - 30_000 };
    passo(&c.amb, &c.d, &c.m, AGORA, &mut r);
    assert_eq!(r.partida_ms, AGORA - 900_000, "até 30 s é só um tique lento");
    assert_eq!(c.linha(&a).situacao, "sumiu");
}

#[test]
fn sumiu_segura_area() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "app", &[]), plano("1.2", "app", &[]), plano("1.3", "outra", &[])])]);
    let repo = c.repo("repo", "bp1");
    c.d.lock().unwrap().config_gravar("limite:sm", "1").unwrap();
    let a = c.rodando("bp1", "1.1", "app", AGORA - 500_000);
    let (b, e) = (c.marcar("bp1", "1.2", &repo), c.marcar("bp1", "1.3", &repo));
    c.tique();
    assert_eq!(c.linha(&a).situacao, "sumiu");
    assert_eq!(c.linha(&b).situacao, "marcada", "o trabalho ficou na cópia: a área segue presa");
    assert_eq!(c.linha(&e).situacao, "rodando", "e o agente que saiu não ocupa vaga");
}

/// Um `git` que avisa ao chegar em `worktree add` e espera a licença do teste.
#[cfg(unix)]
fn git_que_espera(base: &Path) -> (String, PathBuf, PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let (chegou, segue, script) = (base.join("chegou"), base.join("segue"), base.join("git-lento.sh"));
    let corpo = format!("#!/bin/sh\nif [ \"$1\" = worktree ] && [ \"$2\" = add ]; then\n  : > '{}'\n  while [ ! -e '{}' ]; do sleep 0.02; done\nfi\nexec git \"$@\"\n", chegou.display(), segue.display());
    std::fs::write(&script, corpo).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100)); // outro teste pode estar criando processo: o script só roda depois de nenhum filho herdar o arquivo aberto
    (script.to_string_lossy().into_owned(), chegou, segue)
}

#[cfg(unix)]
#[test]
fn cancelar_durante_git_nao_lanca() {
    let mut c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])])]);
    let repo = c.repo("repo", "bp1");
    let (script, chegou, segue) = git_que_espera(&c.base);
    c.amb.git = script;
    let a = c.marcar("bp1", "1.1", &repo);
    std::thread::scope(|s| {
        let tique = s.spawn(|| c.tique());
        let limite = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while !chegou.exists() {
            assert!(std::time::Instant::now() < limite, "o git não chegou no worktree add");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(c.linha(&a).situacao, "preparando");
        assert!(c.d.lock().unwrap().execucao_cancelar(&a, AGORA).unwrap(), "o banco fica livre durante o git");
        std::fs::write(&segue, "").unwrap();
        tique.join().unwrap();
    });
    assert_eq!(c.quantos(), 0);
    let x = c.linha(&a);
    assert_eq!((x.situacao.as_str(), x.tentativa, x.segredo.as_str()), ("cancelada", 0, ""));
}

#[test]
fn plano_apagado_cancela() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "app", &[]), plano("1.2", "", &[])])]);
    let a = c.rodando("bp1", "1.1", "app", AGORA);
    let mut tudo = c.arvore();
    tudo["blueprints"][0]["acoes"][0]["planos"].as_array_mut().unwrap().remove(0);
    c.d.lock().unwrap().gravar_tudo(&Contexto::local(), &tudo).unwrap();
    c.tique();
    let x = c.linha(&a);
    assert_eq!((x.situacao.as_str(), x.motivo.as_str(), x.parar), ("cancelada", "plano-removido", true));
    assert!(x.cancelada_em.is_some());
    let recados = c.d.lock().unwrap().recados_para(&a).unwrap();
    assert!(recados[0].texto.contains("1.1"), "fica o aviso");
    assert_eq!(c.quantos(), 0);
}

#[test]
fn blueprint_apagado_limpa() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])]), blueprint("bp2", vec![plano("2.1", "", &[])])]);
    let a = c.rodando("bp2", "2.1", "", AGORA);
    let doc = do_blueprint(&c.arvore()["blueprints"][1]);
    let doc1 = do_blueprint(&c.arvore()["blueprints"][0]);
    {
        let mut d = c.d.lock().unwrap();
        let (_, e2) = d.plano_abrir("sm", "bp2", "escrita", "", &doc.para_json(), &[], AGORA).unwrap();
        d.plano_fechar(&e2, AGORA).unwrap();
        d.revisao_gravar("bp2", "d:x", 1, "aprovada", "", "h", AGORA).unwrap();
        let (_, e1) = d.plano_abrir("sm", "bp1", "escrita", "", &doc1.para_json(), &[], AGORA).unwrap();
        d.plano_fechar(&e1, AGORA).unwrap();
    }
    let mut tudo = c.arvore();
    tudo["blueprints"].as_array_mut().unwrap().remove(1);
    c.d.lock().unwrap().gravar_tudo(&Contexto::local(), &tudo).unwrap();
    c.tique();
    assert_eq!(c.linha(&a).situacao, "cancelada", "primeiro cancela");
    let d = c.d.lock().unwrap();
    assert!(d.plano_pendente("bp2").unwrap().is_none() && d.revisoes_do("bp2").unwrap().is_empty(), "e limpa o que era do blueprint");
    assert!(d.plano_pendente("bp1").unwrap().is_some(), "o outro blueprint fica");
    assert!(d.execucao_de(&a).unwrap().is_some(), "a execução fica como histórico");
}

#[test]
fn blueprint_apagado_sem_execucao_tambem_limpa() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])])]);
    c.d.lock().unwrap().revisao_gravar("sumido", "d:x", 1, "aprovada", "", "h", AGORA).unwrap();
    c.tique();
    assert!(c.d.lock().unwrap().revisoes_do("sumido").unwrap().is_empty());
}

#[test]
fn retomar_apos_base_em_conflito_tenta_de_novo() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[]), plano("1.2", "", &[]), plano("1.3", "", &["1.1", "1.2"])])]);
    let repo = c.repo("repo", "bp1");
    let (a, b, e) = (c.marcar("bp1", "1.1", &repo), c.marcar("bp1", "1.2", &repo), c.marcar("bp1", "1.3", &repo));
    c.tique();
    c.tique();
    commit(Path::new(&c.linha(&a).pasta_trabalho), "mesmo.txt", "do 1.1\n");
    commit(Path::new(&c.linha(&b).pasta_trabalho), "mesmo.txt", "do 1.2\n");
    c.pronta(&a, "1.1");
    c.pronta(&b, "1.2");
    c.tique();
    let x = c.linha(&e);
    assert_eq!((x.situacao.as_str(), x.motivo.as_str(), x.ramo.as_str()), ("falhou", "base-em-conflito", ""));
    assert_eq!(c.quantos(), 2, "não segue sem as bases");
    assert!(g(&repo, &["branch", "--list", "agente/1.3"]).is_empty(), "a cópia e o ramo novos saem");
    // o usuário integra uma das bases e retoma: tenta tudo de novo
    c.mudar(&a, Mudanca { integrada_em: Some(Some(AGORA)), ..Mudanca::default() });
    c.mudar(&e, Mudanca { so_de: &["falhou"], situacao: Some("marcada"), motivo: Some(""), ..Mudanca::default() });
    c.tique();
    let x = c.linha(&e);
    assert_eq!((x.situacao.as_str(), x.base.as_str(), x.tentativa), ("rodando", "agente/1.2", 1));
    assert_eq!(c.quantos(), 3);
}

#[test]
fn filho_nao_herda_token() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])])]);
    let repo = c.repo("repo", "bp1");
    let token = c.m.ligar_janela();
    let a = c.marcar("bp1", "1.1", &repo);
    c.tique();
    let l = c.lancados.lock().unwrap().clone();
    assert_eq!(l.len(), 1);
    assert!(l[0].1.iter().all(|arg| !arg.to_string_lossy().contains(&token)), "nem na linha de comando");
    assert!(std::env::vars_os().all(|(_, v)| !v.to_string_lossy().contains(&token)), "nem no ambiente que o filho herda");
    let (status, partida) = crate::mcp::rotas::reenviar(&c.m, &c.d, "execucao/partida", &json!({"id": a}));
    assert_eq!(status, 200, "{partida}");
    assert!(!partida.to_string().contains(&token), "nem no que o invólucro recebe");
}

#[test]
fn ao_ligar_devolve_preparando_a_fila() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])])]);
    let a = c.marcar("bp1", "1.1", Path::new("/nao/importa"));
    assert!(c.d.lock().unwrap().execucao_reivindicar(&a).unwrap());
    ao_ligar(&c.d, &c.m);
    assert_eq!(c.linha(&a).situacao, "marcada");
}

#[test]
fn pasta_sumida_falha_sem_lancar() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])])]);
    let a = c.marcar("bp1", "1.1", &c.base.join("nao-existe"));
    c.tique();
    let x = c.linha(&a);
    assert_eq!((x.situacao.as_str(), x.motivo.as_str()), ("falhou", "pasta-nao-existe"));
    assert!(x.terminada_em.is_some());
    assert_eq!(c.quantos(), 0);
}

#[test]
fn lancador_que_falha_vira_falhou() {
    let mut c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])])]);
    let repo = c.repo("repo", "bp1");
    c.amb.lancar = Box::new(|_, _, _| Err("sem-terminal".into()));
    let a = c.marcar("bp1", "1.1", &repo);
    c.tique();
    let x = c.linha(&a);
    assert_eq!((x.situacao.as_str(), x.motivo.as_str()), ("falhou", "sem-terminal"));
}

#[test]
fn aviso_da_copia_com_git_real() {
    let c = Cena::nova(vec![blueprint("bp1", vec![plano("1.1", "", &[])])]);
    let repo = c.repo("repo", "bp1");
    let a = c.marcar("bp1", "1.1", &repo);
    c.tique();
    c.pronta(&a, "1.1");
    aviso_da_copia(&c.d, &c.m, &c.linha(&a));
    assert_eq!(c.linha(&a).motivo, "sem-commit");
    let copia = PathBuf::from(c.linha(&a).pasta_trabalho);
    std::fs::write(copia.join("solto.txt"), "x\n").unwrap();
    aviso_da_copia(&c.d, &c.m, &c.linha(&a));
    assert_eq!(c.linha(&a).motivo, "copia-suja");
    commit(&copia, "solto.txt", "x\n");
    c.mudar(&a, Mudanca { motivo: Some(""), ..Mudanca::default() });
    aviso_da_copia(&c.d, &c.m, &c.linha(&a));
    assert_eq!(c.linha(&a).motivo, "", "com commit e limpa, nada a avisar");
    assert_eq!(c.linha(&a).situacao, "pronta", "o aviso nunca impede");
}
