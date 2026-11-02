//! `plano/figura` e `planejar/figura` (D13): a figura mora em `<dados>/figuras/<sha256>.<ext>`, por conteúdo, gravada com `.tmp` + rename; `plano_figuras` guarda hash e medidas por versão.
//! Servir olha só o disco: a árvore continua citando o hash depois que a retenção apaga a linha da versão antiga.
use super::super::hooks::travado;
use super::escrita::{aberta, doc_gravado};
use super::Mcp;
use crate::dados::planejar::FiguraLinha;
use crate::dados::Dados;
use crate::http::pedido::{falha, Corpo, ErroHttp, Pedido, Resposta};
use crate::plano::estilo::{compilar_estilo, resolver};
use crate::plano::ids::sha256_hex;
use crate::plano::svg::{imagem_tipo, medidas_imagem, medidas_svg, svg_seguro, LIMITE_IMAGEM, LIMITE_SVG, MAX_FIGURAS};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Mutex;

const TIPOS: [(&str, &str); 4] = [("svg", "image/svg+xml"), ("png", "image/png"), ("jpg", "image/jpeg"), ("webp", "image/webp")];

pub fn rota(mcp: &Mcp, dados: &Mutex<Dados>, pedido: &Pedido, recurso: &str, corpo: &Value) -> Result<Resposta, ErroHttp> {
    match recurso {
        "plano/figura" => receber(mcp, dados, corpo),
        "planejar/figura" => servir(mcp, pedido),
        _ => Err(super::nao_encontrado()),
    }
}

fn recusada(erro: String) -> Resposta {
    Resposta::json(422, &json!({"erro": erro, "codigo": "figura-invalida", "erros": [erro], "avisos": []}))
}

/// Só uma chave da versão, o disco não é lido; a regra é a de `pacote::ler_pasta` para o mesmo caminho valer nos dois.
fn caminho_ok(c: &str, tipo: &str) -> Result<(), String> {
    let partes: Vec<&str> = c.split(['/', '\\']).collect();
    if c.is_empty() || c.chars().count() > 500 || c.chars().any(|x| x.is_control() || x == ':') || partes[0].is_empty() || partes.contains(&"..") {
        return Err(format!("{c}: must be a relative path inside the plan folder"));
    }
    let ext = c.rsplit_once('.').map_or(String::new(), |(_, e)| e.to_ascii_lowercase());
    if ext != tipo && !(ext == "jpeg" && tipo == "jpg") {
        return Err(format!("{c}: the extension does not match the type {tipo}"));
    }
    Ok(())
}

fn valor_base64(c: u8) -> Option<u32> {
    match c {
        b'A'..=b'Z' => Some(u32::from(c - b'A')),
        b'a'..=b'z' => Some(u32::from(c - b'a') + 26),
        b'0'..=b'9' => Some(u32::from(c - b'0') + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Base64 padrão, com ou sem preenchimento e com quebras de linha. A raiz não tem o crate, e o resto do crate só codifica.
fn de_base64(s: &str) -> Option<Vec<u8>> {
    let limpos: Vec<u8> = s.bytes().filter(|c| !c.is_ascii_whitespace()).collect();
    let fim = limpos.iter().rposition(|c| *c != b'=').map_or(0, |i| i + 1);
    let mut saida = Vec::with_capacity(fim / 4 * 3 + 3);
    let (mut acumulado, mut bits) = (0u32, 0u32);
    for c in &limpos[..fim] {
        acumulado = (acumulado << 6) | valor_base64(*c)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            saida.push((acumulado >> bits) as u8);
            acumulado &= (1 << bits) - 1;
        }
    }
    Some(saida)
}

/// O SVG roda em `svg_seguro` antes e depois de `resolver`: valor que vem do CSS do plano só entra depois. Devolve os bytes e as medidas.
fn pronta_svg(texto: &str, css: &str, tema: &std::collections::BTreeMap<String, String>) -> Result<(Vec<u8>, Option<(u32, u32)>), String> {
    if texto.len() > LIMITE_SVG {
        return Err(format!("larger than {} MB", LIMITE_SVG >> 20));
    }
    svg_seguro(texto)?;
    let resolvido = resolver(texto, &compilar_estilo(css, tema).1);
    if resolvido.len() > LIMITE_SVG {
        return Err(format!("larger than {} MB after resolving the style", LIMITE_SVG >> 20));
    }
    svg_seguro(&resolvido)?;
    let medidas = medidas_svg(&resolvido);
    Ok((resolvido.into_bytes(), medidas))
}

fn pronta_imagem(base64: &str, tipo: &str) -> Result<(Vec<u8>, Option<(u32, u32)>), String> {
    let bytes = de_base64(base64).ok_or("base64 is not valid")?;
    if bytes.len() > LIMITE_IMAGEM {
        return Err(format!("larger than {} MB", LIMITE_IMAGEM >> 20));
    }
    if imagem_tipo(&bytes) != Some(tipo) {
        return Err(format!("the content is not a {tipo} file"));
    }
    let medidas = medidas_imagem(&bytes);
    Ok((bytes, medidas))
}

/// Por conteúdo: já existe, não reescreve. O nome temporário é único, então dois envios ao mesmo tempo não se atrapalham.
fn gravar_no_disco(pasta: &Path, hash: &str, ext: &str, bytes: &[u8]) -> std::io::Result<()> {
    let destino = pasta.join(format!("{hash}.{ext}"));
    if destino.is_file() {
        return Ok(());
    }
    std::fs::create_dir_all(pasta)?;
    let tmp = pasta.join(format!("{hash}.{}.tmp", super::novo_gesto()));
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, &destino).inspect_err(|_| drop(std::fs::remove_file(&tmp)))
}

fn receber(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> Result<Resposta, ErroHttp> {
    let campo = |c: &str| corpo.get(c).and_then(Value::as_str);
    let (escrita_id, caminho, tipo) = (campo("escritaId").unwrap_or(""), campo("caminho").unwrap_or(""), campo("tipo").unwrap_or(""));
    let (estilo, tema, cheia) = {
        let d = travado(dados);
        let linha = aberta(&d, escrita_id)?;
        // Reenviar o mesmo caminho substitui, então não conta: retomar uma escrita cheia continua valendo.
        let cheia = d.figuras_da_versao(&linha.blueprint_id, linha.numero)? >= MAX_FIGURAS && d.figura_de(&linha.blueprint_id, linha.numero, caminho)?.is_none();
        let doc = doc_gravado(&linha.plano)?;
        (doc.estilo.unwrap_or_default(), doc.tema, cheia)
    };
    if cheia {
        return Ok(recusada(format!("{caminho}: more than {MAX_FIGURAS} figures in this version")));
    }
    if !TIPOS.iter().any(|(t, _)| *t == tipo) {
        return Ok(recusada(format!("{caminho}: tipo must be svg, png, jpg or webp")));
    }
    if let Err(e) = caminho_ok(caminho, tipo) {
        return Ok(recusada(e));
    }
    // Sem o `Mutex<Dados>` no trabalho pesado: resolver o estilo e gravar em disco não seguram o resto do app.
    let pronta = match (tipo, campo("svg"), campo("base64")) {
        ("svg", Some(texto), _) => pronta_svg(texto, &estilo, &tema),
        ("svg", None, _) => Err("the svg field is required".to_string()),
        (_, _, Some(b)) => pronta_imagem(b, tipo),
        _ => Err("the base64 field is required".to_string()),
    };
    let (bytes, medidas) = match pronta {
        Ok(p) => p,
        Err(e) => return Ok(recusada(format!("{caminho}: {e}"))),
    };
    let hash = sha256_hex(&bytes);
    gravar_no_disco(&mcp.pasta_figuras(), &hash, tipo, &bytes).map_err(|e| falha(500, e.to_string()))?;
    let mut d = travado(dados);
    let linha = aberta(&d, escrita_id)?; // pode ter fechado enquanto o disco trabalhava
    d.figura_registrar(&FiguraLinha { blueprint_id: linha.blueprint_id, numero: linha.numero, caminho: caminho.to_string(), hash: hash.clone(), tipo: tipo.to_string(), tamanho: bytes.len() as i64, largura: medidas.map(|m| m.0), altura: medidas.map(|m| m.1) })?;
    drop(d);
    mcp.planejar_mudou();
    Ok(Resposta::json(200, &json!({"ok": true, "hash": hash})))
}

/// À mão: `Resposta::texto` acrescenta charset, e o SVG é aberto em sandbox, sem nenhuma origem.
fn servir(mcp: &Mcp, pedido: &Pedido) -> Result<Resposta, ErroHttp> {
    let hash = pedido.parametro("hash").unwrap_or_default();
    if hash.len() != 64 || !hash.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
        return Err(falha(400, crate::tr!("hash must be 64 hexadecimal characters", "o hash deve ter 64 caracteres hexadecimais")).codigo("hash-invalido"));
    }
    let pasta = mcp.pasta_figuras();
    let achada = TIPOS.iter().find_map(|(ext, tipo)| std::fs::read(pasta.join(format!("{hash}.{ext}"))).ok().map(|b| (*tipo, b)));
    let Some((tipo, bytes)) = achada else { return Err(super::nao_encontrado()) };
    let cabecalhos = [("content-type", tipo), ("x-content-type-options", "nosniff"), ("content-security-policy", "sandbox; default-src 'none'; style-src 'unsafe-inline'"), ("cache-control", "no-store")];
    Ok(Resposta { status: 200, cabecalhos: cabecalhos.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), corpo: Corpo::Bytes(bytes) })
}
