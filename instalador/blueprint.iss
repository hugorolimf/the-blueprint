; Instalador do The Blueprint (beta) (Inno Setup 6.3 ou mais novo). Para gerar:
;   cargo build --release
;   ISCC instalador\blueprint.iss        -> target\instalador\Blueprint-<versão>-instalador.exe
; Instala só para o usuário (sem pedir administrador); na tela inicial dá para escolher "para todos os usuários", que instala em Program Files. Os dados ficam em %APPDATA%\Blueprint\ e
; sobrevivem a atualizações; a desinstalação pergunta se quer apagá-los também.

#define Nome "The Blueprint (beta)"
#define Pasta "Blueprint"
#define Versao "0.1.0"
#define Exe "blueprint.exe"

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
; o app é de instância única (mutex nomeado): se estiver aberto, o instalador pede para fechar antes de trocar o arquivo
AppMutex=Local\Blueprint.InstanciaUnica

[Languages]
Name: "ptbr"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "atalhodesktop"; Description: "Criar um atalho na área de trabalho"; Flags: unchecked
Name: "iniciar"; Description: "Abrir o The Blueprint (beta) quando o Windows iniciar (fica na bandeja)"; Flags: unchecked
Name: "conectar"; Description: "Conectar o MCP ao Claude Code e ao Codex (os que estiverem instalados)"

[Files]
Source: "..\target\release\{#Exe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#Nome}"; Filename: "{app}\{#Exe}"
Name: "{autodesktop}\{#Nome}"; Filename: "{app}\{#Exe}"; Tasks: atalhodesktop
Name: "{autostartup}\{#Nome}"; Filename: "{app}\{#Exe}"; Tasks: iniciar

[Run]
; `blueprint registrar` roda `claude mcp add --scope user …` e `codex mcp add …` com o caminho instalado (de novo a cada atualização, sem duplicar).
; runasoriginaluser: numa instalação "para todos", registra para quem instalou, não para a conta de administrador.
Filename: "{app}\{#Exe}"; Parameters: "registrar"; StatusMsg: "Conectando ao Claude Code e ao Codex..."; Flags: runhidden runasoriginaluser; Tasks: conectar
Filename: "{app}\{#Exe}"; Description: "Abrir o The Blueprint (beta)"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; tira o servidor chamado `blueprint` dos dois agentes (o caminho dele sumiria com a desinstalação)
Filename: "{app}\{#Exe}"; Parameters: "registrar --remover"; Flags: runhidden; RunOnceId: "DesligarMcp"

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

function InitializeSetup: Boolean;
begin
  Result := True;
  if not WebView2Instalado then
    Result := MsgBox('O WebView2 Runtime (a parte do Windows que desenha a janela do The Blueprint) não foi encontrado.' + #13#10 + #13#10 +
      'Instale-o em https://go.microsoft.com/fwlink/p/?LinkId=2124703 e depois abra o The Blueprint (beta). Quer continuar a instalação mesmo assim?',
      mbConfirmation, MB_YESNO) = IDYES;
end;

{ Os dados do usuário (banco e protótipos) não são do instalador: só saem se a pessoa mandar. }
procedure CurUninstallStepChanged(Passo: TUninstallStep);
begin
  if Passo = usPostUninstall then
    if MsgBox('Apagar também os dados do The Blueprint (banco de dados e protótipos em %APPDATA%\Blueprint)?', mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES then
      DelTree(ExpandConstant('{userappdata}\Blueprint'), True, True, True);
end;
