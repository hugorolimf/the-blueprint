//! `orquestrador`: por workspace, se está ligado, por que deve acordar e o teto por hora. Workspace sem linha é desligado. A regra de quando acordar é de `mcp/planejar/orquestrador.rs`.
use super::lista_de;
use crate::dados::Dados;
use crate::erro::Resultado;
use rusqlite::OptionalExtension;

/// Motivos à espera, no máximo; os mais velhos saem. Com o orquestrador desligado eles também juntam, e este teto os segura.
const MAX_MOTIVOS: usize = 50;
const UMA_HORA_MS: i64 = 3_600_000;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrqLinha {
    pub workspace_id: String,
    pub ligado: bool,
    pub motivos: Vec<String>,
    pub urgente: bool,
    pub rodando: bool,
    pub ultimo_em: Option<i64>,
    pub hora_inicio: Option<i64>,
    pub na_hora: i64,
}

impl OrqLinha {
    /// Quantas vezes acordou na hora que está valendo em `agora_ms` (a contagem guardada é de uma hora que pode já ter passado).
    pub fn na_hora_em(&self, agora_ms: i64) -> i64 {
        match self.hora_inicio {
            Some(inicio) if agora_ms - inicio < UMA_HORA_MS => self.na_hora,
            _ => 0,
        }
    }
}

impl Dados {
    pub fn orq_ler(&self, ws: &str) -> Resultado<OrqLinha> {
        let linha = self
            .conn
            .query_row("SELECT ligado, motivos, urgente, rodando, ultimo_em, hora_inicio, na_hora FROM orquestrador WHERE workspace_id = ?1", [ws], |l| {
                Ok(OrqLinha {
                    workspace_id: ws.to_string(),
                    ligado: l.get::<_, i64>(0)? != 0,
                    motivos: lista_de(&l.get::<_, String>(1)?),
                    urgente: l.get::<_, i64>(2)? != 0,
                    rodando: l.get::<_, i64>(3)? != 0,
                    ultimo_em: l.get(4)?,
                    hora_inicio: l.get(5)?,
                    na_hora: l.get(6)?,
                })
            })
            .optional()?;
        Ok(linha.unwrap_or_else(|| OrqLinha { workspace_id: ws.to_string(), ..OrqLinha::default() }))
    }

    pub fn orq_ligar(&mut self, ws: &str, ligado: bool) -> Resultado<()> {
        self.conn.execute(
            "INSERT INTO orquestrador (workspace_id, ligado) VALUES (?1, ?2) ON CONFLICT (workspace_id) DO UPDATE SET ligado = excluded.ligado",
            (ws, ligado as i64),
        )?;
        Ok(())
    }

    /// Junta um motivo para o próximo acordar, sem repetir. Quem decide se acorda é o `passo`, que olha `ligado`.
    pub fn orq_pedir(&mut self, ws: &str, motivo: &str, urgente: bool) -> Resultado<()> {
        let mut motivos = self.orq_ler(ws)?.motivos;
        if !motivos.iter().any(|m| m == motivo) {
            motivos.push(motivo.to_string());
        }
        if motivos.len() > MAX_MOTIVOS {
            motivos.drain(..motivos.len() - MAX_MOTIVOS);
        }
        self.conn.execute(
            "INSERT INTO orquestrador (workspace_id, motivos, urgente) VALUES (?1, ?2, ?3)
             ON CONFLICT (workspace_id) DO UPDATE SET motivos = excluded.motivos, urgente = MAX(urgente, excluded.urgente)",
            (ws, serde_json::to_string(&motivos)?, urgente as i64),
        )?;
        Ok(())
    }

    /// Acordou: leva os motivos (o que chegar agora fica para a próxima), marca `rodando` e conta na hora. Já rodando, `None` e nada muda.
    pub fn orq_inicio(&mut self, ws: &str, agora_ms: i64) -> Resultado<Option<Vec<String>>> {
        let atual = self.orq_ler(ws)?;
        if atual.rodando {
            return Ok(None);
        }
        let conta_nova = atual.na_hora_em(agora_ms) == 0;
        let hora_inicio = if conta_nova { agora_ms } else { atual.hora_inicio.unwrap_or(agora_ms) };
        self.conn.execute(
            "INSERT INTO orquestrador (workspace_id, motivos, urgente, rodando, ultimo_em, hora_inicio, na_hora) VALUES (?1, '[]', 0, 1, ?2, ?3, ?4)
             ON CONFLICT (workspace_id) DO UPDATE SET motivos = '[]', urgente = 0, rodando = 1, ultimo_em = excluded.ultimo_em, hora_inicio = excluded.hora_inicio, na_hora = excluded.na_hora",
            (ws, agora_ms, hora_inicio, atual.na_hora_em(agora_ms) + 1),
        )?;
        Ok(Some(atual.motivos))
    }

    /// O processo do orquestrador terminou; `ultimo_em` conta a espera a partir daqui.
    pub fn orq_fim(&mut self, ws: &str, agora_ms: i64) -> Resultado<()> {
        self.conn.execute("UPDATE orquestrador SET rodando = 0, ultimo_em = ?1 WHERE workspace_id = ?2", (agora_ms, ws))?;
        Ok(())
    }

    /// Ao reiniciar o app nenhum orquestrador está rodando, seja qual for o workspace.
    pub fn orq_zerar_rodando(&mut self) -> Resultado<()> {
        self.conn.execute("UPDATE orquestrador SET rodando = 0 WHERE rodando <> 0", [])?;
        Ok(())
    }
}
