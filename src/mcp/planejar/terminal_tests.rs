//! Testes de mcp::planejar::terminal: o terminal só recebe o id; sem terminal é erro, nunca rodar invisível.
use super::terminal::{args_do_agente, lancar_com};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const ID: &str = "0123456789abcdef";

fn pasta_de_teste(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-terminal-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn recusa(args: &[OsString]) -> String {
    // o procurar dá erro de teste se for chamado: a recusa vem antes de buscar terminal
    lancar_com(&|t| panic!("procurou {t}"), Path::new("."), Path::new("blueprint"), args).unwrap_err()
}

#[cfg(unix)]
fn terminal_falso(pasta: &Path) -> (PathBuf, PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let (script, registro) = (pasta.join("terminal.sh"), pasta.join("registro.txt"));
    let corpo = format!("#!/bin/sh\n{{ pwd; for a in \"$@\"; do printf '%s\\n' \"$a\"; done; }} > \"{0}.tmp\"\nmv \"{0}.tmp\" \"{0}\"\n", registro.display());
    std::fs::write(&script, corpo).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    (script, registro)
}

#[test]
fn terminal_so_recebe_id() {
    assert_eq!(args_do_agente(ID), vec![OsString::from("agente-plano"), OsString::from(ID)]);

    // qualquer outra forma não passa: texto de plano, id curto, argumento a mais, outro subcomando
    let texto: OsString = "fazer; rm -rf /".into();
    let casos: Vec<Vec<OsString>> = vec![
        vec![],
        vec!["agente-plano".into()],
        vec!["agente-plano".into(), texto.clone()],
        vec!["agente-plano".into(), "0123".into()],
        vec!["agente-plano".into(), "0123456789abcdeg".into()],
        vec!["agente-plano".into(), ID.into(), texto.clone()],
        vec!["claude".into(), ID.into()],
        vec![texto],
    ];
    for caso in &casos {
        assert_eq!(recusa(caso), "argumentos-recusados", "{caso:?}");
    }

    // e o terminal de verdade recebe exatamente: exe, agente-plano, id, já na pasta
    #[cfg(unix)]
    {
        let pasta = pasta_de_teste("id");
        let (script, registro) = terminal_falso(&pasta);
        let exe = Path::new("/caminho/do/blueprint");
        let procurar = |t: &str| (t == "xdg-terminal-exec").then(|| script.clone());
        let mut quem = Err(String::new());
        // recém-escrito, o script pode dar ETXTBSY se outra thread do teste estiver em fork
        for _ in 0..50 {
            quem = lancar_com(&procurar, &pasta, exe, &args_do_agente(ID));
            if quem.is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert_eq!(quem, Ok("xdg-terminal-exec"));
        let mut lido = String::new();
        for _ in 0..250 {
            if let Ok(t) = std::fs::read_to_string(&registro) {
                lido = t;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let linhas: Vec<&str> = lido.lines().collect();
        assert_eq!(linhas.len(), 4, "{lido}");
        assert_eq!(std::fs::canonicalize(linhas[0]).unwrap(), std::fs::canonicalize(&pasta).unwrap());
        assert_eq!(&linhas[1..], ["/caminho/do/blueprint", "agente-plano", ID]);
        let _ = std::fs::remove_dir_all(&pasta);
    }
}

#[cfg(not(windows))]
#[test]
fn sem_terminal_e_erro() {
    let pasta = pasta_de_teste("sem");
    assert_eq!(lancar_com(&|_| None, &pasta, Path::new("/nao/roda"), &args_do_agente(ID)), Err("sem-terminal".to_string()));
    let _ = std::fs::remove_dir_all(&pasta);
}
