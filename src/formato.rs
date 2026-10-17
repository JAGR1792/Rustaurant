//! Formato de fechas y montos de la presentación (fase 1 asume la hora
//! de Colombia, UTC-5 sin horario de verano): una sola fuente para las
//! vistas y los correos.

use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, Utc};

/// Fase 1 asume la hora de Colombia (UTC-5, sin horario de verano).
pub(crate) const UTC_A_LOCAL: i64 = 5; // UTC = local + 5 horas

pub(crate) fn local_a_utc(n: NaiveDateTime) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(n + Duration::hours(UTC_A_LOCAL), Utc)
}

pub(crate) fn utc_a_local(dt: DateTime<Utc>) -> NaiveDateTime {
    dt.naive_utc() - Duration::hours(UTC_A_LOCAL)
}

/// "2026-10-18 19:30" en hora local.
pub(crate) fn fecha_hora_fmt(dt: DateTime<Utc>) -> String {
    utc_a_local(dt).format("%Y-%m-%d %H:%M").to_string()
}

/// ISO 8601 con el desfase local (UTC-5) para que el navegador calcule la
/// cuenta regresiva con la hora correcta (p. ej. 2026-10-19T19:30:00-05:00).
pub(crate) fn fecha_iso_local(dt: DateTime<Utc>) -> String {
    let offset = FixedOffset::west_opt((UTC_A_LOCAL * 3600) as i32).expect("desfase UTC-5 válido");
    dt.with_timezone(&offset)
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
}

/// Pesos colombianos con separador de miles: 440000 -> "440.000".
pub(crate) fn cop(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn fecha_iso_local_lleva_el_desfase_de_colombia() {
        let dt = Utc.with_ymd_and_hms(2026, 10, 20, 0, 30, 0).unwrap();
        assert_eq!(fecha_iso_local(dt), "2026-10-19T19:30:00-05:00");
    }

    #[test]
    fn cop_separa_miles() {
        assert_eq!(cop(440_000), "440.000");
        assert_eq!(cop(15_000), "15.000");
        assert_eq!(cop(0), "0");
    }
}
