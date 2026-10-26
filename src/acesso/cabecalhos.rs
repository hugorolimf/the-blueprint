//! HTTPS, Host, Origin e os cabeçalhos de segurança do modo hospedado (docs/colaboracao.md, seção 4). O TLS fica com o proxy reverso: o app
//! confia em `X-Forwarded-Proto` e `X-Forwarded-For` só quando o par da conexão é esta máquina ou o BLUEPRINT_PROXY.
use crate::http::config::Hospedagem;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use std::net::IpAddr;

/// O par da conexão é o proxy? (loopback ou o IP de BLUEPRINT_PROXY; vazio ou inválido nunca é)
pub fn par_confiavel(h: &Hospedagem, ip: &str) -> bool {
    let Ok(ip) = ip.parse::<IpAddr>() else { return false };
    ip.is_loopback() || h.proxy.as_deref().and_then(|p| p.parse::<IpAddr>().ok()) == Some(ip)
}

/// O IP de quem pede: o último endereço de `X-Forwarded-For` (o que o proxy viu) se o par é o proxy, senão o par.
pub fn ip_cliente(h: &Hospedagem, pedido: &Pedido) -> String {
    if par_confiavel(h, &pedido.ip) {
        if let Some(ip) = pedido.cabecalho("x-forwarded-for").and_then(|v| v.rsplit(',').next()).map(str::trim).filter(|v| !v.is_empty()) {
            return ip.to_string();
        }
    }
    pedido.ip.clone()
}

/// Chegou por HTTPS (pelo proxy) e para o host público? Senão, 403 `https-obrigatorio` ou `host-invalido` (inclusive a página).
pub fn conferir(h: &Hospedagem, pedido: &Pedido) -> Result<(), ErroHttp> {
    let proto = pedido.cabecalho("x-forwarded-proto").and_then(|v| v.split(',').next()).map(str::trim);
    if !par_confiavel(h, &pedido.ip) || !proto.is_some_and(|p| p.eq_ignore_ascii_case("https")) {
        return Err(falha(403, crate::tr!("HTTPS is required (through the reverse proxy)", "é preciso HTTPS (pelo proxy reverso)")).codigo("https-obrigatorio"));
    }
    if !pedido.cabecalho("host").is_some_and(|host| host.eq_ignore_ascii_case(&h.host_publico)) {
        return Err(falha(403, crate::tr!("invalid host", "host inválido")).codigo("host-invalido"));
    }
    Ok(())
}

/// A Origin é a da URL pública? (pedido sem Origin não passa: o navegador sempre a manda num POST)
pub fn origem_ok(h: &Hospedagem, pedido: &Pedido) -> bool {
    pedido.cabecalho("origin").is_some_and(|o| o.eq_ignore_ascii_case(&h.url_publica))
}

/// Os cabeçalhos de segurança de toda resposta do modo hospedado.
pub fn proteger(h: &Hospedagem, caminho: &str, mut r: Resposta) -> Resposta {
    let quadro = if caminho.starts_with("/prototipos/") { "SAMEORIGIN" } else { "DENY" };
    r = r.com("referrer-policy", "no-referrer").com("x-content-type-options", "nosniff").com("x-frame-options", quadro);
    if caminho.starts_with("/api/") {
        r = r.com("cache-control", "no-store");
    }
    if h.hsts {
        r = r.com("strict-transport-security", "max-age=31536000");
    }
    r
}
