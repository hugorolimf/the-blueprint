//! O ciclo em segundo plano: ver se há release nova, baixar o instalador, conferir o SHA-256 e deixá-lo pronto. Nada aqui executa o que foi baixado
//! (isso é `instalar.rs`, só com o clique do usuário) e nada aqui roda em laço de repouso: uma verificação ao abrir o app (se passaram 24 h) e as que o usuário pede.
use super::estado::{Fase, Info};
use super::versao::mais_nova;
use super::{hash, rede, release::{self, Release}, Atualizador};
use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};

/// O nome do arquivo vem da release (de fora): só letras, números, `.`, `-` e `_`, para nunca sair da pasta do updater.
fn nome_seguro(nome: &str) -> bool {
    !nome.is_empty() && nome.len() <= 100 && !nome.contains("..") && nome.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

impl Atualizador {
    /// Ao abrir o app (janela): recupera um instalador já baixado e, se passaram 24 h desde a última verificação, verifica em segundo plano.
    pub fn ao_abrir(&self) {
        self.reconstruir();
        if !self.automatica() {
            self.mudar(|e| {
                if e.fase == Fase::Parada {
                    e.fase = Fase::Desligada;
                }
            });
        } else if self.passou_o_intervalo(crate::tempo::agora_ms()) {
            self.verificar_em_segundo_plano(false);
        }
    }

    /// Uma thread para a verificação (e o download, se houver versão nova). `manual`: o usuário pediu, vale mesmo com a verificação automática desligada.
    pub fn verificar_em_segundo_plano(&self, manual: bool) {
        let a = self.clone();
        std::thread::spawn(move || a.verificar(manual));
    }

    /// Verifica, baixa e confere, na thread de quem chama (o `blueprint atualizar` usa direto). Uma só por vez: se já há uma em curso, não faz nada.
    pub fn verificar(&self, manual: bool) {
        if self.i.em_curso.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return;
        }
        let resultado = self.ciclo(manual);
        self.i.em_curso.store(false, Ordering::SeqCst);
        if let Err(erro) = resultado {
            if self.fase() != Fase::Pronta {
                self.mudar(|e| e.fase = Fase::Erro(erro)); // com um instalador já pronto, uma falha de rede não apaga o aviso
            }
        }
    }

    /// Só consulta, sem baixar nada: a release mais nova que a instalada, se houver (`blueprint atualizar --verificar`).
    pub fn consultar(&self) -> Result<Option<Release>, String> {
        let lista = rede::obter_texto(&self.i.origem, &format!("{}/releases?per_page=10", self.i.origem.api))?;
        release::escolher(&lista, &self.i.atual, self.i.so_estaveis)
    }

    /// Recupera do disco um instalador já baixado e conferido (o que o app faz ao abrir), para quem usa o updater sem o app aberto.
    pub fn recuperar(&self) {
        self.reconstruir();
    }

    fn ciclo(&self, manual: bool) -> Result<(), String> {
        if !manual && !self.automatica() {
            self.mudar(|e| {
                if matches!(e.fase, Fase::Parada | Fase::EmDia) {
                    e.fase = Fase::Desligada;
                }
            });
            return Ok(());
        }
        if self.fase() != Fase::Pronta {
            self.mudar(|e| {
                e.fase = Fase::Verificando;
                e.motivo = None;
            });
        }
        let lista = rede::obter_texto(&self.i.origem, &format!("{}/releases?per_page=10", self.i.origem.api))?;
        match release::escolher(&lista, &self.i.atual, self.i.so_estaveis)? {
            Some(release) => self.baixar(&release)?,
            None => {
                if self.fase() != Fase::Pronta {
                    self.mudar(|e| *e = Info { fase: Fase::EmDia, ..Info::default() });
                }
                self.gravar_verificacao();
            }
        }
        Ok(())
    }

    /// Já está pronta exatamente esta versão (arquivo no disco com o hash certo)? Então não baixa de novo.
    fn ja_pronta(&self, versao: &str) -> bool {
        let e = self.i.estado.lock().unwrap_or_else(|e| e.into_inner());
        e.fase == Fase::Pronta && e.versao.as_deref() == Some(versao) && e.arquivo.as_ref().zip(e.sha256.as_ref()).is_some_and(|(a, h)| hash::confere(a, h))
    }

    fn baixar(&self, release: &Release) -> Result<(), String> {
        if self.ja_pronta(&release.versao) {
            self.gravar_verificacao();
            return Ok(());
        }
        let origem = &self.i.origem;
        if !origem.permite(&release.instalador.url) || !origem.permite(&release.sha256sums.url) {
            return Err("a release aponta para um endereço de download fora da lista permitida; nada foi baixado".into());
        }
        if !nome_seguro(&release.instalador.nome) {
            return Err("o nome do instalador na release é inválido; nada foi baixado".into());
        }
        let sums = rede::obter_texto(origem, &release.sha256sums.url)?;
        let esperado = hash::procurar(&sums, &release.instalador.nome).ok_or("o SHA256SUMS da release não tem a linha do instalador; nada foi instalado")?;
        std::fs::create_dir_all(&self.i.pasta).map_err(|e| format!("não consegui criar a pasta da atualização: {e}"))?;
        let (destino, parcial) = (self.i.pasta.join(&release.instalador.nome), self.i.pasta.join(format!("{}.parcial", release.instalador.nome)));
        let _ = std::fs::remove_file(&parcial);
        let (versao, notas) = (release.versao.clone(), release.notas.clone());
        self.mudar(|e| *e = Info { fase: Fase::Baixando(0), versao: Some(versao), notas, ..Info::default() });
        let (total, ultimo) = (release.instalador.tamanho.max(1), AtomicU8::new(0));
        rede::baixar(origem, &release.instalador.url, &parcial, rede::MAX_INSTALADOR, &|bytes| {
            let p = (bytes * 100 / total).min(99) as u8;
            if ultimo.swap(p, Ordering::Relaxed) != p {
                self.mudar(|e| e.fase = Fase::Baixando(p));
            }
        })?;
        if !hash::confere(&parcial, &esperado) {
            let _ = std::fs::remove_file(&parcial);
            return Err("o instalador baixado não confere com o SHA-256 da release; o arquivo foi descartado".into());
        }
        std::fs::rename(&parcial, &destino).map_err(|e| format!("não consegui guardar o instalador: {e}"))?;
        let pronta = Info { fase: Fase::Pronta, versao: Some(release.versao.clone()), notas: release.notas.clone(), arquivo: Some(destino.clone()), sha256: Some(esperado), motivo: None };
        self.gravar_pronta(&pronta);
        self.limpar_antigos(Some(&destino));
        self.gravar_verificacao();
        self.mudar(|e| *e = pronta);
        Ok(())
    }

    /// Ao abrir: se um download anterior terminou (`pronta.json` e o arquivo com o hash certo) e a versão ainda é mais nova que esta, volta a ficar pronta
    /// sem tocar na rede. Qualquer outra coisa na pasta é lixo e sai.
    pub(super) fn reconstruir(&self) {
        let Some(info) = self.ler_pronta() else { return self.limpar_antigos(None) };
        let valida = info.versao.as_deref().is_some_and(|v| mais_nova(&self.i.atual, v))
            && info.arquivo.as_ref().zip(info.sha256.as_ref()).is_some_and(|(a, h)| a.starts_with(&self.i.pasta) && hash::confere(a, h)); // o caminho precisa ser da pasta do updater
        if valida {
            self.limpar_antigos(info.arquivo.as_deref());
            self.mudar(|e| *e = info);
        } else {
            self.apagar_pronta();
            self.limpar_antigos(None);
        }
    }

    /// Apaga instaladores de versões antigas e downloads pela metade, menos `manter`.
    pub(super) fn limpar_antigos(&self, manter: Option<&Path>) {
        for entrada in std::fs::read_dir(&self.i.pasta).into_iter().flatten().flatten() {
            let (caminho, nome) = (entrada.path(), entrada.file_name().to_string_lossy().into_owned());
            if (nome.starts_with("Blueprint-") && (nome.ends_with("-instalador.exe") || nome.ends_with(".parcial"))) && manter != Some(caminho.as_path()) {
                let _ = std::fs::remove_file(caminho);
            }
        }
    }
}
