//! Limite de tentativas com segredo inválido: 10 por IP ou 200 no total, numa janela de 15 min. Só na memória (reiniciar zera, e está tudo
//! bem). Só as falhas contam; nada daqui guarda o segredo tentado.
use std::collections::{HashMap, VecDeque};

pub const JANELA_MS: i64 = 15 * 60 * 1000;
pub const POR_IP: usize = 10;
pub const TOTAL: usize = 200;

#[derive(Default)]
pub struct Limite {
    por_ip: HashMap<String, VecDeque<i64>>,
    todas: VecDeque<i64>,
}

fn esquecer_velhas(fila: &mut VecDeque<i64>, agora: i64) {
    while fila.front().is_some_and(|t| *t <= agora - JANELA_MS) {
        fila.pop_front();
    }
}

impl Limite {
    /// Segundos até poder tentar de novo; None = pode tentar.
    pub fn bloqueado(&mut self, ip: &str, agora: i64) -> Option<i64> {
        esquecer_velhas(&mut self.todas, agora);
        let ip_fila = self.por_ip.get_mut(ip).map(|f| {
            esquecer_velhas(f, agora);
            f.clone()
        });
        let mut libera: Option<i64> = None;
        if self.todas.len() >= TOTAL {
            libera = self.todas.front().copied();
        }
        if let Some(f) = ip_fila.filter(|f| f.len() >= POR_IP) {
            libera = libera.max(f.front().copied());
        }
        libera.map(|primeira| ((primeira + JANELA_MS - agora) / 1000).max(1))
    }

    pub fn falhou(&mut self, ip: &str, agora: i64) {
        self.todas.push_back(agora);
        let f = self.por_ip.entry(ip.to_string()).or_default();
        f.push_back(agora);
        // sem crescer sem fim: IPs sem falha recente saem
        if self.por_ip.len() > 10_000 {
            self.por_ip.retain(|_, f| f.back().is_some_and(|t| *t > agora - JANELA_MS));
        }
    }
}
