//! O terminal do app (ação "Claude Code dentro do app", decisão em docs/terminal-no-app.md): sessões de pseudoconsole (ConPTY) na memória do
//! servidor, uma por clique em "Iniciar com o Claude > Terminal do app". O processo é o `claude` oficial (via `blueprint claude`), com tudo o que ele
//! tem. A página só desenha: recebe a saída por SSE (`fluxo.rs`) e manda as teclas por POST (`rotas.rs`). A sessão vive no servidor, então a
//! janela pode ir para a bandeja (o WebView some) e o Claude continua; ao voltar, a página reconecta e recebe o histórico guardado.
#[cfg(windows)]
mod conpty;
mod fluxo;
pub mod rotas;
#[cfg(test)]
mod tests;

use serde_json::{json, Value};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex, MutexGuard};

/// Sessões com processo vivo ao mesmo tempo (cada uma é um claude inteiro: memória e tokens).
pub const MAX_VIVAS: usize = 8;
/// Sessões guardadas, contando as que terminaram (o usuário ainda lê o histórico até fechar a aba).
const MAX_GUARDADAS: usize = 16;
/// Quanto da saída cada sessão guarda para quem reconecta (a página recarregada, a janela que voltou da bandeja).
const HISTORICO: usize = 256 * 1024;
/// Pedaços na fila de cada página conectada. Página lenta que encher a fila é desligada (ela reconecta e recebe o histórico): o leitor do
/// pseudoconsole nunca espera por ela. É a contrapressão.
const FILA_ASSINANTE: usize = 256;

#[cfg(windows)]
use conpty::Controle;
#[cfg(not(windows))]
struct Controle;
#[cfg(not(windows))]
impl Controle {
    fn redimensionar(&self, _: i16, _: i16) -> bool {
        false
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Msg {
    Dados(String),
    Fim(Option<u32>),
}

struct Saida {
    historico: String,
    assinantes: Vec<SyncSender<Msg>>,
    viva: bool,
    codigo: Option<u32>,
}

/// O que a página informa ao abrir (para a aba e para achar a sessão da ação).
#[derive(Clone, Default)]
pub struct Info {
    pub titulo: String,
    pub workspace_id: String,
    pub blueprint_id: String,
    pub acao_id: String,
}

pub struct Sessao {
    pub id: String,
    pub info: Info,
    pub pasta: String,
    pub criada_em: String,
    saida: Mutex<Saida>,
    entrada: Mutex<Option<File>>,
    controle: Mutex<Option<Controle>>,
}

fn travar<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Junta `novo` ao que sobrou do pedaço anterior e devolve o texto UTF-8 completo; um caractere partido no fim espera o próximo pedaço.
pub(crate) fn texto_utf8(resto: &mut Vec<u8>, novo: &[u8]) -> String {
    resto.extend_from_slice(novo);
    let mut saida = String::new();
    loop {
        match std::str::from_utf8(resto) {
            Ok(s) => {
                saida.push_str(s);
                resto.clear();
                return saida;
            }
            Err(e) => {
                let ok = e.valid_up_to();
                saida.push_str(std::str::from_utf8(&resto[..ok]).unwrap_or_default());
                match e.error_len() {
                    None => {
                        resto.drain(..ok);
                        return saida;
                    }
                    Some(n) => {
                        saida.push('\u{FFFD}');
                        resto.drain(..ok + n);
                    }
                }
            }
        }
    }
}

/// Um argumento citado para a linha de comando do Windows (as regras do CommandLineToArgvW), para montar a linha sem passar pelo cmd.exe.
pub fn citar(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains([' ', '\t', '"']) {
        return arg.to_string();
    }
    let (mut s, mut barras) = (String::from('"'), 0);
    for c in arg.chars() {
        match c {
            '\\' => barras += 1,
            '"' => {
                s.push_str(&"\\".repeat(barras * 2 + 1));
                s.push('"');
                barras = 0;
            }
            _ => {
                s.push_str(&"\\".repeat(barras));
                s.push(c);
                barras = 0;
            }
        }
    }
    s.push_str(&"\\".repeat(barras * 2));
    s.push('"');
    s
}

impl Sessao {
    fn publicar(&self, msg: Msg) {
        let mut s = travar(&self.saida);
        match &msg {
            Msg::Dados(t) => {
                s.historico.push_str(t);
                if s.historico.len() > HISTORICO {
                    let mut corte = s.historico.len() - HISTORICO * 3 / 4; // corta em bloco, não a cada pedaço
                    while !s.historico.is_char_boundary(corte) {
                        corte += 1;
                    }
                    s.historico.drain(..corte);
                }
            }
            Msg::Fim(_) => s.viva = false,
        }
        s.assinantes.retain(|a| a.try_send(msg.clone()).is_ok());
        if !s.viva {
            s.assinantes.clear();
        }
    }

    /// O histórico até agora e, se o processo ainda vive, o canal com o que vier depois (os dois sob a mesma trava: nada se perde nem repete).
    pub fn assinar(&self) -> (String, Option<Receiver<Msg>>, Option<u32>) {
        let mut s = travar(&self.saida);
        if !s.viva {
            return (s.historico.clone(), None, s.codigo);
        }
        let (tx, rx) = sync_channel(FILA_ASSINANTE);
        s.assinantes.push(tx);
        (s.historico.clone(), Some(rx), None)
    }

    pub fn escrever(&self, bytes: &[u8]) -> bool {
        travar(&self.entrada).as_mut().is_some_and(|e| e.write_all(bytes).and_then(|_| e.flush()).is_ok())
    }

    pub fn redimensionar(&self, colunas: i16, linhas: i16) -> bool {
        travar(&self.controle).as_ref().is_some_and(|c| c.redimensionar(colunas.clamp(2, 1000), linhas.clamp(1, 500)))
    }

    /// Derruba o processo (e a árvore dele) e fecha o pseudoconsole; o leitor recebe o fim e avisa as páginas.
    pub fn encerrar(&self) {
        drop(travar(&self.controle).take());
        drop(travar(&self.entrada).take());
    }

    pub fn viva(&self) -> bool {
        travar(&self.saida).viva
    }

    pub fn json(&self) -> Value {
        let s = travar(&self.saida);
        let i = &self.info;
        json!({"id": self.id, "titulo": i.titulo, "workspaceId": i.workspace_id, "blueprintId": i.blueprint_id, "acaoId": i.acao_id, "pasta": self.pasta, "criadaEm": self.criada_em, "viva": s.viva, "codigo": s.codigo})
    }
}

#[derive(Default)]
pub struct Terminais {
    sessoes: Mutex<Vec<Arc<Sessao>>>,
    contador: AtomicU64,
}

impl Terminais {
    pub fn novo() -> Terminais {
        Terminais::default()
    }

    pub fn achar(&self, id: &str) -> Option<Arc<Sessao>> {
        travar(&self.sessoes).iter().find(|s| s.id == id).cloned()
    }

    /// Quantas sessões ainda têm o processo rodando (o updater não fecha o app com alguma viva).
    pub fn vivas(&self) -> usize {
        travar(&self.sessoes).iter().filter(|s| s.viva()).count()
    }

    pub fn listar(&self) -> Value {
        Value::Array(travar(&self.sessoes).iter().map(|s| s.json()).collect())
    }

    /// Tira a sessão da lista e derruba o processo, se ainda vive.
    pub fn fechar(&self, id: &str) -> bool {
        let mut lista = travar(&self.sessoes);
        let Some(i) = lista.iter().position(|s| s.id == id) else { return false };
        let sessao = lista.remove(i);
        drop(lista); // fechar o pseudoconsole pode esperar o leitor esvaziar o cano: sem segurar a lista
        sessao.encerrar();
        true
    }

    /// Abre `linha` (linha de comando citada) em `pasta` num pseudoconsole novo.
    pub fn abrir(&self, linha: &str, pasta: &Path, info: Info, colunas: i16, linhas: i16) -> Result<Arc<Sessao>, String> {
        let mut lista = travar(&self.sessoes);
        if lista.iter().filter(|s| s.viva()).count() >= MAX_VIVAS {
            return Err(format!("já há {MAX_VIVAS} sessões abertas no terminal do app; feche uma antes"));
        }
        while lista.len() >= MAX_GUARDADAS {
            match lista.iter().position(|s| !s.viva()) {
                Some(i) => drop(lista.remove(i)), // a mais antiga que já terminou
                None => break,
            }
        }
        let (controle, saida, entrada, esperar) = abrir_pty(linha, pasta, colunas.clamp(2, 1000), linhas.clamp(1, 500))?;
        let n = self.contador.fetch_add(1, Ordering::Relaxed) + 1;
        let sessao = Arc::new(Sessao {
            id: format!("t{n}"),
            info,
            pasta: pasta.to_string_lossy().into_owned(),
            criada_em: crate::tempo::iso_utc_agora(),
            saida: Mutex::new(Saida { historico: String::new(), assinantes: Vec::new(), viva: true, codigo: None }),
            entrada: Mutex::new(Some(entrada)),
            controle: Mutex::new(Some(controle)),
        });
        let leitor = (sessao.clone(), saida);
        std::thread::spawn(move || ler(&leitor.0, leitor.1));
        let vigia = sessao.clone();
        std::thread::spawn(move || {
            let codigo = esperar();
            travar(&vigia.saida).codigo = codigo;
            drop(travar(&vigia.controle).take()); // fecha o pseudoconsole: o leitor recebe o fim
        });
        lista.push(sessao.clone());
        Ok(sessao)
    }
}

/// Lê a saída até o pseudoconsole fechar (bloqueado no ReadFile: zero CPU em repouso) e repassa o texto às páginas.
fn ler(sessao: &Sessao, mut saida: File) {
    let (mut buf, mut resto) = (vec![0u8; 16 * 1024], Vec::new());
    while let Ok(n) = saida.read(&mut buf) {
        if n == 0 {
            break;
        }
        let t = texto_utf8(&mut resto, &buf[..n]);
        if !t.is_empty() {
            sessao.publicar(Msg::Dados(t));
        }
    }
    let codigo = travar(&sessao.saida).codigo;
    sessao.publicar(Msg::Fim(codigo));
}

type Esperar = Box<dyn FnOnce() -> Option<u32> + Send>;

#[cfg(windows)]
fn abrir_pty(linha: &str, pasta: &Path, colunas: i16, linhas: i16) -> Result<(Controle, File, File, Esperar), String> {
    let a = conpty::abrir(linha, pasta, colunas, linhas)?;
    let processo = a.processo;
    Ok((a.controle, a.saida, a.entrada, Box::new(move || processo.esperar())))
}

#[cfg(not(windows))]
fn abrir_pty(_: &str, _: &Path, _: i16, _: i16) -> Result<(Controle, File, File, Esperar), String> {
    Err("o terminal do app só existe no Windows (ConPTY)".into())
}
