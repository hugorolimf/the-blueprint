//! Datas e horas sem dependência externa: a hora UTC em ISO 8601 (campo `em` dos eventos) e o dia local (`hoje`).
use std::time::{SystemTime, UNIX_EPOCH};

/// (ano, mês, dia) de um número de dias desde 1970-01-01 (algoritmo civil_from_days, de Howard Hinnant).
fn civil(dias: i64) -> (i64, u32, u32) {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let ano = yoe + era * 400 + i64::from(mes <= 2);
    (ano, mes, dia)
}

/// Milissegundos desde 1970 (UTC).
pub fn agora_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// `2026-10-22T02:18:35.123Z`, como o `toISOString()` do JavaScript.
pub fn iso_utc_agora() -> String {
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
    iso_utc(ms)
}

pub fn iso_utc(ms: i64) -> String {
    let (dias, resto) = (ms.div_euclid(86_400_000), ms.rem_euclid(86_400_000));
    let (a, m, d) = civil(dias);
    let (h, min, s, milis) = (resto / 3_600_000, resto / 60_000 % 60, resto / 1000 % 60, resto % 1000);
    format!("{a:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}.{milis:03}Z")
}

#[cfg(windows)]
fn dia_local() -> (i64, u32, u32) {
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    // SAFETY: SYSTEMTIME é só números e `GetLocalTime` preenche a estrutura inteira.
    let t = unsafe {
        let mut t: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut t);
        t
    };
    (i64::from(t.wYear), u32::from(t.wMonth), u32::from(t.wDay))
}

/// Fora do Windows usa o dia em UTC (o app é para Windows; isto só deixa o código compilar e testar em outro sistema).
#[cfg(not(windows))]
fn dia_local() -> (i64, u32, u32) {
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
    civil(ms.div_euclid(86_400_000))
}

/// O dia de hoje no fuso do usuário, `aaaa-mm-dd` (o `hojeIso()` do plano.mjs).
pub fn hoje_local() -> String {
    let (a, m, d) = dia_local();
    format!("{a:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests;
