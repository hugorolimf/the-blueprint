//! Datas aaaa-mm-dd. Como o formato é fixo, comparar os textos é comparar as datas.

fn dias_no_mes(ano: u32, mes: u32) -> u32 {
    match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if (ano % 4 == 0 && ano % 100 != 0) || ano % 400 == 0 => 29,
        _ => 28,
    }
}

/// `aaaa-mm-dd` de um dia que existe no calendário (2026-02-30 não vale).
pub fn dia_valido(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || !b.iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() }) {
        return false;
    }
    let (ano, mes, dia): (u32, u32, u32) = (s[0..4].parse().unwrap_or(0), s[5..7].parse().unwrap_or(0), s[8..10].parse().unwrap_or(0));
    (1..=12).contains(&mes) && dia >= 1 && dia <= dias_no_mes(ano, mes)
}

/// Dias desde 1970-01-01 de uma data `aaaa-mm-dd` válida (algoritmo `days_from_civil`); serve para contar dias entre duas datas.
pub fn numero_do_dia(s: &str) -> i64 {
    let (ano, mes, dia): (i64, i64, i64) = (s[0..4].parse().unwrap_or(0), s[5..7].parse().unwrap_or(0), s[8..10].parse().unwrap_or(0));
    let ano = if mes <= 2 { ano - 1 } else { ano };
    let era = ano.div_euclid(400);
    let ano_da_era = ano - era * 400;
    let dia_do_ano = (153 * (if mes > 2 { mes - 3 } else { mes + 9 }) + 2) / 5 + dia - 1;
    let dia_da_era = ano_da_era * 365 + ano_da_era / 4 - ano_da_era / 100 + dia_do_ano;
    era * 146_097 + dia_da_era - 719_468
}

/// Quantos dias há de `a` até `b` (b depois de a dá positivo); as duas precisam ser datas válidas.
pub fn dias_entre(a: &str, b: &str) -> i64 {
    numero_do_dia(b) - numero_do_dia(a)
}

/// Milissegundos desde 1970 de um instante ISO `aaaa-mm-ddThh:mm:ss(.mmm)Z`; `None` se o texto não tiver esse formato.
pub fn ms_do_iso(s: &str) -> Option<i64> {
    let (dia, hora) = s.split_once('T')?;
    if !dia_valido(dia) {
        return None;
    }
    let hora = hora.trim_end_matches('Z');
    let (hms, fracao) = hora.split_once('.').unwrap_or((hora, "0"));
    let mut partes = hms.split(':').map(|p| p.parse::<i64>().ok());
    let (h, m, seg) = (partes.next()??, partes.next()??, partes.next()??);
    let ms: i64 = format!("{:0<3}", fracao).get(0..3)?.parse().ok()?;
    Some(((numero_do_dia(dia) * 24 + h) * 60 + m) * 60_000 + seg * 1000 + ms)
}
