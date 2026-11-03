use crate::contexto::Contexto;
use crate::dados::{migracoes, Dados};
use rusqlite::Connection;
use serde_json::{json, Value};

#[test]
fn cria_o_banco_do_zero_pelas_migracoes() {
    let mut conn = Connection::open_in_memory().unwrap();
    let aplicadas = migracoes::aplicar(&mut conn).unwrap();
    assert_eq!(aplicadas, vec!["001_inicial", "002_vocabulario", "003_pastas", "004_linha_do_tempo", "005_acesso", "006_historico", "007_tempos_planos", "008_fila_mcp", "009_planejar", "010_sem_datas"]);
    assert_eq!(
        migracoes::tabelas(&conn).unwrap(),
        vec!["acessos", "acoes", "atividade", "blueprints", "chaves_mcp", "checkpoints", "comentarios", "convites", "execucoes", "fila_mcp", "fila_mcp_contadores", "historico_campos", "linha_do_tempo", "membros", "migracoes", "orquestrador", "pastas", "planejar_config", "plano_figuras", "plano_revisoes", "plano_versoes", "planos", "prototipos", "recados", "sessoes", "workspaces"]
    );
}

#[test]
fn aplicar_de_novo_nao_faz_nada() {
    let mut conn = Connection::open_in_memory().unwrap();
    migracoes::aplicar(&mut conn).unwrap();
    assert!(migracoes::aplicar(&mut conn).unwrap().is_empty());
}

#[test]
fn toda_linha_tem_revisao_e_atualizado_em() {
    let d = Dados::em_memoria().unwrap();
    for tabela in ["workspaces", "blueprints", "acoes", "planos", "checkpoints", "comentarios", "prototipos"] {
        let colunas: Vec<String> = d
            .conexao()
            .prepare(&format!("SELECT name FROM pragma_table_info('{tabela}')"))
            .unwrap()
            .query_map([], |l| l.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        for esperada in ["id", "ordem", "revisao", "atualizado_em"] {
            assert!(colunas.iter().any(|c| c == esperada), "{tabela} sem {esperada}");
        }
    }
}

#[test]
fn o_banco_em_arquivo_usa_wal() {
    let caminho = std::env::temp_dir().join(format!("blueprint-wal-{}.db", std::process::id()));
    let d = Dados::abrir(&caminho).unwrap();
    let modo: String = d.conexao().query_row("PRAGMA journal_mode", [], |l| l.get(0)).unwrap();
    assert_eq!(modo, "wal");
    drop(d);
    for sufixo in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{sufixo}", caminho.display()));
    }
}

/// Um banco parado na 009, com datas em todo lugar, como o de quem atualiza o app.
fn banco_na_009() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE migracoes (nome TEXT PRIMARY KEY, aplicada_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')))").unwrap();
    let anteriores = [
        ("001_inicial", include_str!("../migracoes/001_inicial.sql")),
        ("002_vocabulario", include_str!("../migracoes/002_vocabulario.sql")),
        ("003_pastas", include_str!("../migracoes/003_pastas.sql")),
        ("004_linha_do_tempo", include_str!("../migracoes/004_linha_do_tempo.sql")),
        ("005_acesso", include_str!("../migracoes/005_acesso.sql")),
        ("006_historico", include_str!("../migracoes/006_historico.sql")),
        ("007_tempos_planos", include_str!("../migracoes/007_tempos_planos.sql")),
        ("008_fila_mcp", include_str!("../migracoes/008_fila_mcp.sql")),
        ("009_planejar", include_str!("../migracoes/009_planejar.sql")),
    ];
    for (nome, sql) in anteriores {
        conn.execute_batch(sql).unwrap();
        conn.execute("INSERT INTO migracoes (nome) VALUES (?1)", [nome]).unwrap();
    }
    conn.execute_batch(
        r#"
        INSERT INTO workspaces (id, ordem) VALUES ('w', 0);
        INSERT INTO blueprints (id, workspace_id, ordem, extra) VALUES ('b', 'w', 0, '{"decisoes":[{"id":"d","titulo":"T","data":"2026-01-01"}]}');
        INSERT INTO acoes (id, blueprint_id, ordem, nome, inicio, fim, extra) VALUES ('a', 'b', 0, 'A', '2026-01-01', '2026-01-02', '{"semData":true,"projetoId":"p"}');
        INSERT INTO planos (id, acao_id, ordem, texto, agente, extra) VALUES ('q', 'a', 0, 'Q', '{"estado":"concluido","inicioEm":"2026-01-01T10:00:00Z","feitoEm":"2026-01-02T10:00:00Z"}', '{"horas":2,"area":"x"}');
        INSERT INTO checkpoints (id, acao_id, ordem, nome, data, feito_em) VALUES ('c', 'a', 0, 'C', '2026-01-02', '2026-01-02T10:00:00Z');
        INSERT INTO tempos_planos (plano_id, inicio_em, fim_em) VALUES ('q', '2026-01-01T10:00:00Z', '2026-01-02T10:00:00Z');
        INSERT INTO plano_versoes (blueprint_id, numero, workspace_id, escrita_id, origem, situacao, plano, atualizada_em, criada_em) VALUES ('b', 1, 'w', 'e', 'escrita', 'fechada',
            '{"projetos":[{"acoes":[{"id":"1","nome":"Um","de":"2026-01-01","ate":"2026-01-02","checkpoints":[{"id":"c","texto":"K","data":"2026-01-02"}]}]}],"decisoes":[{"id":"d","texto":"T","data":"2026-01-01"}]}', 1, 1);
        INSERT INTO fila_mcp (id, item, entrada) VALUES (1,
            '{"proposta":{"acoes":[{"nome":"N","inicio":"2026-01-01","fim":"2026-01-02","planos":[{"texto":"P","horas":2,"de":"fa"}],"checkpoints":[{"nome":"K","data":"2026-01-02"}]}]}}',
            '{"acoes":[{"nome":"N","inicio":"2026-01-01","fim":"2026-01-02"}]}');
        INSERT INTO historico_campos (versao, em, membro_id, workspace_id, tipo, item_id, campo, operacao) VALUES (1, 'x', 'm', 'w', 'acao', 'a', 'inicio', 'alterar'), (2, 'x', 'm', 'w', 'acao', 'a', 'nome', 'alterar');
        "#,
    )
    .unwrap();
    conn
}

#[test]
fn a_010_apaga_as_datas_e_deixa_o_resto() {
    let mut conn = banco_na_009();
    assert_eq!(migracoes::aplicar(&mut conn).unwrap(), vec!["010_sem_datas"]);
    let colunas = |tabela: &str| -> Vec<String> { conn.prepare(&format!("SELECT name FROM pragma_table_info('{tabela}')")).unwrap().query_map([], |l| l.get(0)).unwrap().collect::<Result<_, _>>().unwrap() };
    let (acoes, checkpoints) = (colunas("acoes"), colunas("checkpoints"));
    assert!(!acoes.iter().any(|c| c == "inicio" || c == "fim"), "{acoes:?}");
    assert!(!checkpoints.iter().any(|c| c == "data" || c == "feito_em"), "{checkpoints:?}");
    assert!(!migracoes::tabelas(&conn).unwrap().iter().any(|t| t == "tempos_planos"));

    let texto = |sql: &str| -> String { conn.query_row(sql, [], |l| l.get(0)).unwrap() };
    let json_de = |sql: &str| -> Value { serde_json::from_str(&texto(sql)).unwrap() };
    assert_eq!(texto("SELECT nome FROM acoes"), "A", "o resto da linha fica");
    assert_eq!(json_de("SELECT extra FROM acoes"), json!({"projetoId": "p"}));
    assert_eq!(json_de("SELECT extra FROM planos"), json!({"area": "x"}));
    assert_eq!(json_de("SELECT agente FROM planos"), json!({"estado": "concluido"}));
    assert_eq!(json_de("SELECT extra FROM blueprints"), json!({"decisoes": [{"id": "d", "titulo": "T"}]}));
    assert_eq!(
        json_de("SELECT plano FROM plano_versoes"),
        json!({"projetos": [{"acoes": [{"id": "1", "nome": "Um", "checkpoints": [{"id": "c", "texto": "K"}]}]}], "decisoes": [{"id": "d", "texto": "T"}]})
    );
    assert_eq!(
        json_de("SELECT item FROM fila_mcp"),
        json!({"proposta": {"acoes": [{"nome": "N", "planos": [{"texto": "P", "de": "fa"}], "checkpoints": [{"nome": "K"}]}]}}),
        "o `de` do plano é a origem de um movimento, não uma data"
    );
    assert_eq!(json_de("SELECT entrada FROM fila_mcp"), json!({"acoes": [{"nome": "N"}]}));
    assert_eq!(texto("SELECT group_concat(campo) FROM historico_campos"), "nome");
}

#[test]
fn json_antigo_nao_traz_as_datas_de_volta() {
    let (mut d, ctx) = (Dados::em_memoria().unwrap(), Contexto::local());
    let antigo = json!({"blueprints": [{"id": "b", "nome": "B", "acoes": [{"id": "a", "nome": "A", "inicio": "2026-01-01", "fim": "2026-01-02", "semData": true, "status": "planejado",
        "planos": [{"id": "q", "texto": "Q", "horas": 2, "feitoEm": "2026-01-02T10:00:00Z", "agente": {"estado": "concluido", "inicioEm": "2026-01-01T10:00:00Z", "feitoEm": "2026-01-02T10:00:00Z"}}],
        "checkpoints": [{"id": "c", "nome": "C", "data": "2026-01-02", "feitoEm": "2026-01-02T10:00:00Z", "feito": true}]}]}]});
    d.gravar_tudo(&ctx, &antigo).unwrap();
    let tudo = d.ler_tudo(&ctx).unwrap().clone();
    let a = &tudo["blueprints"][0]["acoes"][0];
    assert_eq!((a["nome"].as_str(), a["status"].as_str()), (Some("A"), Some("planejado")));
    assert!(["inicio", "fim", "semData"].iter().all(|c| a.get(*c).is_none()), "{a}");
    assert_eq!(a["planos"][0], json!({"id": "q", "texto": "Q", "agente": {"estado": "concluido"}}));
    assert_eq!(a["checkpoints"][0], json!({"id": "c", "nome": "C", "feito": true}));
}
