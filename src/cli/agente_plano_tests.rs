//! O invólucro roda de verdade: cada caso abre este mesmo binário de teste num processo próprio (`auxiliar`), porque `executar` troca a pasta atual e, como líder de
//! grupo, mata o próprio grupo. O agente é um script e o app é um servidor falso; `cfg!(test)` encurta os intervalos.
use super::*;
use apoio::*;
use std::fs;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::PathBuf;

#[path = "agente_plano_apoio.rs"]
mod apoio;

/// Não testa nada sozinho: é a entrada do processo próprio de cada caso.
#[test]
fn auxiliar() {
    if let Ok(id) = std::env::var("BLUEPRINT_TESTE_INVOLUCRO") {
        std::process::exit(executar(&id));
    }
}

#[test]
fn id_invalido_sai() {
    for id in ["", "abc", "0123456789abcde", "0123456789abcdef0", "0123456789abcdeg", "../../etc/passwd", "; rm -rf x #####"] {
        assert_eq!(executar(id), 2, "{id:?} não é id");
    }
    assert_eq!(crate::cli::executar(vec!["agente-plano".into(), ID.into(), "prompt solto".into()]), 2, "só o id cabe na linha de comando");
}

#[test]
fn manda_sinal_e_fim() {
    let pasta = pasta_temporaria("sinal");
    let agente = script(&pasta, "agente", "sleep 0.5\nexit 7");
    let s = servidor(200, partida(&[&agente], &pasta, "faça"));
    assert_eq!(rodar_ate_o_fim(&s).code(), Some(7), "sai com o código do agente");
    assert_eq!(s.de("execucao/partida"), [json!({ "id": ID })]);
    let sinais = s.de("execucao/sinal");
    assert!(sinais.len() >= 2, "um sinal por intervalo: {sinais:?}");
    assert!(sinais.iter().all(|c| *c == json!({ "id": ID, "segredo": SEGREDO })), "{sinais:?}");
    assert_eq!(s.de("execucao/fim"), [json!({ "id": ID, "segredo": SEGREDO, "codigo": 7 })]);
    let _ = fs::remove_dir_all(pasta);
}

#[test]
fn parar_mata_filho() {
    let pasta = pasta_temporaria("parar");
    let agente = script(&pasta, "agente", &format!("echo $$ > '{}/pid'\nexec sleep 30", pasta.display()));
    let s = servidor(200, partida(&[&agente], &pasta, "faça"));
    let mut filho = involucro(ID, &[("BLUEPRINT_URL", &s.url())]).spawn().unwrap();
    assert!(esperar_ate(|| fs::read_to_string(pasta.join("pid")).is_ok_and(|t| t.ends_with('\n'))), "o agente subiu");
    let pid: i32 = fs::read_to_string(pasta.join("pid")).unwrap().trim().parse().unwrap();
    assert!(!morto(pid));
    s.parar.store(true, Ordering::SeqCst);
    assert_eq!(terminar(&mut filho).code(), Some(CODIGO_PARADO));
    assert!(esperar_ate(|| morto(pid)), "o agente morreu junto");
    assert_eq!(s.de("execucao/fim"), [json!({ "id": ID, "segredo": SEGREDO, "codigo": CODIGO_PARADO })], "um fim só");
    let _ = fs::remove_dir_all(pasta);
}

#[test]
fn parar_como_lider_leva_os_netos() {
    let pasta = pasta_temporaria("lider");
    let agente = script(&pasta, "agente", &format!("sleep 30 &\necho $! > '{d}/neto'\necho $$ > '{d}/pid'\nwait", d = pasta.display()));
    let s = servidor(200, partida(&[&agente], &pasta, "faça"));
    let mut comando = involucro(ID, &[("BLUEPRINT_URL", &s.url())]);
    comando.process_group(0); // como no terminal: o invólucro é o líder
    let mut filho = comando.spawn().unwrap();
    assert!(esperar_ate(|| fs::read_to_string(pasta.join("pid")).is_ok_and(|t| t.ends_with('\n'))), "o agente subiu");
    let ler = |nome: &str| fs::read_to_string(pasta.join(nome)).unwrap().trim().parse::<i32>().unwrap();
    let (pid, neto) = (ler("pid"), ler("neto"));
    s.parar.store(true, Ordering::SeqCst);
    assert_eq!(terminar(&mut filho).signal(), Some(libc::SIGTERM), "o invólucro sai com o grupo");
    assert!(esperar_ate(|| morto(pid) && morto(neto)), "filho e neto morreram");
    assert_eq!(s.de("execucao/fim").len(), 1, "o fim vai antes de matar");
    let _ = fs::remove_dir_all(pasta);
}

#[test]
fn prompt_fora_da_linha_do_terminal() {
    let pasta = pasta_temporaria("prompt");
    let (m1, m2) = (pasta.join("marca1"), pasta.join("marca2"));
    let prompt = format!("texto; touch '{}' $(touch '{}') \"aspas\" `id`\nsegunda linha", m1.display(), m2.display());
    let corpo = format!("printf '%s\\0' \"$@\" > '{d}/args'\ncat /proc/$PPID/cmdline > '{d}/linha' 2>/dev/null\nexit 0", d = pasta.display());
    let agente = script(&pasta, "agente", &corpo);
    let s = servidor(200, partida(&[&agente, "--fixo"], &pasta, &prompt));
    assert_eq!(rodar_ate_o_fim(&s).code(), Some(0));
    let args = fs::read_to_string(pasta.join("args")).unwrap();
    assert_eq!(args.trim_end_matches('\0').split('\0').collect::<Vec<_>>(), ["--fixo", prompt.as_str()], "o prompt é um argumento só, o último, do filho");
    assert!(!m1.exists() && !m2.exists(), "nenhum shell leu o prompt");
    if let Ok(linha) = fs::read(pasta.join("linha")) {
        let linha = String::from_utf8_lossy(&linha);
        assert!(!linha.is_empty() && !linha.contains("texto;") && !linha.contains(SEGREDO), "{linha}");
    }
    let _ = fs::remove_dir_all(pasta);
}

#[test]
fn entra_na_pasta() {
    let pasta = pasta_temporaria("pasta");
    let trabalho = pasta.join("cópia do plano");
    fs::create_dir_all(&trabalho).unwrap();
    let agente = script(&pasta, "agente", &format!("pwd > '{}/pwd'", pasta.display()));
    let s = servidor(200, partida(&[&agente], &trabalho, "faça"));
    assert_eq!(rodar_ate_o_fim(&s).code(), Some(0));
    assert_eq!(PathBuf::from(fs::read_to_string(pasta.join("pwd")).unwrap().trim()), trabalho.canonicalize().unwrap());
    // pasta que sumiu: o agente não roda fora dela, e o app fica sabendo
    fs::remove_file(pasta.join("pwd")).unwrap();
    let s = servidor(200, partida(&[&agente], &pasta.join("sumiu"), "faça"));
    assert_eq!(rodar_ate_o_fim(&s).code(), Some(1));
    assert!(!pasta.join("pwd").exists(), "o agente não rodou");
    assert_eq!(s.de("execucao/fim"), [json!({ "id": ID, "segredo": SEGREDO, "codigo": 1 })]);
    let _ = fs::remove_dir_all(pasta);
}

#[test]
fn passa_url_ao_filho() {
    let pasta = pasta_temporaria("url");
    let agente = script(&pasta, "agente", &format!("env > '{}/env'", pasta.display()));
    let s = servidor(200, partida(&[&agente], &pasta, "faça"));
    anotar_porta(&pasta, s.porta); // o invólucro não tem BLUEPRINT_URL: o filho só a recebe dele
    let dados = pasta.display().to_string();
    assert_eq!(terminar(&mut involucro(ID, &[("XDG_DATA_HOME", &dados)]).spawn().unwrap()).code(), Some(0));
    let env = fs::read_to_string(pasta.join("env")).unwrap();
    for linha in [format!("BLUEPRINT_URL={}", s.url()), format!("BLUEPRINT_EXECUCAO={ID}"), format!("BLUEPRINT_EXECUCAO_SEGREDO={SEGREDO}"), "BLUEPRINT_AGENTE_NOME=plano 1.2".to_string(), "BLUEPRINT_MCP_PERFIL=executor".to_string()] {
        assert!(env.lines().any(|l| l == linha), "faltou {linha}: {env}");
    }
    let _ = fs::remove_dir_all(pasta);
}

#[test]
fn acha_porta_no_arquivo() {
    let pasta = pasta_temporaria("porta");
    let agente = script(&pasta, "agente", "exit 0");
    let s = servidor(200, partida(&[&agente], &pasta, "faça"));
    anotar_porta(&pasta, PORTA_FECHADA); // o app ainda não está nesta porta
    let dados = pasta.display().to_string();
    let mut filho = involucro(ID, &[("XDG_DATA_HOME", &dados)]).spawn().unwrap();
    sleep(Duration::from_millis(150));
    assert!(s.de("execucao/partida").is_empty());
    anotar_porta(&pasta, s.porta); // reabriu em outra porta: a tentativa seguinte relê o arquivo
    assert_eq!(terminar(&mut filho).code(), Some(0));
    assert_eq!(s.de("execucao/partida").len(), 1);
    assert_eq!(s.de("execucao/fim").len(), 1);
    let _ = fs::remove_dir_all(pasta);
}

#[test]
fn app_fechado_nao_derruba_e_o_fim_acha_a_porta_nova() {
    let pasta = pasta_temporaria("fechado");
    let liberar = pasta.join("liberar");
    let agente = script(&pasta, "agente", &format!("while [ ! -f '{}' ]; do sleep 0.02; done\nexit 3", liberar.display()));
    let s = servidor(200, partida(&[&agente], &pasta, "faça"));
    anotar_porta(&pasta, s.porta);
    let dados = pasta.display().to_string();
    let mut filho = involucro(ID, &[("XDG_DATA_HOME", &dados)]).spawn().unwrap();
    assert!(esperar_ate(|| !s.de("execucao/sinal").is_empty()), "o primeiro sinal chegou");
    anotar_porta(&pasta, PORTA_FECHADA);
    sleep(Duration::from_millis(350));
    assert!(filho.try_wait().unwrap().is_none(), "sem app, o agente segue");
    anotar_porta(&pasta, s.porta);
    fs::write(&liberar, "").unwrap();
    assert_eq!(terminar(&mut filho).code(), Some(3));
    assert_eq!(s.de("execucao/fim"), [json!({ "id": ID, "segredo": SEGREDO, "codigo": 3 })]);
    let _ = fs::remove_dir_all(pasta);
}

#[test]
fn partida_recusada_nao_roda_nada() {
    let pasta = pasta_temporaria("recusa");
    let s = servidor(409, json!({ "erro": "já partiu", "codigo": "ja-partiu" }));
    assert_eq!(rodar_ate_o_fim(&s).code(), Some(1));
    assert_eq!(s.de("execucao/partida").len(), 1, "o app respondeu: não tenta de novo");
    assert!(s.de("execucao/sinal").is_empty() && s.de("execucao/fim").is_empty());
    let _ = fs::remove_dir_all(pasta);
}

#[test]
fn comando_que_nao_existe_avisa_o_fim() {
    let pasta = pasta_temporaria("semcomando");
    let s = servidor(200, partida(&["comando-que-nao-existe-aqui"], &pasta, "faça"));
    assert_eq!(rodar_ate_o_fim(&s).code(), Some(CODIGO_SEM_COMANDO));
    assert_eq!(s.de("execucao/fim"), [json!({ "id": ID, "segredo": SEGREDO, "codigo": CODIGO_SEM_COMANDO })]);
    let _ = fs::remove_dir_all(pasta);
}

/// O caminho do Claude: o `claude::montar_comando` de verdade, com um `claude` falso no PATH.
#[test]
fn claude_recebe_o_prompt_com_plugin_sem_canal() {
    let pasta = pasta_temporaria("claude");
    let bin = pasta.join("bin");
    fs::create_dir_all(&bin).unwrap();
    script(&bin, "claude", &format!("printf '%s\\0' \"$@\" > '{d}/args'\nenv > '{d}/env'", d = pasta.display()));
    let dev = pasta.join("Blueprint/claude-plugin-dev");
    // o binário de teste tem centenas de MB: a cópia desta compilação já "existe", e `plugin_e_exe` não copia
    let exe = std::env::current_exe().unwrap();
    let carimbo = fs::metadata(&exe).unwrap().modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    fs::create_dir_all(dev.join("bin")).unwrap();
    fs::write(dev.join(format!("bin/blueprint-{carimbo}.exe")), "").unwrap();
    let mut p = partida(&[], &pasta, "Execute the plan");
    p["claude"] = json!(true);
    let s = servidor(200, p);
    anotar_porta(&pasta, s.porta);
    let (dados, caminho) = (pasta.display().to_string(), format!("{}:/usr/bin:/bin", bin.display()));
    assert_eq!(terminar(&mut involucro(ID, &[("XDG_DATA_HOME", &dados), ("PATH", &caminho)]).spawn().unwrap()).code(), Some(0));
    let args = fs::read_to_string(pasta.join("args")).unwrap();
    let args: Vec<&str> = args.trim_end_matches('\0').split('\0').collect();
    assert_eq!(args.last(), Some(&"Execute the plan"), "{args:?}");
    assert!(args.contains(&"--plugin-dir") && args.contains(&"--agent"), "{args:?}");
    assert!(!args.iter().any(|a| a.contains("channels")), "sem canal: {args:?}");
    assert_eq!(s.de("hook/status").len(), 1, "a consulta de vínculo foi ao app achado pelo arquivo, não à porta 3000");
    let env = fs::read_to_string(pasta.join("env")).unwrap();
    assert!(env.lines().any(|l| l == "BLUEPRINT_MCP_PERFIL=executor") && !env.lines().any(|l| l.starts_with("BLUEPRINT_CHANNEL=")), "{env}");
    let plugin = if dev.join("plugins").exists() { dev } else { pasta.join("Blueprint/claude-plugin") };
    let hooks = fs::read_to_string(plugin.join("plugins/blueprint/hooks/hooks.json")).unwrap();
    assert!(hooks.contains(&format!("{}/api/mcp/hook/parada", s.url())), "os hooks levam a porta real: {hooks}");
    let _ = fs::remove_dir_all(pasta);
}
