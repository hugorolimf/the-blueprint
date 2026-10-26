//! O pseudoconsole do Windows (ConPTY): roda um programa num console invisível e devolve os dois canos (o que ele escreve e o que ele lê).
//! Chamadas diretas ao kernel32 pelo `windows-sys` que o app já usa (nenhuma crate nova). O processo nasce suspenso e entra num Job Object com
//! KILL_ON_JOB_CLOSE antes de rodar: fechar a sessão (ou o app cair) derruba a árvore inteira (claude, node, git…), sem processo órfão.
use std::ffi::c_void;
use std::fs::File;
use std::io;
use std::os::windows::io::FromRawHandle;
use std::path::Path;
use std::ptr::{null, null_mut};
use std::sync::Arc;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::Console::{ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole, COORD, HPCON};
use windows_sys::Win32::System::JobObjects::{AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, GetExitCodeProcess, InitializeProcThreadAttributeList, ResumeThread, UpdateProcThreadAttribute, WaitForSingleObject, CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, INFINITE,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, STARTF_USESTDHANDLES, STARTUPINFOEXW,
};

/// O processo, dividido entre quem espera ele terminar e o controle da sessão; o handle fecha quando os dois soltam.
pub struct Processo(HANDLE);
// SAFETY: um HANDLE de processo pode ser usado de qualquer thread; só é fechado no Drop, quando ninguém mais o tem.
unsafe impl Send for Processo {}
unsafe impl Sync for Processo {}

impl Processo {
    /// Dorme (sem CPU) até o processo sair e devolve o código de saída.
    pub fn esperar(&self) -> Option<u32> {
        let mut codigo = 0u32;
        // SAFETY: o handle é válido enquanto `self` existir.
        unsafe { (WaitForSingleObject(self.0, INFINITE) == 0 && GetExitCodeProcess(self.0, &mut codigo) != 0).then_some(codigo) }
    }
}

impl Drop for Processo {
    fn drop(&mut self) {
        // SAFETY: handle próprio, fechado uma vez.
        unsafe { CloseHandle(self.0) };
    }
}

/// O pseudoconsole e o job da sessão. Soltar o `Controle` encerra tudo: o job derruba a árvore e o pseudoconsole fecha o cano de saída
/// (quem lê recebe o fim do arquivo).
pub struct Controle {
    hpc: HPCON,
    job: HANDLE,
}
// SAFETY: HPCON e o handle do job valem em qualquer thread; o Controle fica atrás de um Mutex na sessão.
unsafe impl Send for Controle {}

impl Controle {
    pub fn redimensionar(&self, colunas: i16, linhas: i16) -> bool {
        // SAFETY: `hpc` vale até o Drop.
        unsafe { ResizePseudoConsole(self.hpc, COORD { X: colunas, Y: linhas }) == 0 }
    }
}

impl Drop for Controle {
    fn drop(&mut self) {
        // SAFETY: handles próprios, fechados uma vez. O cano de saída continua sendo lido por outra thread, então ClosePseudoConsole não trava.
        unsafe {
            CloseHandle(self.job);
            ClosePseudoConsole(self.hpc);
        }
    }
}

pub struct Aberto {
    pub controle: Controle,
    pub processo: Arc<Processo>,
    /// o que o programa escreve na tela
    pub saida: File,
    /// o que chega ao teclado do programa
    pub entrada: File,
}

fn largo(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn erro(o_que: &str) -> String {
    format!("{o_que}: {}", io::Error::last_os_error())
}

/// Fecha os handles que sobrarem se algo falhar no meio da abertura.
struct Sobra(Vec<HANDLE>);
impl Drop for Sobra {
    fn drop(&mut self) {
        // SAFETY: só entram aqui handles abertos por `abrir` e ainda não entregues a ninguém.
        self.0.iter().filter(|h| !h.is_null()).for_each(|h| unsafe {
            CloseHandle(*h);
        });
    }
}

/// Abre `linha` (linha de comando já citada, como o CreateProcessW espera) em `pasta`, num pseudoconsole de `colunas` x `linhas`.
pub fn abrir(linha: &str, pasta: &Path, colunas: i16, linhas: i16) -> Result<Aberto, String> {
    let (mut entrada_leitura, mut entrada_escrita, mut saida_leitura, mut saida_escrita): (HANDLE, HANDLE, HANDLE, HANDLE) = (null_mut(), null_mut(), null_mut(), null_mut());
    // SAFETY: chamadas Win32 com ponteiros para variáveis locais; cada handle aberto vai para `sobra` até ter dono.
    unsafe {
        if CreatePipe(&mut entrada_leitura, &mut entrada_escrita, null(), 0) == 0 || CreatePipe(&mut saida_leitura, &mut saida_escrita, null(), 0) == 0 {
            let _ = Sobra(vec![entrada_leitura, entrada_escrita, saida_leitura, saida_escrita]);
            return Err(erro("CreatePipe"));
        }
        let mut hpc: HPCON = 0;
        let hr = CreatePseudoConsole(COORD { X: colunas, Y: linhas }, entrada_leitura, saida_escrita, 0, &mut hpc);
        // o pseudoconsole duplicou as pontas dele: as nossas cópias fecham já (senão a leitura nunca recebe o fim)
        drop(Sobra(vec![entrada_leitura, saida_escrita]));
        let mut sobra = Sobra(vec![entrada_escrita, saida_leitura]);
        if hr != 0 {
            return Err(format!("CreatePseudoConsole: HRESULT {hr:#x}"));
        }
        let mut tamanho = 0usize;
        InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut tamanho);
        let mut lista_buf = vec![0u8; tamanho];
        let lista = lista_buf.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;
        let fechar_hpc = || ClosePseudoConsole(hpc);
        if InitializeProcThreadAttributeList(lista, 1, 0, &mut tamanho) == 0 {
            fechar_hpc();
            return Err(erro("InitializeProcThreadAttributeList"));
        }
        let ok = UpdateProcThreadAttribute(lista, 0, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize, hpc as *const c_void, std::mem::size_of::<HPCON>(), null_mut(), null());
        let mut si: STARTUPINFOEXW = std::mem::zeroed();
        si.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
        // handles padrão nulos: sem isto o filho herdaria os canais redirecionados do app em vez do pseudoconsole
        si.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        si.lpAttributeList = lista;
        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();
        let mut cmd = largo(linha);
        let cwd = largo(&pasta.to_string_lossy());
        let criado = ok != 0 && CreateProcessW(null(), cmd.as_mut_ptr(), null(), null(), 0, EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT | CREATE_SUSPENDED, null(), cwd.as_ptr(), &si.StartupInfo, &mut pi) != 0;
        let falha_criar = (!criado).then(|| erro(if ok == 0 { "UpdateProcThreadAttribute" } else { "CreateProcessW" }));
        DeleteProcThreadAttributeList(lista);
        if let Some(e) = falha_criar {
            fechar_hpc();
            return Err(e);
        }
        let processo = Arc::new(Processo(pi.hProcess));
        let job = CreateJobObjectW(null(), null());
        let mut limite: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        limite.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let no_job = !job.is_null()
            && SetInformationJobObject(job, JobObjectExtendedLimitInformation, &limite as *const _ as *const c_void, std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32) != 0
            && AssignProcessToJobObject(job, pi.hProcess) != 0;
        let controle = Controle { hpc, job };
        if !no_job {
            let e = erro("Job Object");
            drop(controle); // o job (vazio ou sem limite) fecha; o processo suspenso morre junto com o pseudoconsole
            windows_sys::Win32::System::Threading::TerminateProcess(pi.hProcess, 1);
            CloseHandle(pi.hThread);
            return Err(e);
        }
        ResumeThread(pi.hThread);
        CloseHandle(pi.hThread);
        sobra.0.clear();
        Ok(Aberto { controle, processo, saida: File::from_raw_handle(saida_leitura), entrada: File::from_raw_handle(entrada_escrita) })
    }
}
