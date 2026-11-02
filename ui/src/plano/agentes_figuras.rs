//! Os quadros das figuras do Claude nesta tela: o agente de plano (`construtor`) em cada pose e o orquestrador (`principal`), com a ordem e o ritmo de `folha/figuras.rs`.
//! Só o que mexe tem vários quadros; o que está parado fica no primeiro, para a tela não repintar à toa.

use super::regras::Pose;
use std::time::Duration;

macro_rules! quadros {
    ($($nome:literal),* $(,)?) => {
        [$(($nome, include_bytes!(concat!("../../assets/claude/", $nome, ".svg.gz")) as &[u8])),*]
    };
}

type Quadro = (&'static str, &'static [u8]);

const TRABALHANDO: [Quadro; 5] = quadros!("construtor-trabalhando-0", "construtor-trabalhando-1", "construtor-trabalhando-2", "construtor-trabalhando-3", "construtor-trabalhando-4");
const ESPERANDO: [Quadro; 3] = quadros!("construtor-esperando-0", "construtor-esperando-1", "construtor-esperando-2");
const BLOQUEADO: [Quadro; 1] = quadros!("construtor-bloqueado-0");
const CONCLUIDO: [Quadro; 1] = quadros!("construtor-concluido-0");
const PARADO: [Quadro; 1] = quadros!("construtor-parado-0");
const PRINCIPAL_ACORDADO: [Quadro; 5] = quadros!("principal-trabalhando-0", "principal-trabalhando-1", "principal-trabalhando-2", "principal-trabalhando-3", "principal-trabalhando-4");
const PRINCIPAL_DORMINDO: [Quadro; 1] = quadros!("principal-parado-0");

const ORDEM_TRABALHANDO: [usize; 8] = [0, 1, 2, 1, 0, 3, 4, 3];
const ORDEM_ESPERANDO: [usize; 3] = [0, 1, 2];

/// O quadro de `ms` milissegundos e quando pedir o próximo (`None` se a figura está parada).
fn escolher(lista: &'static [Quadro], ordem: &[usize], ritmo: u64, ms: u64) -> (Quadro, Option<Duration>) {
    if ordem.len() < 2 {
        return (lista[0], None);
    }
    let passo = ms / ritmo;
    (lista[ordem[passo as usize % ordem.len()]], Some(Duration::from_millis(ritmo - ms % ritmo)))
}

/// A figura do agente de um plano nesta pose, no instante `ms` (0 com o movimento desligado).
pub fn do_agente(p: Pose, ms: u64) -> (Quadro, Option<Duration>) {
    match p {
        Pose::Trabalhando => escolher(&TRABALHANDO, &ORDEM_TRABALHANDO, 110, ms),
        Pose::Esperando => escolher(&ESPERANDO, &ORDEM_ESPERANDO, 300, ms),
        Pose::Bloqueado => escolher(&BLOQUEADO, &[0], 1000, ms),
        Pose::Concluido => escolher(&CONCLUIDO, &[0], 1000, ms),
        Pose::Parado => escolher(&PARADO, &[0], 1000, ms),
    }
}

/// A figura do orquestrador: acordado trabalha, senão dorme.
pub fn do_orquestrador(acordado: bool, ms: u64) -> (Quadro, Option<Duration>) {
    if acordado {
        escolher(&PRINCIPAL_ACORDADO, &ORDEM_TRABALHANDO, 110, ms)
    } else {
        escolher(&PRINCIPAL_DORMINDO, &[0], 1000, ms)
    }
}
