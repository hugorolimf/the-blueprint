use super::{conteudo, figura};
use crate::apresentacao::editar::{aplicar, editar_registro, OPERACOES_MAX};
use crate::idioma::{com, Idioma};
use serde_json::{json, Value};

fn base() -> (Value, Value) {
    (conteudo(), json!([figura("fichas")]))
}

fn registro() -> Value {
    json!({"id": "0123456789abcdef", "workspaceId": "sm", "blueprintId": "p1", "titulo": "Atendimento de usuários", "publico": "cliente", "nivel": "padrao", "em": "2026-10-26T10:00:00Z", "conteudo": conteudo(), "figuras": [figura("fichas")]})
}

fn erro(c: &mut Value, f: &mut Value, ops: Value) -> (u16, String) {
    let e = aplicar(c, f, &ops).unwrap_err();
    (e.status, e.codigo.to_string())
}

#[test]
fn definir_troca_um_campo_cria_o_que_falta_e_recusa_caminho_torto() {
    let (mut c, mut f) = base();
    aplicar(&mut c, &mut f, &json!([{"op": "definir", "caminho": "etapas.0.titulo", "valor": "Cadastro e base"}, {"op": "definir", "caminho": "subtitulo", "valor": "Plano"}, {"op": "definir", "caminho": "resumo.0", "valor": "Novo"}])).unwrap();
    assert_eq!((c["etapas"][0]["titulo"].clone(), c["subtitulo"].clone(), c["resumo"][0].clone()), (json!("Cadastro e base"), json!("Plano"), json!("Novo")));
    for caminho in ["etapas.9.titulo", "naoexiste.x", "etapas.0.titulo.x", "etapas..titulo", "resumo.7"] {
        assert_eq!(erro(&mut c, &mut f, json!([{"op": "definir", "caminho": caminho, "valor": 1}])), (400, "caminho-invalido".into()), "{caminho:?}");
    }
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "definir", "caminho": "titulo"}])).1, "operacao-invalida", "sem valor");
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "definir", "caminho": "", "valor": 1}])).1, "operacao-invalida", "caminho vazio");
}

#[test]
fn inserir_remover_e_mover_em_lista() {
    let (mut c, mut f) = base();
    aplicar(&mut c, &mut f, &json!([
        {"op": "inserir", "caminho": "resumo", "valor": "segunda"},
        {"op": "inserir", "caminho": "resumo", "indice": 0, "valor": "zero"},
        {"op": "inserir", "caminho": "riscos", "valor": {"titulo": "Prazo", "texto": "Depende da Meta."}},
    ])).unwrap();
    assert_eq!((c["resumo"].clone(), c["riscos"].clone()), (json!(["zero", "Você recebe o atendimento no WhatsApp.", "segunda"]), json!([{"titulo": "Prazo", "texto": "Depende da Meta."}])), "uma lista que não existe nasce vazia");
    aplicar(&mut c, &mut f, &json!([{"op": "mover", "caminho": "resumo.2", "para": 0}, {"op": "remover", "caminho": "resumo.1"}, {"op": "remover", "caminho": "riscos.0.texto"}])).unwrap();
    assert_eq!((c["resumo"].clone(), c["riscos"][0].clone()), (json!(["segunda", "Você recebe o atendimento no WhatsApp."]), json!({"titulo": "Prazo"})));
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "inserir", "caminho": "resumo", "indice": 9, "valor": "x"}])).1, "operacao-invalida");
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "mover", "caminho": "resumo.0", "para": 5}])).1, "operacao-invalida");
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "mover", "caminho": "titulo", "para": 0}])).1, "caminho-invalido", "só se move dentro de uma lista");
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "remover", "caminho": "resumo.5"}])).1, "caminho-invalido");
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "remover", "caminho": "nada"}])).1, "caminho-invalido");
}

#[test]
fn figura_trecho_exige_exatamente_uma_ocorrencia() {
    let (mut c, mut f) = base();
    aplicar(&mut c, &mut f, &json!([{"op": "figura_trecho", "figura": "fichas", "de": "<svg></svg>", "para": "<svg id=\"a\"></svg>"}])).unwrap();
    assert!(f[0]["codigo"].as_str().unwrap().contains("<svg id=\"a\"></svg>"));
    let antes = f.clone();
    let e = aplicar(&mut c, &mut f, &json!([{"op": "figura_trecho", "figura": "fichas", "de": "nao-existe", "para": "x"}])).unwrap_err();
    assert_eq!((e.status, e.codigo, e.params.unwrap()["ocorrencias"].clone()), (422, "trecho-ausente", json!(0)));
    let e = aplicar(&mut c, &mut f, &json!([{"op": "figura_trecho", "figura": "fichas", "de": "svg", "para": "x"}])).unwrap_err();
    assert_eq!((e.status, e.codigo), (422, "trecho-ambiguo"), "nunca chuta qual trocar");
    assert!(e.params.unwrap()["ocorrencias"].as_u64().unwrap() > 1);
    assert_eq!(f, antes, "uma figura recusada não muda");
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "figura_trecho", "figura": "nada", "de": "a", "para": "b"}])), (404, "figura-nao-encontrada".into()));
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "figura_trecho", "figura": "fichas", "de": "", "para": "b"}])).1, "operacao-invalida");
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "figura_trecho", "figura": "fichas", "de": "svg"}])).1, "operacao-invalida", "sem para");
}

#[test]
fn figura_definir_e_figura_remover() {
    let (mut c, mut f) = base();
    f.as_array_mut().unwrap().push(figura("outra"));
    aplicar(&mut c, &mut f, &json!([{"op": "figura_definir", "figura": "fichas", "campo": "legenda", "valor": "Nova legenda"}, {"op": "figura_definir", "figura": "fichas", "campo": "secao", "valor": "visao"}, {"op": "figura_remover", "figura": "outra"}])).unwrap();
    assert_eq!((f[0]["legenda"].clone(), f[0]["secao"].clone(), f.as_array().unwrap().len()), (json!("Nova legenda"), json!("visao"), 1));
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "figura_definir", "figura": "fichas", "campo": "codigo", "valor": "x"}])).1, "operacao-invalida", "o código inteiro não se troca por aqui");
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "figura_remover", "figura": "outra"}])).1, "figura-nao-encontrada");
}

#[test]
fn operacao_desconhecida_lista_torta_e_limite() {
    let (mut c, mut f) = base();
    assert_eq!(erro(&mut c, &mut f, json!([{"op": "apagar_tudo"}])).1, "operacao-invalida");
    assert_eq!(erro(&mut c, &mut f, json!([5])).1, "operacao-invalida");
    assert_eq!(erro(&mut c, &mut f, json!({"op": "definir"})).1, "operacao-invalida", "operacoes é uma lista");
    let muitas = Value::Array((0..=OPERACOES_MAX).map(|_| json!({"op": "definir", "caminho": "titulo", "valor": "x"})).collect());
    assert_eq!(erro(&mut c, &mut f, muitas).1, "operacao-invalida");
    // o erro diz qual operação falhou
    let e = aplicar(&mut c, &mut f, &json!([{"op": "definir", "caminho": "titulo", "valor": "ok"}, {"op": "remover", "caminho": "nada"}])).unwrap_err();
    assert_eq!(e.params.as_ref().unwrap()["operacao"], json!(1));
    let msg = com(Idioma::En, || aplicar(&mut conteudo(), &mut json!([]), &json!([{"op": "x"}, {"op": "y"}])).unwrap_err().mensagem);
    assert!(msg.starts_with("operation 0: unknown operation"), "{msg}");
}

#[test]
fn editar_registro_e_tudo_ou_nada_e_valida_o_resultado() {
    let r = registro();
    let (novo, _avisos) = editar_registro(&r, &json!({"operacoes": [{"op": "definir", "caminho": "titulo", "valor": "Plano novo"}, {"op": "definir", "caminho": "nivel", "valor": "rapido"}], "autor": "agente"}), "2026-10-27T01:00:00Z").unwrap();
    assert_eq!((novo["titulo"].clone(), novo["nivel"].clone(), novo["editadoEm"].clone(), novo["editadoPor"].clone(), novo["em"].clone()), (json!("Plano novo"), json!("rapido"), json!("2026-10-27T01:00:00Z"), json!("agente"), json!("2026-10-26T10:00:00Z")), "os metadados seguem o conteúdo e `em` fica");
    assert_eq!(novo["conteudo"]["titulo"], json!("Plano novo"));
    // a segunda operação falha: a primeira não vale (o registro de origem nem muda)
    let e = editar_registro(&r, &json!({"operacoes": [{"op": "definir", "caminho": "titulo", "valor": "X"}, {"op": "remover", "caminho": "nada"}]}), "t").unwrap_err();
    assert_eq!(e.codigo, "caminho-invalido");
    assert_eq!(r, registro());
    // sem autor, é o usuário
    assert_eq!(editar_registro(&r, &json!({"operacoes": [{"op": "definir", "caminho": "titulo", "valor": "Y"}]}), "t").unwrap().0["editadoPor"], json!("usuario"));
    // os avisos do validar voltam (a figura que a etapa usa foi removida)
    let (_, avisos) = editar_registro(&r, &json!({"operacoes": [{"op": "figura_remover", "figura": "fichas"}]}), "t").unwrap();
    assert!(avisos.iter().any(|a| a.contains("Stage \"e1\" points to figure \"fichas\"")), "{avisos:?}");
}

#[test]
fn editar_registro_recusa_o_que_o_validar_recusa_e_o_que_nao_e_edicao() {
    let r = registro();
    let cod = |corpo: Value| editar_registro(&r, &corpo, "t").unwrap_err().codigo;
    assert_eq!(cod(json!({"operacoes": [{"op": "definir", "caminho": "publico", "valor": "todos"}]})), "publico-invalido");
    assert_eq!(cod(json!({"operacoes": [{"op": "definir", "caminho": "titulo", "valor": "  "}]})), "falta-titulo");
    assert_eq!(cod(json!({"operacoes": [{"op": "definir", "caminho": "interno", "valor": {"acoes": []}}]})), "interno-no-cliente");
    assert_eq!(cod(json!({"operacoes": [{"op": "figura_definir", "figura": "fichas", "campo": "id", "valor": "Id Ruim"}]})), "figura-id-invalido");
    assert_eq!(cod(json!({})), "nada-a-editar");
    assert_eq!(cod(json!({"operacoes": null, "conteudo": null})), "nada-a-editar");
    assert_eq!(cod(json!({"operacoes": [], "autor": "robo"})), "autor-invalido");
}

#[test]
fn conteudo_e_figuras_inteiros_substituem_a_parte_e_o_resto_fica() {
    let r = registro();
    let (so_conteudo, _) = editar_registro(&r, &json!({"conteudo": {"versao": 1, "publico": "interno", "nivel": "completo", "titulo": "Outro"}}), "t").unwrap();
    assert_eq!((so_conteudo["titulo"].clone(), so_conteudo["publico"].clone(), so_conteudo["figuras"].clone()), (json!("Outro"), json!("interno"), r["figuras"].clone()), "só o conteúdo veio: as figuras ficam");
    let (so_figuras, _) = editar_registro(&r, &json!({"figuras": []}), "t").unwrap();
    assert_eq!((so_figuras["conteudo"].clone(), so_figuras["figuras"].clone()), (r["conteudo"].clone(), json!([])));
}
