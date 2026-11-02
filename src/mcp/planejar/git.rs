//! O git do Planejar: a cópia (worktree) de cada plano e a integração do ramo dele. Sem shell, com `--` antes de ramo, `GIT_TERMINAL_PROMPT=0` e tempo-limite com kill.
//! Passa de 300 linhas (teto 500) porque preparar e integrar dividem o mesmo `rodar` e os mesmos marcadores, e a fatia é dona só deste arquivo; o corte natural é `rodar` num módulo próprio.
//! Nada aqui segura o `Mutex<Dados>`: quem chama (motor e gesto) faz o git fora dele. Os erros de `preparar_copia` são os códigos que vão ao `motivo` da execução.
#![allow(dead_code)] // quem usa são `motor` e `gestos`, de outras fatias
use crate::plano::ids::sha256_hex;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const LIMITE: Duration = Duration::from_secs(20);
/// `worktree add` e `merge`: matar no meio deixa `index.lock`, então recebem bem mais tempo.
const LIMITE_LONGO: Duration = Duration::from_secs(300);
/// Variáveis que, herdadas (app aberto de dentro de um hook, por exemplo), fariam o git mexer em outro repositório.
const VARS_DE_REPOSITORIO: [&str; 7] = ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR", "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_PREFIX"];
/// O que o git deixa na pasta principal enquanto há operação pela metade.
const EM_CURSO: [(&str, &str); 6] = [("MERGE_HEAD", "MERGE_HEAD"), ("REBASE_HEAD", "REBASE_HEAD"), ("rebase-merge", "REBASE_HEAD"), ("rebase-apply", "REBASE_HEAD"), ("CHERRY_PICK_HEAD", "CHERRY_PICK_HEAD"), ("REVERT_HEAD", "REVERT_HEAD")];

/// A cópia pronta. Sem git, `raiz` e `pasta_trabalho` são a própria pasta e `ramo` fica vazio (um agente por pasta).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Copia {
    pub raiz: PathBuf,
    /// `<cópia>/<subpasta relativa>`, quando o blueprint está ligado a uma subpasta do repositório.
    pub pasta_trabalho: PathBuf,
    /// Pode ter sufixo: é este, e não o pedido, que a execução grava.
    pub ramo: String,
    /// `HEAD` ou os ramos de base; vazio ao reusar (a linha da execução já tem o valor certo).
    pub base: String,
    pub reusada: bool,
}

/// Por que a integração não aconteceu. `codigo` é o `codigo` do 409 de `gesto/integrar`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recusa {
    RamoInvalido,
    RamoMudou,
    Ocupada(&'static str),
    Suja,
    Conflito,
    Falhou(String),
}

impl Recusa {
    pub fn codigo(&self) -> &'static str {
        match self {
            Recusa::RamoInvalido => "ramo-invalido",
            Recusa::RamoMudou => "ramo-mudou",
            Recusa::Ocupada(_) => "principal-ocupada",
            Recusa::Suja => "principal-suja",
            Recusa::Conflito => "conflito",
            Recusa::Falhou(_) => "git-falhou",
        }
    }
}

/// O programa `git`: `VAR_GIT` troca nos testes.
pub fn programa() -> String {
    std::env::var(crate::repositorio::acesso::VAR_GIT).ok().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "git".into())
}

pub(super) struct Saida {
    pub(super) ok: bool,
    pub(super) saida: String,
    pub(super) erro: String,
}

pub(super) fn rodar(git: &str, pasta: &Path, args: &[&str], limite: Duration) -> Saida {
    let mut c = Command::new(git);
    c.args(args).current_dir(pasta).env("GIT_TERMINAL_PROMPT", "0").env("GCM_INTERACTIVE", "never").env("LC_ALL", "C").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    for v in VARS_DE_REPOSITORIO {
        c.env_remove(v);
    }
    sem_janela(&mut c);
    let mut filho = match c.spawn() {
        Ok(f) => f,
        Err(e) => return Saida { ok: false, saida: String::new(), erro: format!("git: {e}") },
    };
    let ler = |fonte: Option<Box<dyn Read + Send>>| {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut texto = Vec::new();
            if let Some(mut f) = fonte {
                let _ = f.read_to_end(&mut texto);
            }
            let _ = tx.send(String::from_utf8_lossy(&texto).into_owned());
        });
        rx
    };
    // leitura em thread: saída grande não trava o git; um hook que herdou o cano não trava este lado (o `recv_timeout` desiste)
    let saida = ler(filho.stdout.take().map(|f| Box::new(f) as Box<dyn Read + Send>));
    let erro = ler(filho.stderr.take().map(|f| Box::new(f) as Box<dyn Read + Send>));
    let fim = Instant::now() + limite;
    let ok = loop {
        match filho.try_wait() {
            Ok(Some(estado)) => break Some(estado.success()),
            Ok(None) if Instant::now() >= fim => {
                let _ = filho.kill();
                let _ = filho.wait();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(_) => break Some(false),
        }
    };
    let colher = |rx: mpsc::Receiver<String>| rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default();
    let (saida, erro) = (colher(saida), colher(erro));
    match ok {
        Some(ok) => Saida { ok, saida, erro },
        None => Saida { ok: false, saida, erro: format!("git {}: tempo esgotado", args.first().unwrap_or(&"")) },
    }
}

#[cfg(windows)]
fn sem_janela(c: &mut Command) {
    use std::os::windows::process::CommandExt;
    c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
}
#[cfg(not(windows))]
fn sem_janela(_: &mut Command) {}

fn linha(s: &Saida) -> Option<String> {
    let t = s.saida.trim();
    (s.ok && !t.is_empty()).then(|| t.to_string())
}

/// O primeiro pedaço do erro do git, curto o bastante para o `motivo` da execução.
fn resumo(s: &Saida) -> String {
    s.erro.trim().chars().take(200).collect()
}

/// Nome que pode chegar ao git como ramo ou base: nada que ele leia como opção nem como faixa. A checagem vem antes de qualquer processo.
pub fn ramo_seguro(r: &str) -> bool {
    let forma = !r.is_empty() && r.len() <= 200 && r.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-' | b'/'));
    forma && !r.contains("..") && r.split('/').all(|p| !p.is_empty() && !p.starts_with(['-', '.']) && !p.ends_with('.') && !p.to_ascii_lowercase().ends_with(".lock"))
}

/// A raiz da árvore de trabalho que contém `pasta`.
pub fn raiz(git: &str, pasta: &Path) -> Option<PathBuf> {
    linha(&rodar(git, pasta, &["rev-parse", "--show-toplevel"], LIMITE)).map(PathBuf::from)
}

/// O `.git` comum, canônico: é igual na pasta principal e em todas as cópias do mesmo repositório.
pub fn repositorio(git: &str, pasta: &Path) -> Option<String> {
    let comum = linha(&rodar(git, pasta, &["rev-parse", "--git-common-dir"], LIMITE))?;
    let caminho = std::fs::canonicalize(pasta.join(comum)).ok()?;
    let texto = caminho.to_string_lossy();
    Some(texto.strip_prefix(r"\\?\").unwrap_or(&texto).to_string())
}

/// Sobe procurando `.git`, sem abrir processo: roda a cada tique do motor.
pub fn tem_git(pasta: &Path) -> bool {
    pasta.ancestors().any(|p| p.join(".git").exists())
}

/// Operação pela metade na pasta: o nome do marcador. Vale também na cópia.
pub fn ocupada(git: &str, principal: &Path) -> Option<&'static str> {
    EM_CURSO.iter().find(|(arquivo, _)| marcador(git, principal, arquivo)).map(|(_, nome)| *nome)
}

fn marcador(git: &str, pasta: &Path, arquivo: &str) -> bool {
    linha(&rodar(git, pasta, &["rev-parse", "--git-path", arquivo], LIMITE)).is_some_and(|p| pasta.join(p).exists())
}

/// O ramo em que a pasta está; `None` com HEAD destacado.
pub fn ramo_atual(git: &str, pasta: &Path) -> Option<String> {
    linha(&rodar(git, pasta, &["symbolic-ref", "--quiet", "--short", "HEAD"], LIMITE))
}

/// Quantos commits `ramo` tem que o HEAD de `pasta` não tem. Zero: o agente não fez commit (ou já está integrado).
pub fn adiantado_de_head(git: &str, pasta: &Path, ramo: &str) -> Option<u64> {
    if !ramo_seguro(ramo) {
        return None;
    }
    linha(&rodar(git, pasta, &["rev-list", "--count", &format!("HEAD..refs/heads/{ramo}"), "--"], LIMITE))?.parse().ok()
}

/// Há alteração sem commit? Sem resposta do git, conta como limpa: isto só vira aviso na linha.
pub fn sujo(git: &str, pasta: &Path) -> bool {
    linha(&rodar(git, pasta, &["status", "--porcelain"], LIMITE)).is_some()
}

fn ramo_existe(git: &str, pasta: &Path, ramo: &str) -> bool {
    rodar(git, pasta, &["show-ref", "--verify", "--quiet", &format!("refs/heads/{ramo}")], LIMITE).ok
}

/// O ramo está em uso em alguma árvore de trabalho deste repositório (a principal inclusive)?
fn ramo_em_uso(git: &str, pasta: &Path, ramo: &str) -> bool {
    let alvo = format!("branch refs/heads/{ramo}");
    rodar(git, pasta, &["worktree", "list", "--porcelain"], LIMITE).saida.lines().any(|l| l.trim() == alvo)
}

/// 4 hexadecimais do blueprint: o destino é `<copias>/<12 hex do bp>/<plano>`, e a pasta pai já os traz.
fn sufixo(destino: &Path) -> String {
    let pai = destino.parent().and_then(Path::file_name).map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if pai.len() >= 4 && pai.bytes().all(|c| c.is_ascii_hexdigit()) {
        return pai[..4].to_string();
    }
    sha256_hex(pai.as_bytes())[..4].to_string()
}

fn derivado_de(candidato: &str, ramo: &str, suf: &str) -> bool {
    candidato.strip_prefix(ramo).and_then(|r| r.strip_prefix('-')).is_some_and(|r| r == suf || r.strip_prefix(suf).is_some_and(|n| n.starts_with('-')))
}

/// Se `destino` é uma árvore de trabalho deste repositório com a raiz nele, o ramo em que está.
fn ramo_da_copia(git: &str, principal: &Path, destino: &Path) -> Option<String> {
    let propria = std::fs::canonicalize(raiz(git, destino)?).ok()? == std::fs::canonicalize(destino).ok()?;
    (propria && repositorio(git, destino)? == repositorio(git, principal)?).then(|| ramo_atual(git, destino))?
}

/// A pasta de trabalho dentro da cópia. A subpasta pode não estar no ramo (vazia ou fora do git): nasce, para o agente ter onde entrar.
fn subpasta_na_copia(ligada: &Path, raiz_principal: &Path, destino: &Path) -> Result<PathBuf, String> {
    let (ligada, base) = (std::fs::canonicalize(ligada).map_err(|e| e.to_string())?, std::fs::canonicalize(raiz_principal).map_err(|e| e.to_string())?);
    let relativa = ligada.strip_prefix(&base).map_err(|_| "pasta-fora-do-repositorio".to_string())?;
    let pasta = destino.join(relativa);
    std::fs::create_dir_all(&pasta).map_err(|e| e.to_string())?;
    Ok(pasta)
}

/// Desfaz o que este preparo criou. `--force` só aqui: a cópia nasceu agora e não tem trabalho de ninguém.
fn limpar(git: &str, principal: &Path, destino: &Path, ramo_criado: Option<&str>) {
    let caminho = destino.to_string_lossy();
    if !rodar(git, principal, &["worktree", "remove", "--force", "--", &caminho], LIMITE_LONGO).ok && destino.exists() {
        let _ = std::fs::remove_dir_all(destino);
    }
    rodar(git, principal, &["worktree", "prune"], LIMITE);
    if let Some(r) = ramo_criado {
        rodar(git, principal, &["branch", "-D", "--", r], LIMITE);
    }
}

/// O nome que o plano usa: o pedido, se livre e sem dono alheio; senão `<ramo>-<sufixo>`. Nunca adota ramo do usuário nem o de outra cópia.
fn escolher_ramo(git: &str, principal: &Path, ramo: &str, ramo_nosso: bool, destino: &Path) -> Result<(String, bool), String> {
    let existe = ramo_existe(git, principal, ramo);
    if !existe {
        return Ok((ramo.to_string(), true));
    }
    if ramo_nosso && !ramo_em_uso(git, principal, ramo) {
        return Ok((ramo.to_string(), false));
    }
    let suf = sufixo(destino);
    let candidatos = std::iter::once(format!("{ramo}-{suf}")).chain((2..10).map(|n| format!("{ramo}-{suf}-{n}")));
    candidatos.into_iter().find(|c| !ramo_existe(git, principal, c)).map(|c| (c, true)).ok_or_else(|| "ramo-sem-nome-livre".to_string())
}

/// Prepara a cópia do plano (contrato, seção 7 e D10). `principal` é a pasta ligada ao blueprint, que pode ser subpasta do repositório; `ramo_nosso` = registrado em execução deste plano.
/// Zero bases: parte do HEAD. Uma: desse ramo. Várias: da primeira, com `merge` das outras; conflito desfaz tudo, e retomar tenta tudo de novo.
pub fn preparar_copia(git: &str, principal: &Path, ramo: &str, ramo_nosso: bool, destino: &Path, bases: &[String]) -> Result<Copia, String> {
    if !ramo_seguro(ramo) || !bases.iter().all(|b| ramo_seguro(b)) {
        return Err("ramo-invalido".into());
    }
    if !tem_git(principal) {
        return Ok(Copia { raiz: principal.to_path_buf(), pasta_trabalho: principal.to_path_buf(), ramo: String::new(), base: String::new(), reusada: false });
    }
    let raiz_principal = raiz(git, principal).ok_or("git-indisponivel")?;
    if !rodar(git, &raiz_principal, &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"], LIMITE).ok {
        return Err("repositorio-sem-commit".into());
    }
    let cheio = destino.read_dir().is_ok_and(|mut d| d.next().is_some());
    if cheio {
        let Some(atual) = ramo_da_copia(git, &raiz_principal, destino) else { return Err("destino-ocupado".into()) };
        if atual != ramo && !derivado_de(&atual, ramo, &sufixo(destino)) {
            return Err("copia-em-outro-ramo".into());
        }
        let pasta_trabalho = subpasta_na_copia(principal, &raiz_principal, destino)?;
        return Ok(Copia { raiz: destino.to_path_buf(), pasta_trabalho, ramo: atual, base: String::new(), reusada: true });
    }
    // registrado e ausente (apagado à mão): sem o prune, o `add` recusa
    rodar(git, &raiz_principal, &["worktree", "prune"], LIMITE);
    let (escolhido, novo) = escolher_ramo(git, &raiz_principal, ramo, ramo_nosso, destino)?;
    if let Some(pai) = destino.parent() {
        std::fs::create_dir_all(pai).map_err(|e| e.to_string())?;
    }
    let caminho = destino.to_string_lossy();
    let partida = bases.first().map_or("HEAD", String::as_str);
    let args: Vec<&str> = if novo { vec!["worktree", "add", "-b", &escolhido, "--", &caminho, partida] } else { vec!["worktree", "add", "--", &caminho, &escolhido] };
    let criado = novo.then_some(escolhido.as_str());
    let adicao = rodar(git, &raiz_principal, &args, LIMITE_LONGO);
    if !adicao.ok {
        limpar(git, &raiz_principal, destino, criado);
        return Err(format!("worktree-add: {}", resumo(&adicao)));
    }
    // ramo que já existia traz as bases de quando nasceu; só ramo novo recebe o merge
    for outra in bases.iter().skip(1).filter(|_| novo) {
        let fusao = rodar(git, destino, &["merge", "--no-edit", "--", outra], LIMITE_LONGO);
        if fusao.ok {
            continue;
        }
        let conflito = marcador(git, destino, "MERGE_HEAD");
        if conflito {
            rodar(git, destino, &["merge", "--abort"], LIMITE_LONGO);
        }
        limpar(git, &raiz_principal, destino, criado);
        return Err(if conflito { "base-em-conflito".into() } else { format!("base-falhou: {}", resumo(&fusao)) });
    }
    let pasta_trabalho = match subpasta_na_copia(principal, &raiz_principal, destino) {
        Ok(p) => p,
        Err(e) => {
            limpar(git, &raiz_principal, destino, criado);
            return Err(e);
        }
    };
    let base = if bases.is_empty() { "HEAD".to_string() } else { bases.join(" ") };
    Ok(Copia { raiz: destino.to_path_buf(), pasta_trabalho, ramo: escolhido, base, reusada: false })
}

/// `git merge --no-ff --no-edit -- <ramo>` na pasta principal, se ela segue em `ramo_esperado` e sem operação pela metade. Só aborta o merge que este comando abriu; alteração local é do usuário.
pub fn integrar(git: &str, principal: &Path, ramo_esperado: &str, ramo: &str) -> Result<(), Recusa> {
    if !ramo_seguro(ramo) {
        return Err(Recusa::RamoInvalido);
    }
    if ramo_atual(git, principal).as_deref() != Some(ramo_esperado) {
        return Err(Recusa::RamoMudou);
    }
    if let Some(nome) = ocupada(git, principal) {
        return Err(Recusa::Ocupada(nome));
    }
    let fusao = rodar(git, principal, &["merge", "--no-ff", "--no-edit", "--", ramo], LIMITE_LONGO);
    if fusao.ok {
        return Ok(());
    }
    if marcador(git, principal, "MERGE_HEAD") {
        let aborto = rodar(git, principal, &["merge", "--abort"], LIMITE_LONGO);
        return if aborto.ok { Err(Recusa::Conflito) } else { Err(Recusa::Falhou(resumo(&aborto))) };
    }
    if fusao.erro.contains("would be overwritten") || fusao.erro.contains("commit your changes or stash them") {
        return Err(Recusa::Suja);
    }
    Err(Recusa::Falhou(resumo(&fusao)))
}

/// `worktree remove` (sem `--force`: cópia com alteração fica, e o erro volta) e `branch -d`. `copia` pode ser a pasta de trabalho, em subpasta ou já apagada à mão.
pub fn remover_copia(git: &str, principal: &Path, copia: &Path, ramo: &str) -> Result<(), String> {
    if !ramo_seguro(ramo) {
        return Err("ramo-invalido".into());
    }
    let raiz_principal = raiz(git, principal).ok_or("git-indisponivel")?;
    let viva = copia.ancestors().find(|p| p.is_dir()).and_then(|p| raiz(git, p)).filter(|r| repositorio(git, r) == repositorio(git, &raiz_principal));
    if let Some(alvo) = viva {
        if std::fs::canonicalize(&alvo).ok() == std::fs::canonicalize(&raiz_principal).ok() {
            return Err("copia-e-a-principal".into());
        }
        let remocao = rodar(git, &raiz_principal, &["worktree", "remove", "--", &alvo.to_string_lossy()], LIMITE_LONGO);
        if !remocao.ok {
            return Err(format!("worktree-remove: {}", resumo(&remocao)));
        }
    }
    rodar(git, &raiz_principal, &["worktree", "prune"], LIMITE);
    let apagado = rodar(git, &raiz_principal, &["branch", "-d", "--", ramo], LIMITE);
    if apagado.ok { Ok(()) } else { Err(format!("branch-d: {}", resumo(&apagado))) }
}
