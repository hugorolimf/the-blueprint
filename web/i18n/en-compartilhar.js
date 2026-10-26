// Inglês: o botão e o diálogo Compartilhar do app local (compartilhar.js). Chave = texto-fonte em português.
Object.assign(I18N.catalogos.en, {
  // equipes-locais.js: as equipes lembradas na tela de workspaces
  "Você entra como {nome}": "You join as {nome}", "Tirar da lista": "Remove from list",
  "Abrindo a equipe na janela do app…": "Opening the team in the app window…",
  // HTML (blueprint.html)
  "Compartilhar": "Share", "Compartilhar este workspace com outras pessoas pela internet": "Share this workspace with other people over the internet",
  // compartilhar.js
  "Baixando o túnel…": "Downloading the tunnel…",
  "Só na primeira vez: o app baixa o cloudflared, o programa da Cloudflare que cria o endereço público.": "First time only: the app downloads cloudflared, the Cloudflare program that creates the public address.",
  "Ligando o túnel…": "Starting the tunnel…", "Pedindo um endereço público à Cloudflare; leva alguns segundos.": "Asking Cloudflare for a public address; it takes a few seconds.",
  "Compartilhando «{nome}»": "Sharing “{nome}”",
  "«{nome}» não está compartilhado": "“{nome}” is not shared",
  "O túnel já está ligado para outro workspace. Ligue para incluir «{nome}» no mesmo endereço: cada convidado só vê o workspace do próprio convite.": "The tunnel is already on for another workspace. Turn it on to add “{nome}” to the same address: each guest only sees the workspace in their own invite.",
  "Quem tiver um link de convite entra por este endereço e só vê este workspace.": "Anyone with an invite link joins through this address and sees only this workspace.",
  "Não foi possível compartilhar": "Could not share", "O túnel não ligou. Tente de novo.": "The tunnel did not start. Try again.",
  "Compartilhamento desligado": "Sharing is off",
  "Ligue para criar um endereço público para «{nome}»: só este workspace fica visível para quem você convidar.": "Turn it on to create a public address for “{nome}”: only this workspace is visible to the people you invite.",
  "Ligar": "Turn on", "Desligar": "Turn off", "Ligando o compartilhamento": "Turning sharing on",
  "O app precisa ficar aberto enquanto as pessoas usam. O endereço muda a cada vez que você liga: depois de religar, mande os links de novo.": "The app must stay open while people use it. The address changes every time you turn it on: after turning it on again, send the links again.",
  "Esta versão do app não compartilha workspaces.": "This version of the app cannot share workspaces.",
  "Este é o endereço público. Quem não tem link de convite não entra.": "This is the public address. People without an invite link cannot join.",
  "Parar de compartilhar este workspace?": "Stop sharing this workspace?",
  "Quem usa este workspace perde o acesso na hora. Sem nenhum workspace compartilhado, o túnel desliga e o endereço muda ao ligar de novo.": "People using this workspace lose access right away. With no workspace shared, the tunnel turns off and the address changes when you turn it on again.",
  "Endereço": "Address", "Temporário": "Temporary", "Fixo": "Fixed",
  "Cloudflare, sem conta. O endereço muda a cada vez que você liga: depois de religar, mande os links de novo.": "Cloudflare, no account. The address changes every time you turn it on: after turning it on again, send the links again.",
  "Tailscale Funnel: o mesmo endereço sempre, então os links e as sessões continuam valendo. Precisa do Tailscale instalado e conectado, com HTTPS e o Funnel ligados na sua tailnet; o app não guarda senha nem chave.": "Tailscale Funnel: always the same address, so links and sessions keep working. Needs Tailscale installed and connected, with HTTPS and Funnel turned on for your tailnet; the app keeps no password or key.",
  "O app precisa ficar aberto enquanto as pessoas usam.": "The app has to stay open while people use it.",
  "O app precisa ficar aberto enquanto as pessoas usam. O endereço é fixo: não precisa mandar os links de novo.": "The app has to stay open while people use it. The address is fixed: you do not need to send the links again.",
  "Trocar o tipo de endereço muda o endereço: quem já entrou precisa de um link de convite novo uma vez (e o comando blueprint conectar do Claude dessa pessoa, rodado de novo). Depois, no fixo, o endereço não muda mais.": "Changing the address type changes the address: people who already joined need a new invite link once (and to run their Claude's blueprint conectar command again). After that, with the fixed address, it never changes again.",
});
