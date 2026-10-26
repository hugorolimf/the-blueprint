; Instalador do The Blueprint (beta) (Inno Setup 6.3 ou mais novo). Para gerar:
;   cargo build --release
;   ISCC instalador\blueprint.iss        -> target\instalador\Blueprint-<versão>-instalador.exe
; Instala só para o usuário (sem pedir administrador); na tela inicial dá para escolher "para todos os usuários", que instala em Program Files. Os dados ficam em %APPDATA%\Blueprint\ e
; sobrevivem a atualizações; a desinstalação pergunta se quer apagá-los também.

#define Nome "The Blueprint (beta)"
#define Pasta "Blueprint"
; a versão vem do Cargo.toml: `node scripts/gerar-release.mjs` passa ISCC /DVersao=<versão>; este valor só vale se alguém rodar o ISCC direto
#ifndef Versao
  #define Versao "0.1.0"
#endif
#define Exe "blueprint.exe"
; de onde vem o blueprint.exe; para outra pasta de build: ISCC /DOrigem=C:\caminho\release instalador\blueprint.iss
#ifndef Origem
  #define Origem "..\target\release"
#endif

[Setup]
AppId={{B5E9C6A2-3C1F-4D1E-9A57-5E2C7B1F8A40}
AppName={#Nome}
AppVersion={#Versao}
AppPublisher=The Blueprint
DefaultDirName={autopf}\{#Pasta}
DefaultGroupName={#Nome}
DisableProgramGroupPage=yes
OutputDir=..\target\instalador
OutputBaseFilename=Blueprint-{#Versao}-instalador
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
UninstallDisplayIcon={app}\{#Exe}
SetupIconFile=..\recursos\icone\icone.ico
; o app é de instância única (mutex nomeado): se estiver aberto, o instalador pede para fechar antes de trocar o arquivo
; só os testes (ISCC /DSemMutex=1) tiram a trava, para testar o instalador com o app do usuário aberto sem fechá-lo
#ifndef SemMutex
AppMutex=Local\Blueprint.InstanciaUnica
#endif
; sem o Restart Manager: ele não fecha os ajudantes sem janela que as sessões do Claude abrem (watch, hook statusline, --mcp) e o diálogo "incapaz de fechar" aparecia;
; quem os encerra é o PrepareToInstall (abaixo), e a janela do app continua protegida pelo AppMutex
CloseApplications=no

[Languages]
Name: "ptbr"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "atalhodesktop"; Description: "Criar um atalho na área de trabalho"; Flags: unchecked
Name: "iniciar"; Description: "Abrir o The Blueprint (beta) quando o Windows iniciar (fica na bandeja)"; Flags: unchecked
Name: "conectar"; Description: "Conectar ao Claude Code (plugin, MCP e o painel do Blueprint no terminal) e ao Codex (os que estiverem instalados)"

Name: "statusline"; Description: "Mostrar o Blueprint na status line do Claude Code (preserva a existente)"; Flags: unchecked

[Files]
Source: "{#Origem}\{#Exe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#Nome}"; Filename: "{app}\{#Exe}"
Name: "{autodesktop}\{#Nome}"; Filename: "{app}\{#Exe}"; Tasks: atalhodesktop
Name: "{autostartup}\{#Nome}"; Filename: "{app}\{#Exe}"; Tasks: iniciar

[Run]
; `blueprint registrar` instala o plugin `blueprint` (skill, agente, MCP, hooks) e o mod `blueprint-terminal` (marca e estado do blueprint no painel do terminal),
; ambos embutidos no .exe, e cai em `claude mcp add --scope user …` se o plugin falhar; também roda e `codex mcp add …` com o caminho instalado (de novo a cada atualização, sem duplicar).
; runasoriginaluser: numa instalação "para todos", registra para quem instalou, não para a conta de administrador.
Filename: "{app}\{#Exe}"; Parameters: "registrar"; StatusMsg: "Conectando ao Claude Code e ao Codex..."; Flags: runhidden runasoriginaluser; Tasks: conectar
Filename: "{app}\{#Exe}"; Parameters: "registrar --status-line"; StatusMsg: "Configurando a status line..."; Flags: runhidden runasoriginaluser; Tasks: statusline
; atualização feita pelo próprio app (blueprint atualizar / botão Atualizar): ele passa /ATUALIZACAO=1, sai para soltar o .exe, e o instalador o abre de novo no fim
Filename: "{app}\{#Exe}"; Flags: nowait runasoriginaluser; Check: AtualizacaoPeloApp
Filename: "{app}\{#Exe}"; Description: "Abrir o The Blueprint (beta)"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; tira o servidor chamado `blueprint` dos dois agentes (o caminho dele sumiria com a desinstalação)
Filename: "{app}\{#Exe}"; Parameters: "registrar --remover"; Flags: runhidden; RunOnceId: "DesligarMcp"

Filename: "{app}\{#Exe}"; Parameters: "registrar --status-line --remover"; Flags: runhidden; RunOnceId: "DesligarStatusline"

[Code]
const
  ChaveWebView2 = 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';

{ O WebView2 Runtime é o que desenha a janela. O Windows 11 já traz; no Windows 10 pode faltar. }
function WebView2Instalado: Boolean;
var
  Versao: String;
begin
  Result := RegQueryStringValue(HKLM32, ChaveWebView2, 'pv', Versao) or
            RegQueryStringValue(HKLM64, ChaveWebView2, 'pv', Versao) or
            RegQueryStringValue(HKCU, ChaveWebView2, 'pv', Versao);
  Result := Result and (Versao <> '') and (Versao <> '0.0.0.0');
end;

{ O app se atualiza chamando o instalador com /ATUALIZACAO=1 (src/atualizador/instalar.rs). }
function AtualizacaoPeloApp: Boolean;
begin
  Result := ExpandConstant('{param:ATUALIZACAO|0}') = '1';
end;

function InitializeSetup: Boolean;
begin
  Result := True;
  if not WebView2Instalado then
    Result := MsgBox('O WebView2 Runtime (a parte do Windows que desenha a janela do The Blueprint) não foi encontrado.' + #13#10 + #13#10 +
      'Instale-o em https://go.microsoft.com/fwlink/p/?LinkId=2124703 e depois abra o The Blueprint (beta). Quer continuar a instalação mesmo assim?',
      mbConfirmation, MB_YESNO) = IDYES;
end;

{ As sessões do Claude rodam o mesmo blueprint.exe sem janela (watch, hook statusline, --mcp) e o Windows trava o arquivo enquanto elas existem. Encerra só os desta
  pasta de instalação (outro blueprint.exe, de outra pasta, não é tocado); cada sessão do Claude reabre o dela. A janela do app já saiu: o AppMutex a exige fechada antes. }
procedure PararAjudantes;
var
  Exe, Comando: String;
  Codigo: Integer;
begin
  Exe := ExpandConstant('{app}\{#Exe}');
  StringChangeEx(Exe, '''', '''''', True);
  Comando := '-NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "Get-CimInstance Win32_Process | Where-Object { $_.Name -eq ''{#Exe}'' -and $_.ExecutablePath -ieq ''' + Exe +
    ''' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }"';
  Exec(ExpandConstant('{sys}\WindowsPowerShell\v1.0\powershell.exe'), Comando, '', SW_HIDE, ewWaitUntilTerminated, Codigo);
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  PararAjudantes;
  Result := '';
end;

{ Os dados do usuário (banco e protótipos) não são do instalador: só saem se a pessoa mandar. }
procedure CurUninstallStepChanged(Passo: TUninstallStep);
begin
  if Passo = usPostUninstall then
    if MsgBox('Apagar também os dados do The Blueprint (banco de dados e protótipos em %APPDATA%\Blueprint)?', mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES then
      DelTree(ExpandConstant('{userappdata}\Blueprint'), True, True, True);
end;
