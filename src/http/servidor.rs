//! O laço do servidor: aceita conexões (tiny_http), monta o `Pedido`, chama o roteador e escreve a `Resposta`.
//! Uma thread por pedido: o app é local e os pedidos são poucos, e assim uma conexão SSE parada não bloqueia mais ninguém.
use super::config::Config;
use super::estado::{abrir_dados, Estado};
use super::estaticos::Assets;
use super::pedido::{Corpo, Pedido, Resposta};
use super::rotas::{corpo_grande_demais, despachar, limite_corpo};
use crate::atualizador::Atualizador;
use crate::erro::{Erro, Resultado};
use crate::mcp::Mcp;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tiny_http::{Header, Request, Response, Server, StatusCode};

pub struct Servidor {
    pub porta: u16,
    pub url: String,
    pub estado: Arc<Estado>,
    servidor: Arc<Server>,
    fio: Option<JoinHandle<()>>,
}

/// Abre o banco, liga o servidor e começa a atender em segundo plano. Com `porta = 0` o sistema escolhe uma porta livre.
pub fn iniciar(config: Config) -> Resultado<Servidor> {
    let dados = abrir_dados(&config)?;
    let servidor = Server::http((config.host.as_str(), config.porta)).map_err(|e| Erro::Io(std::io::Error::other(format!("não consegui escutar em {}:{}: {e}", config.host, config.porta))))?;
    let porta = servidor.server_addr().to_ip().map_or(config.porta, |a| a.port());
    let url = config.url(porta);
    let mcp = Mcp::novo(&config.host, config.rede_mcp, &config.exe.to_string_lossy(), &url, config.prototipos.clone());
    let (versao_dados, sincronia) = super::estado::versao_e_sincronia(crate::sincronia::presenca_do_ambiente());
    let estado = Arc::new(Estado { ao_mostrar: Mutex::new(None), ao_sair: Mutex::new(None), atualizador: Atualizador::do_ambiente(&config.banco), assets: Assets::novo(config.assets_dir.clone()), dados: Mutex::new(dados), dados_json: Mutex::new(None), versao_dados, sincronia, mcp, terminais: crate::terminal::Terminais::novo(), config });
    // cada mudança do updater (baixando, pronta…) chega à página pelo mesmo SSE da fila
    let fraco = Arc::downgrade(&estado);
    estado.atualizador.ouvir(move |estado_json| {
        if let Some(e) = fraco.upgrade() {
            e.mcp.notificar("atualizacao", Some(estado_json.clone()));
        }
    });
    // o que o agente registrou com o app fechado entra na fila antes de o primeiro pedido chegar (src/mcp/pendentes.rs)
    crate::mcp::pendentes::reenviar_todos(&estado.mcp, &estado.dados, &crate::mcp::pendentes::pasta_do_banco(&estado.config.banco));
    let servidor = Arc::new(servidor);
    let fio = {
        let (servidor, estado) = (servidor.clone(), estado.clone());
        thread::spawn(move || {
            for pedido in servidor.incoming_requests() {
                let estado = estado.clone();
                thread::spawn(move || atender(&estado, pedido));
            }
        })
    };
    Ok(Servidor { porta, url, estado, servidor, fio: Some(fio) })
}

impl Servidor {
    /// Fica aqui até o servidor parar (o modo `serve` não faz mais nada).
    pub fn aguardar(mut self) {
        if let Some(fio) = self.fio.take() {
            let _ = fio.join();
        }
    }

    pub fn parar(&self) {
        self.servidor.unblock();
    }
}

fn atender(estado: &Estado, mut req: Request) {
    let metodo = req.method().as_str().to_uppercase();
    let url = req.url().to_string();
    let cabecalhos: Vec<(String, String)> = req.headers().iter().map(|h| (h.field.as_str().as_str().to_string(), h.value.as_str().to_string())).collect();
    let caminho = url.split('?').next().unwrap_or("").to_string();
    let mut corpo = Vec::new();
    if metodo == "POST" || metodo == "PUT" {
        let limite = limite_corpo(&metodo, &caminho);
        let lido = req.as_reader().take(limite as u64 + 1).read_to_end(&mut corpo);
        if lido.is_err() || corpo.len() > limite {
            return escrever(req, corpo_grande_demais(&caminho));
        }
    }
    let resposta = despachar(estado, &Pedido::novo(&metodo, &url, cabecalhos, corpo));
    escrever(req, resposta);
}

fn cabecalhos_de(resposta: &Resposta) -> Vec<Header> {
    resposta.cabecalhos.iter().filter_map(|(k, v)| Header::from_bytes(k.as_bytes(), v.as_bytes()).ok()).collect()
}

fn escrever(req: Request, resposta: Resposta) {
    let cabecalhos = cabecalhos_de(&resposta);
    match resposta.corpo {
        Corpo::Bytes(bytes) => {
            let mut r = Response::from_data(bytes).with_status_code(StatusCode(resposta.status));
            for h in cabecalhos {
                r.add_header(h);
            }
            let _ = req.respond(r);
        }
        // SSE: escreve direto na conexão e descarrega a cada trecho (a resposta normal seria guardada em buffer); só termina quando o cliente sai
        Corpo::Fluxo(mut leitor) => {
            let mut saida = req.into_writer();
            let mut cabeca = format!("HTTP/1.1 {} OK\r\n", resposta.status);
            for h in &cabecalhos {
                cabeca.push_str(&format!("{}: {}\r\n", h.field, h.value));
            }
            cabeca.push_str("Connection: close\r\n\r\n");
            if saida.write_all(cabeca.as_bytes()).and_then(|_| saida.flush()).is_err() {
                return;
            }
            let mut trecho = [0u8; 1024];
            loop {
                match leitor.read(&mut trecho) {
                    Ok(0) | Err(_) => return,
                    Ok(n) => {
                        if saida.write_all(&trecho[..n]).and_then(|_| saida.flush()).is_err() {
                            return;
                        }
                    }
                }
            }
        }
    }
}
