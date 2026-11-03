//! Testes de plano::validar (seção 3.2 do contrato).
use super::formato::{Detalhe, FiguraInfo, PlanoDoc, FORMATO};
use super::pacote;
use super::validar::{achar_ciclo, avisos, validar, validar_peca};
use serde_json::{json, Value};
use std::cell::Cell;
use std::path::Path;

fn sem_figura(_: &str, _: &Detalhe) -> Vec<String> {
    Vec::new()
}

/// Um documento de uma ação `1`, com os planos dados.
fn doc_de(planos: Value) -> PlanoDoc {
    doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "1", "nome": "Um", "planos": planos}]}]}))
}

fn doc(v: Value) -> PlanoDoc {
    PlanoDoc::de_json(&v).unwrap()
}

fn erros_de(p: &PlanoDoc) -> Vec<String> {
    validar(p, None, &sem_figura)
}

fn tem(erros: &[String], trechos: &[&str]) -> bool {
    erros.iter().any(|e| trechos.iter().all(|t| e.contains(t)))
}

#[test]
fn aceita_plano_so_com_texto() {
    let p = doc_de(json!(["Fazer", {"id": "1.9", "texto": "Outro"}]));
    assert_eq!(erros_de(&p), Vec::<String>::new());
    assert!(tem(&avisos(&p), &["plan 1.1", "Write the id"]), "o id gerado avisa: {:?}", avisos(&p));
    assert!(!tem(&avisos(&p), &["plan 1.9"]));
}

#[test]
fn dependencia_inexistente() {
    let p = doc_de(json!([{"id": "a", "texto": "t", "depende_de": ["z"]}]));
    assert!(tem(&erros_de(&p), &["does not exist", "z"]), "{:?}", erros_de(&p));
    assert_eq!(validar_peca(&p, &p.projetos[0].acoes[0]), Vec::<String>::new(), "ao chegar a dependência ainda pode vir em outra peça");
}

#[test]
fn id_repetido() {
    let p = doc_de(json!([{"id": "a", "texto": "t"}, {"id": "a", "texto": "u"}]));
    assert!(tem(&erros_de(&p), &["plan a", "repeated id"]), "{:?}", erros_de(&p));
    let outra = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "1", "nome": "Um"}, {"id": "1", "nome": "Dois"}]}]}));
    assert!(tem(&erros_de(&outra), &["action 1", "repeated id"]));
    let entre_acoes = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "1", "nome": "Um", "planos": [{"id": "x", "texto": "t"}]}, {"id": "2", "nome": "Dois", "planos": [{"id": "x", "texto": "u"}]}]}]}));
    assert!(tem(&erros_de(&entre_acoes), &["plan x", "repeated id"]), "o plano é o trabalho de um agente: id único no blueprint");
    assert!(tem(&validar_peca(&entre_acoes, &entre_acoes.projetos[0].acoes[1]), &["plan x", "repeated id"]), "a peça já bate com o que existe");
    let pecas = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "1", "nome": "Um", "detalhes": [{"id": "d"}, {"id": "d"}]}]}], "decisoes": [{"id": "x", "texto": "t"}, {"id": "x", "texto": "u"}], "perguntas": [{"id": "y", "texto": "t"}, {"id": "y", "texto": "u"}]}));
    let e = erros_de(&pecas);
    assert!(tem(&e, &["detail d", "repeated id"]) && tem(&e, &["decision x", "repeated id"]) && tem(&e, &["question y", "repeated id"]), "{e:?}");
}

#[test]
fn ciclo() {
    let p = doc_de(json!([{"id": "a", "texto": "t", "depende_de": ["b"]}, {"id": "b", "texto": "u", "depende_de": ["a"]}]));
    assert!(tem(&erros_de(&p), &["cycle", "a -> b -> a"]), "{:?}", erros_de(&p));
    assert!(tem(&validar_peca(&p, &p.projetos[0].acoes[0]), &["cycle"]), "ciclo entre os conhecidos já é erro ao chegar");
    let planos: Vec<_> = p.projetos[0].acoes[0].planos.iter().collect();
    assert_eq!(achar_ciclo(&planos), Some(vec!["a".to_string(), "b".to_string(), "a".to_string()]));
    let sozinho = doc_de(json!([{"id": "a", "texto": "t", "depende_de": ["a"]}]));
    assert!(tem(&erros_de(&sozinho), &["a -> a"]));
    let losango = doc_de(json!([{"id": "a", "texto": "t", "depende_de": ["b", "c"]}, {"id": "b", "texto": "t", "depende_de": ["d"]}, {"id": "c", "texto": "t", "depende_de": ["d"]}, {"id": "d", "texto": "t"}]));
    assert_eq!(erros_de(&losango), Vec::<String>::new(), "dois caminhos para o mesmo plano não são ciclo");
    // uma cadeia longa não estoura a pilha
    let cadeia: Vec<Value> = (0..50_000).map(|i| json!({"id": format!("p{i}"), "texto": "t", "depende_de": [format!("p{}", i + 1)]})).collect();
    let longa = doc_de(json!(cadeia));
    let planos: Vec<_> = longa.projetos[0].acoes[0].planos.iter().collect();
    assert_eq!(achar_ciclo(&planos), None);
}

#[test]
fn formato_futuro_recusado() {
    let v = json!({"formato": "blueprint.plano/2", "blueprint": "B", "projetos": [{"acoes": []}]});
    assert!(PlanoDoc::de_json(&v).unwrap_err().iter().any(|e| e.contains("blueprint.plano/2") && e.contains(FORMATO)));
    let mut p = doc_de(json!([]));
    p.formato = "blueprint.plano/2".into();
    assert!(tem(&erros_de(&p), &["formato", FORMATO]), "a struct montada à mão também não passa");
}

#[test]
fn ref_com_barra_recusado() {
    let p = doc_de(json!([{"id": "a/b", "texto": "t"}]));
    assert!(tem(&erros_de(&p), &["plan a/b", "id must start"]), "{:?}", erros_de(&p));
    let acao = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "1/2", "nome": "Um"}]}]}));
    assert!(tem(&erros_de(&acao), &["action 1/2", "id must start"]));
    let pecas = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "1", "nome": "Um", "detalhes": [{"id": "d/e"}]}]}], "decisoes": [{"id": "x/y", "texto": "t"}], "perguntas": [{"id": "../z", "texto": "t"}]}));
    let e = erros_de(&pecas);
    assert!(tem(&e, &["detail 1-d/e"]) && tem(&e, &["decision x/y"]) && tem(&e, &["question ../z"]), "{e:?}");
}

#[test]
fn ref_com_hifen_inicial_recusado() {
    let p = doc_de(json!([{"id": "-force", "texto": "t"}]));
    assert!(tem(&erros_de(&p), &["plan -force", "id must start"]), "um ramo em forma de opção do git: {:?}", erros_de(&p));
}

#[test]
fn area_com_ponto_ponto_recusada() {
    let p = doc_de(json!([{"id": "a", "texto": "t", "area": "../fora"}, {"id": "b", "texto": "t", "area": "app/../fora"}, {"id": "c", "texto": "t", "area": "app/barra"}, {"id": "d", "texto": "t", "area": ""}]));
    let e = erros_de(&p);
    assert!(tem(&e, &["plan a", "area"]) && tem(&e, &["plan b", "area"]), "{e:?}");
    assert!(!tem(&e, &["plan c"]) && !tem(&e, &["plan d"]), "área boa, e `\"\"` é como se apaga o campo: {e:?}");
}

#[test]
fn ids_so_por_caixa_recusados() {
    let p = doc_de(json!([{"id": "Abc", "texto": "t"}, {"id": "abc", "texto": "u"}]));
    assert!(tem(&erros_de(&p), &["plan abc", "only by case", "Abc"]), "{:?}", erros_de(&p));
    let acoes = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "A", "nome": "Um"}]}, {"acoes": [{"id": "a", "nome": "Dois"}]}]}));
    assert!(tem(&erros_de(&acoes), &["action a", "only by case"]), "ações de projetos diferentes também");
}

#[test]
fn projeto_vazio_so_avisa() {
    let p = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"nome": "Depois", "acoes": []}]}));
    assert_eq!(erros_de(&p), Vec::<String>::new());
    assert!(tem(&avisos(&p), &["Depois", "no action yet"]));
    assert!(PlanoDoc::de_json(&json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"nome": "Depois"}]})).is_err(), "`acoes` ausente é erro");
    let sem_projeto = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": []}));
    assert!(tem(&erros_de(&sem_projeto), &["projetos: at least one"]));
}

#[test]
fn acao_nao_tocada_com_campos_ruins_passa() {
    let p = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [
        {"id": "velha", "nome": "Antiga", "planos": [{"id": "v1", "texto": "t", "area": "../x"}], "tabelas": [{"linhas": []}]},
        {"id": "nova", "nome": "Nova", "planos": [{"id": "n1", "texto": "t", "depende_de": ["z"]}]}
    ]}]}));
    let tocadas = ["nova".to_string()];
    let e = validar(&p, Some(&tocadas), &sem_figura);
    assert_eq!(e.len(), 1, "só a dependência, que vale para tudo: {e:?}");
    assert!(tem(&e, &["n1", "does not exist"]));
    let velha = ["velha".to_string()];
    assert!(tem(&validar(&p, Some(&velha), &sem_figura), &["action velha", "colunas is required"]), "a mesma ação, tocada, passa pelas regras de campo");
    assert!(tem(&erros_de(&p), &["table 1: colunas is required"]), "None = todas as ações");
    assert_eq!(validar(&doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": []})), Some(&[]), &sem_figura), Vec::<String>::new(), "escrita num blueprint ainda sem ação");
}

#[test]
fn figura_da_arvore_nao_pede_linha() {
    let mut p = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "1", "nome": "Um", "detalhes": [
        {"id": "antiga", "figura": "figuras/a.svg"}, {"id": "nova", "figura": "figuras/b.svg"}, {"id": "sem", "titulo": "Só título"}
    ]}]}]}));
    p.projetos[0].acoes[0].detalhes[0].figura_info = Some(FiguraInfo { hash: "h".into(), tipo: "svg".into(), largura: Some(10), altura: Some(10) });
    let chamadas = Cell::new(0);
    let faltando = |acao: &str, d: &Detalhe| {
        chamadas.set(chamadas.get() + 1);
        vec![format!("detail {acao}-{}: figure not found", d.id)]
    };
    let e = validar(&p, None, &faltando);
    assert_eq!(chamadas.get(), 1, "só a figura sem `figura_info`: {e:?}");
    assert_eq!(e, vec!["detail 1-nova: figure not found".to_string()]);
    let tocadas: [String; 0] = [];
    assert_eq!(validar(&p, Some(&tocadas), &faltando), Vec::<String>::new(), "ação não tocada nem chama");
}

#[test]
fn exemplos_passam() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plano/fixtures");
    for nome in ["monitor", "reforma", "biblioteca"] {
        let pacote = pacote::ler_pasta(&raiz.join(nome)).unwrap_or_else(|e| panic!("{nome}: {e:?}"));
        let p = PlanoDoc::de_json(&pacote.plano).unwrap_or_else(|e| panic!("{nome}: {e:?}"));
        let figura = |acao: &str, d: &Detalhe| {
            let caminho = d.figura.as_deref().unwrap_or("");
            if pacote.figuras.iter().any(|(c, _)| c == caminho) { Vec::new() } else { vec![format!("detail {acao}-{}: figure {caminho} was not read", d.id)] }
        };
        assert_eq!(validar(&p, None, &figura), Vec::<String>::new(), "{nome}");
        for a in p.projetos.iter().flat_map(|pr| &pr.acoes) {
            assert_eq!(validar_peca(&p, a), Vec::<String>::new(), "{nome}, ação {}", a.id);
        }
    }
}

/// As mesmas regras sobre os três exemplos, sem depender de `pacote`: a figura citada só precisa existir na pasta.
#[test]
fn exemplos_validam_sem_pacote() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plano/fixtures");
    for nome in ["monitor", "reforma", "biblioteca"] {
        let pasta = raiz.join(nome);
        let v: Value = serde_json::from_str(&std::fs::read_to_string(pasta.join("plano.json")).unwrap()).unwrap();
        let p = PlanoDoc::de_json(&v).unwrap_or_else(|e| panic!("{nome}: {e:?}"));
        let figura = |_: &str, d: &Detalhe| if pasta.join(d.figura.as_deref().unwrap_or("")).is_file() { Vec::new() } else { vec![format!("{}: not found", d.id)] };
        assert_eq!(validar(&p, None, &figura), Vec::<String>::new(), "{nome}");
    }
}

#[test]
fn peca_sem_nome_usa_o_do_plano_e_substitui_a_antiga() {
    let p = doc_de(json!([{"id": "a", "texto": "t"}]));
    let mut peca = p.projetos[0].acoes[0].clone();
    peca.nome.clear();
    assert_eq!(validar_peca(&p, &peca), Vec::<String>::new(), "`plano/escrever` pode omitir o nome de uma ação que já tem");
    let nova = super::formato::AcaoDoc { id: "2".into(), ..Default::default() };
    assert!(tem(&validar_peca(&p, &nova), &["action 2", "nome"]));
}

/// Uma ação `1` com o plano `1.1` e a peça dada no campo dado.
fn com_peca(campo: &str, peca: Value) -> PlanoDoc {
    let mut acao = json!({"id": "1", "nome": "Um", "planos": [{"id": "1.1", "texto": "T"}]});
    acao[campo] = json!([peca]);
    doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [acao]}]}))
}

#[test]
fn a_peca_so_aponta_para_um_plano_da_propria_acao() {
    assert!(erros_de(&com_peca("detalhes", json!({"id": "01", "plano": "1.1"}))).is_empty());
    assert!(erros_de(&com_peca("detalhes", json!({"id": "01", "plano": ""}))).is_empty(), "vazio tira a peça do plano");
    assert!(tem(&erros_de(&com_peca("detalhes", json!({"id": "01", "plano": "9"}))), &["detail 01", "the plan 9"]));
    assert!(tem(&erros_de(&com_peca("checkpoints", json!({"id": "c1", "texto": "K", "plano": "9"}))), &["checkpoint c1", "the plan 9"]));
    assert!(tem(&erros_de(&com_peca("tabelas", json!({"colunas": ["A"], "linhas": [], "plano": "9"}))), &["table 1", "the plan 9"]));
}

#[test]
fn decisao_e_pergunta_apontam_para_um_plano_das_suas_acoes() {
    let com = |campo: &str, peca: Value| {
        let mut v = json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "1", "nome": "Um", "planos": [{"id": "1.1", "texto": "T"}]}]}]});
        v[campo] = json!([peca]);
        doc(v)
    };
    assert!(erros_de(&com("decisoes", json!({"id": "d", "texto": "X", "acoes": ["1"], "plano": "1.1"}))).is_empty());
    assert!(tem(&erros_de(&com("decisoes", json!({"id": "d", "texto": "X", "plano": "1.1"}))), &["decision d", "the plan 1.1"]), "sem a ação em acoes, não aparece no plano");
    assert!(tem(&erros_de(&com("perguntas", json!({"id": "q", "texto": "X", "acoes": ["1"], "plano": "9"}))), &["question q", "the plan 9"]));
}

#[test]
fn o_numero_do_plano_segue_o_da_acao() {
    assert_eq!(numero_fora_da_acao("1", "1.1"), None);
    assert_eq!(numero_fora_da_acao("2.3", "2.3.1"), None);
    assert!(numero_fora_da_acao("1", "2.1").unwrap().contains("1.1"));
    assert!(numero_fora_da_acao("1", "11.1").is_some(), "11.1 começa com 1, mas não é da ação 1");
    assert!(numero_fora_da_acao("1", "1").is_some());
    assert_eq!(numero_fora_da_acao("velha", "v1"), None, "id de letras é de planos antigos");
}

#[test]
fn forma_desconhecida_e_entrega_longa_sao_erro() {
    let com = |campos: serde_json::Value| {
        let mut plano = json!({"id": "1.1", "texto": "T"});
        plano.as_object_mut().unwrap().extend(campos.as_object().unwrap().clone());
        let a = AcaoDoc::de_json(&json!({"id": "1", "nome": "Um", "planos": [plano]})).unwrap();
        let mut erros = Vec::new();
        regras_da_acao(&a, true, &mut erros);
        erros
    };
    assert!(com(json!({"forma": "tela", "entrega": "Seletor de modelo no cartão"})).is_empty());
    assert!(com(json!({"forma": ""})).is_empty(), "vazio apaga o campo");
    assert!(com(json!({"forma": "castelo"}))[0].contains("forma"));
    assert!(com(json!({"entrega": "x".repeat(81)}))[0].contains("entrega"));
}

#[test]
fn variante_fora_do_catalogo_so_avisa() {
    let detalhe = |v: &str| json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "1", "nome": "Um", "planos": [{"id": "1.1", "texto": "t"}], "detalhes": [{"id": "01", "plano": "1.1", "variante": v}]}]}]});
    let estranha = doc(detalhe("tablet"));
    assert_eq!(erros_de(&estranha), Vec::<String>::new());
    assert!(tem(&avisos(&estranha), &["detail 01", "\"tablet\"", "janela"]), "{:?}", avisos(&estranha));
    let conhecida = doc(detalhe("celular"));
    assert!(!tem(&avisos(&conhecida), &["variante"]), "{:?}", avisos(&conhecida));
}
