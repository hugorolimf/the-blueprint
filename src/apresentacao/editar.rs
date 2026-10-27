//! Edição pontual de uma apresentação (contrato em docs/interno/apresentacao-ia.md, "Manual editing" e "Agent edits"): operações pequenas sobre o conteúdo e as figuras, aplicadas
//! numa CÓPIA e validadas inteiras (`validar`) antes de gravar: ou tudo entra, ou nada. O agente e a tela de edição mandam as mesmas operações, então uma edição do usuário e uma do
//! agente em campos diferentes nunca se sobrescrevem. Tudo o que o agente lê aqui é inglês.
use super::validar::validar;
use crate::http::pedido::{falha, ErroHttp};
use serde_json::{json, Value};

pub const OPERACOES_MAX: usize = 200;

fn caminho_invalido(caminho: &str) -> ErroHttp {
    falha(400, crate::tr!("invalid path \"{caminho}\": use dotted keys and 0-based list indexes (etapas.2.titulo)", "caminho inválido \"{caminho}\": use chaves com ponto e índices de lista a partir de 0 (etapas.2.titulo)")).codigo("caminho-invalido").params(json!({"caminho": caminho}))
}

fn operacao_invalida(m: String) -> ErroHttp {
    falha(400, m).codigo("operacao-invalida")
}

fn passos(caminho: &str) -> Result<Vec<&str>, ErroHttp> {
    let p: Vec<&str> = caminho.split('.').collect();
    if caminho.is_empty() || p.iter().any(|s| s.is_empty()) {
        return Err(caminho_invalido(caminho));
    }
    Ok(p)
}

fn filho<'a>(v: &'a mut Value, passo: &str) -> Option<&'a mut Value> {
    match v {
        Value::Array(a) => passo.parse::<usize>().ok().and_then(|i| a.get_mut(i)),
        Value::Object(m) => m.get_mut(passo),
        _ => None,
    }
}

/// O valor em `passos` (a lista vazia é a própria raiz).
fn alcancar<'a>(raiz: &'a mut Value, passos: &[&str], caminho: &str) -> Result<&'a mut Value, ErroHttp> {
    let mut atual = raiz;
    for p in passos {
        atual = match filho(atual, p) {
            Some(x) => x,
            None => return Err(caminho_invalido(caminho)),
        };
    }
    Ok(atual)
}

fn texto<'a>(op: &'a Value, chave: &str) -> Result<&'a str, ErroHttp> {
    op.get(chave).and_then(Value::as_str).filter(|t| !t.is_empty()).ok_or_else(|| operacao_invalida(crate::tr!("this operation needs \"{chave}\" (text)", "esta operação precisa de \"{chave}\" (texto)")))
}

fn indice(op: &Value, chave: &str) -> Result<Option<usize>, ErroHttp> {
    match op.get(chave) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v.as_u64().map(|n| Some(n as usize)).ok_or_else(|| operacao_invalida(crate::tr!("\"{chave}\" must be a whole number from 0", "\"{chave}\" deve ser um número inteiro a partir de 0"))),
    }
}

fn definir(c: &mut Value, caminho: &str, valor: Value) -> Result<(), ErroHttp> {
    let p = passos(caminho)?;
    let (ultimo, pais) = p.split_last().expect("passos não é vazio");
    match alcancar(c, pais, caminho)? {
        Value::Array(a) => match ultimo.parse::<usize>().ok().filter(|i| *i < a.len()) {
            Some(i) => a[i] = valor,
            None => return Err(caminho_invalido(caminho)),
        },
        Value::Object(m) => {
            m.insert((*ultimo).to_string(), valor);
        }
        _ => return Err(caminho_invalido(caminho)),
    }
    Ok(())
}

fn inserir(c: &mut Value, caminho: &str, onde: Option<usize>, valor: Value) -> Result<(), ErroHttp> {
    let p = passos(caminho)?;
    let (ultimo, pais) = p.split_last().expect("passos não é vazio");
    let pai = alcancar(c, pais, caminho)?;
    if let Value::Object(m) = pai {
        m.entry((*ultimo).to_string()).or_insert_with(|| json!([])); // uma lista que ainda não existe nasce vazia
    }
    let Some(Value::Array(a)) = filho(pai, ultimo) else { return Err(caminho_invalido(caminho)) };
    let i = onde.unwrap_or(a.len());
    if i > a.len() {
        let n = a.len();
        return Err(operacao_invalida(crate::tr!("indice {i} is past the end of the list ({n} items)", "indice {i} passa do fim da lista ({n} itens)")));
    }
    a.insert(i, valor);
    Ok(())
}

fn remover(c: &mut Value, caminho: &str) -> Result<(), ErroHttp> {
    let p = passos(caminho)?;
    let (ultimo, pais) = p.split_last().expect("passos não é vazio");
    match alcancar(c, pais, caminho)? {
        Value::Array(a) => match ultimo.parse::<usize>().ok().filter(|i| *i < a.len()) {
            Some(i) => {
                a.remove(i);
            }
            None => return Err(caminho_invalido(caminho)),
        },
        Value::Object(m) => {
            m.remove(*ultimo).ok_or_else(|| caminho_invalido(caminho))?;
        }
        _ => return Err(caminho_invalido(caminho)),
    }
    Ok(())
}

fn mover(c: &mut Value, caminho: &str, para: usize) -> Result<(), ErroHttp> {
    let p = passos(caminho)?;
    let (ultimo, pais) = p.split_last().expect("passos não é vazio");
    let Value::Array(a) = alcancar(c, pais, caminho)? else { return Err(caminho_invalido(caminho)) };
    let de = ultimo.parse::<usize>().ok().filter(|i| *i < a.len()).ok_or_else(|| caminho_invalido(caminho))?;
    if para >= a.len() {
        let n = a.len();
        return Err(operacao_invalida(crate::tr!("\"para\" {para} is past the end of the list ({n} items)", "\"para\" {para} passa do fim da lista ({n} itens)")));
    }
    let item = a.remove(de);
    a.insert(para, item);
    Ok(())
}

fn figura<'a>(figuras: &'a mut Value, id: &str) -> Result<&'a mut Value, ErroHttp> {
    figuras.as_array_mut().and_then(|l| l.iter_mut().find(|f| f.get("id").and_then(Value::as_str) == Some(id))).ok_or_else(|| falha(404, crate::tr!("figure \"{id}\" does not exist in this presentation", "a figura \"{id}\" não existe nesta apresentação")).codigo("figura-nao-encontrada").params(json!({"id": id})))
}

fn figura_trecho(figuras: &mut Value, op: &Value) -> Result<(), ErroHttp> {
    let (id, de) = (texto(op, "figura")?, texto(op, "de")?);
    let para = op.get("para").and_then(Value::as_str).ok_or_else(|| operacao_invalida(crate::tr!("this operation needs \"para\" (text, may be empty)", "esta operação precisa de \"para\" (texto, pode ser vazio)")))?;
    let f = figura(figuras, id)?;
    let Some(codigo) = f.get("codigo").and_then(Value::as_str) else {
        return Err(falha(400, crate::tr!("figure \"{id}\" has no codigo to edit", "a figura \"{id}\" não tem codigo para editar")).codigo("codigo-invalido").params(json!({"id": id})));
    };
    // shortcut: conta ocorrências sem sobreposição; vale para trechos de código, onde se sobrepor a si mesmo não acontece
    let n = codigo.matches(de).count();
    if n != 1 {
        let (codigo_erro, mensagem) = if n == 0 {
            ("trecho-ausente", crate::tr!("the piece to replace was not found in the codigo of figure \"{id}\" (0 matches); copy it exactly from ler_apresentacao", "o trecho a trocar não está no codigo da figura \"{id}\" (0 ocorrências); copie-o exatamente de ler_apresentacao"))
        } else {
            ("trecho-ambiguo", crate::tr!("the piece to replace occurs {n} times in the codigo of figure \"{id}\"; make it longer so it matches exactly once", "o trecho a trocar aparece {n} vezes no codigo da figura \"{id}\"; aumente-o para casar uma vez só"))
        };
        return Err(falha(422, mensagem).codigo(codigo_erro).params(json!({"id": id, "ocorrencias": n})));
    }
    let novo = codigo.replacen(de, para, 1);
    f["codigo"] = Value::String(novo);
    Ok(())
}

fn figura_definir(figuras: &mut Value, op: &Value) -> Result<(), ErroHttp> {
    let (id, campo) = (texto(op, "figura")?, texto(op, "campo")?);
    if campo == "codigo" {
        return Err(operacao_invalida(crate::tr!("do not set codigo whole: use figura_trecho to change one piece of it", "não troque o codigo inteiro: use figura_trecho para mudar um trecho dele")));
    }
    let valor = op.get("valor").cloned().ok_or_else(|| operacao_invalida(crate::tr!("this operation needs \"valor\"", "esta operação precisa de \"valor\"")))?;
    figura(figuras, id)?[campo] = valor;
    Ok(())
}

fn figura_remover(figuras: &mut Value, op: &Value) -> Result<(), ErroHttp> {
    let id = texto(op, "figura")?;
    figura(figuras, id)?;
    figuras.as_array_mut().expect("figura() só acha em lista").retain(|f| f.get("id").and_then(Value::as_str) != Some(id));
    Ok(())
}

fn uma(conteudo: &mut Value, figuras: &mut Value, op: &Value) -> Result<(), ErroHttp> {
    let nome = op.get("op").and_then(Value::as_str).unwrap_or("");
    let valor = || op.get("valor").cloned().ok_or_else(|| operacao_invalida(crate::tr!("this operation needs \"valor\"", "esta operação precisa de \"valor\"")));
    match nome {
        "definir" => definir(conteudo, texto(op, "caminho")?, valor()?),
        "inserir" => inserir(conteudo, texto(op, "caminho")?, indice(op, "indice")?, valor()?),
        "remover" => remover(conteudo, texto(op, "caminho")?),
        "mover" => mover(conteudo, texto(op, "caminho")?, indice(op, "para")?.ok_or_else(|| operacao_invalida(crate::tr!("this operation needs \"para\" (a list index)", "esta operação precisa de \"para\" (um índice de lista)")))?),
        "figura_trecho" => figura_trecho(figuras, op),
        "figura_definir" => figura_definir(figuras, op),
        "figura_remover" => figura_remover(figuras, op),
        outra => Err(operacao_invalida(crate::tr!("unknown operation \"{outra}\" (definir, inserir, remover, mover, figura_trecho, figura_definir, figura_remover)", "operação desconhecida \"{outra}\" (definir, inserir, remover, mover, figura_trecho, figura_definir, figura_remover)"))),
    }
}

/// Aplica as operações em ordem; a primeira que falha desfaz tudo (quem chama só grava se voltar `Ok`).
pub fn aplicar(conteudo: &mut Value, figuras: &mut Value, operacoes: &Value) -> Result<(), ErroHttp> {
    let Some(lista) = operacoes.as_array() else {
        return Err(operacao_invalida(crate::tr!("operacoes must be a list", "operacoes deve ser uma lista")));
    };
    if lista.len() > OPERACOES_MAX {
        let n = OPERACOES_MAX;
        return Err(operacao_invalida(crate::tr!("at most {n} operations per call", "no máximo {n} operações por chamada")));
    }
    for (i, op) in lista.iter().enumerate() {
        uma(conteudo, figuras, op).map_err(|mut e| {
            let m = std::mem::take(&mut e.mensagem);
            e.mensagem = crate::tr!("operation {i}: {m}", "operação {i}: {m}");
            let mut p = e.params.take().unwrap_or_else(|| json!({}));
            p["operacao"] = json!(i);
            e.params = Some(p);
            e
        })?;
    }
    Ok(())
}

/// O registro depois de uma edição: `corpo` traz `operacoes` e/ou o `conteudo` e as `figuras` inteiros (que substituem a parte); o resultado passa pelo `validar` inteiro.
/// Devolve o registro novo (os metadados `titulo`, `publico` e `nivel` seguem o conteúdo; `editadoEm` e `editadoPor` marcam a edição) e os avisos.
pub fn editar_registro(registro: &Value, corpo: &Value, agora: &str) -> Result<(Value, Vec<String>), ErroHttp> {
    let autor = match corpo.get("autor").and_then(Value::as_str) {
        None | Some("usuario") => "usuario",
        Some("agente") => "agente",
        Some(_) => return Err(falha(400, crate::tr!("autor must be \"usuario\" or \"agente\"", "autor deve ser \"usuario\" ou \"agente\"")).codigo("autor-invalido")),
    };
    let tem = |k: &str| corpo.get(k).is_some_and(|v| !v.is_null());
    if !(tem("operacoes") || tem("conteudo") || tem("figuras")) {
        return Err(falha(400, crate::tr!("nothing to edit: send operacoes, conteudo or figuras", "nada a editar: envie operacoes, conteudo ou figuras")).codigo("nada-a-editar"));
    }
    let (mut conteudo, mut figuras) = (if tem("conteudo") { corpo["conteudo"].clone() } else { registro["conteudo"].clone() }, if tem("figuras") { corpo["figuras"].clone() } else { registro["figuras"].clone() });
    if tem("operacoes") {
        aplicar(&mut conteudo, &mut figuras, &corpo["operacoes"])?;
    }
    let validada = validar(&conteudo, &figuras)?;
    let mut novo = registro.clone();
    novo["conteudo"] = conteudo;
    novo["figuras"] = if figuras.is_null() { json!([]) } else { figuras };
    novo["titulo"] = json!(validada.titulo);
    novo["publico"] = json!(validada.publico);
    novo["nivel"] = json!(validada.nivel);
    novo["editadoEm"] = json!(agora);
    novo["editadoPor"] = json!(autor);
    Ok((novo, validada.avisos))
}
