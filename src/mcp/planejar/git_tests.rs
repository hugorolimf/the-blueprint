//! Testes de mcp::planejar::git com git de verdade, em pasta temporária. A identidade e a assinatura ficam no config do repositório de teste, para não depender da máquina.
//! Passa de 300 linhas (teto 500): são os 18 cenários do contrato em cima de uma única cena de git, e o arquivo de teste da fatia é um só.
use super::git::{adiantado_de_head, integrar, ocupada, preparar_copia, raiz, ramo_atual, remover_copia, repositorio, sujo, tem_git, Recusa};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const GIT: &str = "git";
static SEQUENCIA: AtomicUsize = AtomicUsize::new(0);

/// A pasta do teste: `repo/` é a principal; as cópias ficam em `copias/abcdef012345/`, fora dele.
struct Cena {
    base: PathBuf,
    repo: PathBuf,
}

impl Drop for Cena {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

fn g(pasta: &Path, args: &[&str]) -> String {
    let saida = Command::new(GIT).args(args).current_dir(pasta).env("GIT_TERMINAL_PROMPT", "0").output().unwrap();
    assert!(saida.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&saida.stderr));
    String::from_utf8_lossy(&saida.stdout).trim().to_string()
}

fn escrever(pasta: &Path, nome: &str, texto: &str) {
    let caminho = pasta.join(nome);
    std::fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    std::fs::write(caminho, texto).unwrap();
}

fn ler(pasta: &Path, nome: &str) -> String {
    std::fs::read_to_string(pasta.join(nome)).unwrap()
}

fn commit(pasta: &Path, nome: &str, texto: &str) {
    escrever(pasta, nome, texto);
    g(pasta, &["add", "-A"]);
    g(pasta, &["commit", "-q", "-m", nome]);
}

impl Cena {
    fn vazia() -> Cena {
        let n = SEQUENCIA.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!("blueprint-git-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let repo = base.join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        g(&repo, &["init", "-q", "-b", "main"]);
        for (chave, valor) in [("user.name", "Teste"), ("user.email", "teste@exemplo.invalid"), ("commit.gpgsign", "false"), ("core.autocrlf", "false")] {
            g(&repo, &["config", chave, valor]);
        }
        Cena { base, repo }
    }

    fn nova() -> Cena {
        let c = Cena::vazia();
        commit(&c.repo, "a.txt", "linha\n");
        c
    }

    fn destino(&self, plano: &str) -> PathBuf {
        self.base.join("copias").join("abcdef012345").join(plano)
    }

    /// Um ramo com um commit, e a principal de volta em `main`.
    fn ramo(&self, nome: &str, arquivo: &str, texto: &str) {
        g(&self.repo, &["checkout", "-q", "-b", nome]);
        commit(&self.repo, arquivo, texto);
        g(&self.repo, &["checkout", "-q", "main"]);
    }

    /// Cópia com um commit no arquivo, pronta para integrar.
    fn copia_com_commit(&self, ramo: &str, arquivo: &str, texto: &str) -> PathBuf {
        let destino = self.destino("p1");
        preparar_copia(GIT, &self.repo, ramo, false, &destino, &[]).unwrap();
        commit(&destino, arquivo, texto);
        destino
    }

    fn merge_head(&self) -> bool {
        self.repo.join(".git/MERGE_HEAD").exists()
    }
}

/// O git de verdade, anotando cada chamada. Sem Unix não há script: devolve o git comum e nada é anotado.
fn espiao(c: &Cena) -> (String, PathBuf) {
    let registro = c.base.join("chamadas.txt");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script = c.base.join("git-espiao.sh");
        std::fs::write(&script, format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\nexec git \"$@\"\n", registro.display())).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        // recém-escrito, o script pode dar ETXTBSY enquanto outra thread do teste está em fork
        for _ in 0..100 {
            if Command::new(&script).arg("--version").output().is_ok_and(|s| s.status.success()) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let _ = std::fs::remove_file(&registro);
        (script.to_string_lossy().into_owned(), registro)
    }
    #[cfg(not(unix))]
    {
        (GIT.to_string(), registro)
    }
}

fn chamadas(registro: &Path) -> String {
    std::fs::read_to_string(registro).unwrap_or_default()
}

#[test]
fn cria_copia_e_ramo() {
    let c = Cena::nova();
    let destino = c.destino("p1");
    let copia = preparar_copia(GIT, &c.repo, "agente/1.1", false, &destino, &[]).unwrap();
    assert_eq!((copia.ramo.as_str(), copia.base.as_str(), copia.reusada), ("agente/1.1", "HEAD", false));
    assert_eq!((copia.raiz.as_path(), copia.pasta_trabalho.as_path()), (destino.as_path(), destino.as_path()));
    assert_eq!(ler(&destino, "a.txt"), "linha\n");
    assert_eq!(ramo_atual(GIT, &destino).as_deref(), Some("agente/1.1"));
    assert_eq!(ramo_atual(GIT, &c.repo).as_deref(), Some("main"), "a principal não troca de ramo");
    assert_eq!(repositorio(GIT, &destino), repositorio(GIT, &c.repo));
}

#[test]
fn reusa_copia_valida() {
    let c = Cena::nova();
    let destino = c.destino("p1");
    preparar_copia(GIT, &c.repo, "agente/1.1", true, &destino, &[]).unwrap();
    escrever(&destino, "trabalho.txt", "sem commit");
    let (git, registro) = espiao(&c);
    let copia = preparar_copia(&git, &c.repo, "agente/1.1", true, &destino, &[]).unwrap();
    assert!(copia.reusada);
    assert_eq!(copia.ramo, "agente/1.1");
    assert_eq!(ler(&destino, "trabalho.txt"), "sem commit", "o trabalho na cópia fica");
    assert!(!chamadas(&registro).contains("worktree add"), "sem add ao reusar");
}

#[test]
fn copia_apagada_a_mao_prune() {
    let c = Cena::nova();
    let destino = c.destino("p1");
    preparar_copia(GIT, &c.repo, "agente/1.1", true, &destino, &[]).unwrap();
    commit(&destino, "feito.txt", "ja commitado");
    std::fs::remove_dir_all(&destino).unwrap();
    let copia = preparar_copia(GIT, &c.repo, "agente/1.1", true, &destino, &[]).unwrap();
    assert_eq!((copia.ramo.as_str(), copia.reusada), ("agente/1.1", false));
    assert_eq!(ler(&destino, "feito.txt"), "ja commitado", "o ramo volta com o commit");
}

#[test]
fn ramo_em_outra_copia_sufixo() {
    let c = Cena::nova();
    preparar_copia(GIT, &c.repo, "agente/1.1", true, &c.destino("p1"), &[]).unwrap();
    let segunda = preparar_copia(GIT, &c.repo, "agente/1.1", true, &c.destino("p2"), &[]).unwrap();
    assert_eq!(segunda.ramo, "agente/1.1-abcd", "4 hex do blueprint");
    assert_eq!(ramo_atual(GIT, &c.destino("p2")).as_deref(), Some("agente/1.1-abcd"));
    assert_eq!(ramo_atual(GIT, &c.destino("p1")).as_deref(), Some("agente/1.1"));
    // voltar à mesma cópia pedindo o ramo original acha a sufixada, sem criar outra
    let de_novo = preparar_copia(GIT, &c.repo, "agente/1.1", true, &c.destino("p2"), &[]).unwrap();
    assert_eq!((de_novo.ramo.as_str(), de_novo.reusada), ("agente/1.1-abcd", true));
}

#[test]
fn ramo_do_usuario_nao_adota() {
    let c = Cena::nova();
    c.ramo("agente/1.1", "do-usuario.txt", "meu");
    let ponta = g(&c.repo, &["rev-parse", "agente/1.1"]);
    let destino = c.destino("p1");
    let copia = preparar_copia(GIT, &c.repo, "agente/1.1", false, &destino, &[]).unwrap();
    assert_eq!(copia.ramo, "agente/1.1-abcd");
    assert!(!destino.join("do-usuario.txt").exists());
    assert_eq!(g(&c.repo, &["rev-parse", "agente/1.1"]), ponta, "o ramo do usuário não anda");
}

#[test]
fn subpasta_relativa() {
    let c = Cena::nova();
    commit(&c.repo, "app/web/b.txt", "b");
    let destino = c.destino("p1");
    let copia = preparar_copia(GIT, &c.repo.join("app/web"), "agente/1.1", false, &destino, &[]).unwrap();
    assert_eq!(copia.raiz, destino);
    assert_eq!(copia.pasta_trabalho, destino.join("app/web"));
    assert_eq!(ler(&copia.pasta_trabalho, "b.txt"), "b");
    assert_eq!(raiz(GIT, &copia.pasta_trabalho).map(|p| std::fs::canonicalize(p).unwrap()), Some(std::fs::canonicalize(&destino).unwrap()));

    // subpasta fora do git não está no ramo: nasce na cópia
    std::fs::create_dir_all(c.repo.join("novo/sem-git")).unwrap();
    let outra = preparar_copia(GIT, &c.repo.join("novo/sem-git"), "agente/1.2", false, &c.destino("p2"), &[]).unwrap();
    assert!(outra.pasta_trabalho.is_dir());
    assert!(outra.pasta_trabalho.ends_with("novo/sem-git"));
}

#[test]
fn raiz_de_repositorio_normal() {
    let c = Cena::nova();
    let canonico = std::fs::canonicalize(&c.repo).unwrap();
    commit(&c.repo, "sub/x.txt", "x");
    assert_eq!(raiz(GIT, &c.repo).map(|p| std::fs::canonicalize(p).unwrap()).as_ref(), Some(&canonico));
    assert_eq!(raiz(GIT, &c.repo.join("sub")).map(|p| std::fs::canonicalize(p).unwrap()).as_ref(), Some(&canonico));
    assert!(tem_git(&c.repo.join("sub")), "sobe até achar o .git");
    assert_eq!(repositorio(GIT, &c.repo.join("sub")), repositorio(GIT, &c.repo));

    let sem = c.base.join("sem-git");
    std::fs::create_dir_all(&sem).unwrap();
    if !tem_git(&sem) {
        assert_eq!(raiz(GIT, &sem), None);
        // sem git, o plano roda na própria pasta e sem ramo
        let copia = preparar_copia(GIT, &sem, "agente/1.1", false, &c.destino("p9"), &[]).unwrap();
        assert_eq!((copia.pasta_trabalho.as_path(), copia.ramo.as_str()), (sem.as_path(), ""));
        assert!(!c.destino("p9").exists());
    }
}

#[test]
fn base_em_dois_ramos() {
    let c = Cena::nova();
    c.ramo("agente/a", "de-a.txt", "a");
    c.ramo("agente/b", "de-b.txt", "b");
    let destino = c.destino("p1");
    let bases = ["agente/a".to_string(), "agente/b".to_string()];
    let copia = preparar_copia(GIT, &c.repo, "agente/c", false, &destino, &bases).unwrap();
    assert_eq!(copia.base, "agente/a agente/b");
    assert_eq!((ler(&destino, "de-a.txt").as_str(), ler(&destino, "de-b.txt").as_str()), ("a", "b"));
    assert_eq!(ramo_atual(GIT, &destino).as_deref(), Some("agente/c"));

    // uma base só: parte dela, sem merge
    let uma = c.destino("p2");
    let copia = preparar_copia(GIT, &c.repo, "agente/d", false, &uma, &bases[..1]).unwrap();
    assert_eq!(copia.base, "agente/a");
    assert!(uma.join("de-a.txt").exists() && !uma.join("de-b.txt").exists());
}

#[test]
fn conflito_de_base_aborta_e_remove() {
    let c = Cena::nova();
    c.ramo("agente/a", "a.txt", "de a\n");
    c.ramo("agente/b", "a.txt", "de b\n");
    let destino = c.destino("p1");
    let bases = ["agente/a".to_string(), "agente/b".to_string()];
    assert_eq!(preparar_copia(GIT, &c.repo, "agente/c", false, &destino, &bases), Err("base-em-conflito".to_string()));
    assert!(!destino.exists(), "a cópia sai");
    assert_eq!(g(&c.repo, &["branch", "--list", "agente/c"]), "", "o ramo novo sai");
    assert_eq!(g(&c.repo, &["worktree", "list", "--porcelain"]).matches("worktree ").count(), 1);
    assert!(!sujo(GIT, &c.repo), "a principal fica como estava");
    // retomar tenta tudo de novo, e dá o mesmo erro: nunca segue sem as bases
    assert_eq!(preparar_copia(GIT, &c.repo, "agente/c", true, &destino, &bases), Err("base-em-conflito".to_string()));
}

#[test]
fn integrar_e_remover() {
    let c = Cena::nova();
    let destino = c.copia_com_commit("agente/1.1", "novo.txt", "do agente");
    assert_eq!(adiantado_de_head(GIT, &c.repo, "agente/1.1"), Some(1));
    integrar(GIT, &c.repo, "main", "agente/1.1").unwrap();
    assert_eq!(ler(&c.repo, "novo.txt"), "do agente");
    assert_eq!(g(&c.repo, &["rev-list", "--count", "--merges", "HEAD"]), "1", "--no-ff faz commit de merge");
    assert_eq!(adiantado_de_head(GIT, &c.repo, "agente/1.1"), Some(0));

    // com alteração na cópia, `worktree remove` sem --force recusa e nada se perde
    escrever(&destino, "esquecido.txt", "não commitado");
    assert!(remover_copia(GIT, &c.repo, &destino, "agente/1.1").unwrap_err().starts_with("worktree-remove"));
    assert_eq!(ler(&destino, "esquecido.txt"), "não commitado");
    std::fs::remove_file(destino.join("esquecido.txt")).unwrap();

    remover_copia(GIT, &c.repo, &destino, "agente/1.1").unwrap();
    assert!(!destino.exists());
    assert_eq!(g(&c.repo, &["branch", "--list", "agente/1.1"]), "");
}

#[test]
fn integrar_conflito_aborta() {
    let c = Cena::nova();
    let destino = c.copia_com_commit("agente/1.1", "a.txt", "do agente\n");
    commit(&c.repo, "a.txt", "do usuario\n");
    let ponta = g(&c.repo, &["rev-parse", "HEAD"]);
    assert_eq!(integrar(GIT, &c.repo, "main", "agente/1.1"), Err(Recusa::Conflito));
    assert_eq!(Recusa::Conflito.codigo(), "conflito");
    assert!(!c.merge_head(), "o merge deste comando foi abortado");
    assert_eq!(ler(&c.repo, "a.txt"), "do usuario\n");
    assert_eq!(g(&c.repo, &["rev-parse", "HEAD"]), ponta);
    assert!(destino.is_dir(), "ramo e cópia ficam");
    assert_eq!(ocupada(GIT, &c.repo), None);
}

#[test]
fn conflito_preserva_alteracao_local() {
    let c = Cena::nova();
    commit(&c.repo, "outro.txt", "original\n");
    c.copia_com_commit("agente/1.1", "a.txt", "do agente\n");
    commit(&c.repo, "a.txt", "do usuario\n");
    escrever(&c.repo, "outro.txt", "alterado sem commit\n");
    escrever(&c.repo, "solto.txt", "sem add\n");
    assert_eq!(integrar(GIT, &c.repo, "main", "agente/1.1"), Err(Recusa::Conflito));
    assert_eq!(ler(&c.repo, "outro.txt"), "alterado sem commit\n", "o abort não leva o que não é dele");
    assert_eq!(ler(&c.repo, "solto.txt"), "sem add\n");
    assert_eq!(ler(&c.repo, "a.txt"), "do usuario\n");
}

#[test]
fn alteracao_local_sem_sobreposicao_integra() {
    let c = Cena::nova();
    commit(&c.repo, "outro.txt", "original\n");
    c.copia_com_commit("agente/1.1", "novo.txt", "do agente\n");
    escrever(&c.repo, "outro.txt", "alterado sem commit\n");
    integrar(GIT, &c.repo, "main", "agente/1.1").unwrap();
    assert_eq!(ler(&c.repo, "novo.txt"), "do agente\n");
    assert_eq!(ler(&c.repo, "outro.txt"), "alterado sem commit\n");
}

#[test]
fn sobreposicao_principal_suja_sem_abort() {
    let c = Cena::nova();
    c.copia_com_commit("agente/1.1", "a.txt", "do agente\n");
    escrever(&c.repo, "a.txt", "alterado sem commit\n");
    let (git, registro) = espiao(&c);
    let ponta = g(&c.repo, &["rev-parse", "HEAD"]);
    assert_eq!(integrar(&git, &c.repo, "main", "agente/1.1"), Err(Recusa::Suja));
    assert_eq!(ler(&c.repo, "a.txt"), "alterado sem commit\n");
    assert_eq!(g(&c.repo, &["rev-parse", "HEAD"]), ponta);
    assert!(!c.merge_head());
    let feitas = chamadas(&registro);
    assert!(!feitas.contains("--abort"), "sem abort: {feitas}");

    // arquivo solto que o ramo traz: o git recusa do mesmo jeito
    let d = Cena::nova();
    d.copia_com_commit("agente/1.1", "novo.txt", "do agente\n");
    escrever(&d.repo, "novo.txt", "solto\n");
    assert_eq!(integrar(GIT, &d.repo, "main", "agente/1.1"), Err(Recusa::Suja));
    assert_eq!(ler(&d.repo, "novo.txt"), "solto\n");
}

#[test]
fn merge_em_curso_recusa_sem_tocar() {
    let c = Cena::nova();
    c.copia_com_commit("agente/1.1", "novo.txt", "do agente\n");
    // o usuário está no meio de um merge com conflito, de outro ramo
    c.ramo("dele", "a.txt", "do ramo dele\n");
    commit(&c.repo, "a.txt", "da main\n");
    let tentativa = Command::new(GIT).args(["merge", "--no-edit", "dele"]).current_dir(&c.repo).output().unwrap();
    assert!(!tentativa.status.success());
    let conflitado = ler(&c.repo, "a.txt");
    assert!(conflitado.contains("<<<<<<<"));
    let cabeca = std::fs::read_to_string(c.repo.join(".git/MERGE_HEAD")).unwrap();

    let (git, registro) = espiao(&c);
    assert_eq!(integrar(&git, &c.repo, "main", "agente/1.1"), Err(Recusa::Ocupada("MERGE_HEAD")));
    assert_eq!(Recusa::Ocupada("MERGE_HEAD").codigo(), "principal-ocupada");
    assert_eq!(std::fs::read_to_string(c.repo.join(".git/MERGE_HEAD")).unwrap(), cabeca, "o merge dele segue");
    assert_eq!(ler(&c.repo, "a.txt"), conflitado);
    let feitas = chamadas(&registro);
    assert!(!feitas.lines().any(|l| l.starts_with("merge")), "nem chega a chamar merge: {feitas}");
}

#[test]
fn ramo_mudou_recusa() {
    let c = Cena::nova();
    c.copia_com_commit("agente/1.1", "novo.txt", "do agente\n");
    g(&c.repo, &["checkout", "-q", "-b", "outro"]);
    let ponta = g(&c.repo, &["rev-parse", "HEAD"]);
    assert_eq!(integrar(GIT, &c.repo, "main", "agente/1.1"), Err(Recusa::RamoMudou));
    assert_eq!(g(&c.repo, &["rev-parse", "HEAD"]), ponta);
    // HEAD destacado também não é o ramo que a tela mostrou
    g(&c.repo, &["checkout", "-q", "--detach"]);
    assert_eq!(integrar(GIT, &c.repo, "main", "agente/1.1"), Err(Recusa::RamoMudou));
}

#[test]
fn ramo_hifen_recusado_antes_do_git() {
    // um git que não existe: se algo tentasse rodá-lo, o erro seria outro
    let c = Cena::vazia();
    let sem_git = "/nao/existe/git";
    for ramo in ["-x", "--upload-pack=x", "agente/-x", "a..b", "a b", "a;b", "", "x.lock", "HEAD^"] {
        assert_eq!(preparar_copia(sem_git, &c.repo, ramo, false, &c.destino("p1"), &[]), Err("ramo-invalido".to_string()), "{ramo}");
        assert_eq!(integrar(sem_git, &c.repo, "main", ramo), Err(Recusa::RamoInvalido), "{ramo}");
        assert_eq!(remover_copia(sem_git, &c.repo, &c.destino("p1"), ramo), Err("ramo-invalido".to_string()), "{ramo}");
        assert_eq!(adiantado_de_head(sem_git, &c.repo, ramo), None, "{ramo}");
    }
    assert_eq!(preparar_copia(sem_git, &c.repo, "agente/1.1", false, &c.destino("p1"), &["-x".to_string()]), Err("ramo-invalido".to_string()));

    // e com o espião: nem uma chamada ao git
    let (git, registro) = espiao(&c);
    assert_eq!(preparar_copia(&git, &c.repo, "-x", false, &c.destino("p1"), &[]), Err("ramo-invalido".to_string()));
    assert_eq!(integrar(&git, &c.repo, "main", "--force"), Err(Recusa::RamoInvalido));
    assert_eq!(chamadas(&registro), "");
}

#[test]
fn sem_commit_erro() {
    let c = Cena::vazia();
    let destino = c.destino("p1");
    assert_eq!(preparar_copia(GIT, &c.repo, "agente/1.1", false, &destino, &[]), Err("repositorio-sem-commit".to_string()));
    assert!(!destino.exists());
}

#[test]
fn destino_alheio_nao_e_tocado() {
    let c = Cena::nova();
    let destino = c.destino("p1");
    escrever(&destino, "alheio.txt", "de alguém");
    assert_eq!(preparar_copia(GIT, &c.repo, "agente/1.1", false, &destino, &[]), Err("destino-ocupado".to_string()));
    assert_eq!(ler(&destino, "alheio.txt"), "de alguém");
}

#[cfg(unix)]
#[test]
fn tempo_esgotado_mata_o_git() {
    use std::os::unix::fs::PermissionsExt;
    let c = Cena::vazia();
    let script = c.base.join("git-lento.sh");
    let pid = c.base.join("pid.txt");
    std::fs::write(&script, format!("#!/bin/sh\necho $$ > \"{}\"\nexec sleep 30\n", pid.display())).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut espera = std::time::Duration::from_millis(600);
    let mut saida = super::git::rodar(script.to_str().unwrap(), &c.repo, &["merge"], espera);
    // ETXTBSY num script recém-escrito: tenta de novo até ele rodar
    while saida.erro.starts_with("git: ") && espera < std::time::Duration::from_secs(3) {
        std::thread::sleep(std::time::Duration::from_millis(50));
        espera += std::time::Duration::from_millis(100);
        saida = super::git::rodar(script.to_str().unwrap(), &c.repo, &["merge"], espera);
    }
    assert!(!saida.ok);
    assert_eq!(saida.erro, "git merge: tempo esgotado");
    let numero = std::fs::read_to_string(&pid).unwrap();
    assert!(!Path::new(&format!("/proc/{}", numero.trim())).exists(), "o processo foi morto e colhido");
}
