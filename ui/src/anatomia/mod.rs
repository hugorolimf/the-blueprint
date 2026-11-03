//! O Anatomy do The Blueprint: o motor que desenha objetos em traço, dentro do app e em tempo real, com o tema da planta azul.
//! Nasceu do kit em JavaScript (`kit/iso-kit`, `kit/lathe`, ver `NOTICE.md`), que gera SVG parado com a câmera fixa. Aqui as mesmas ideias viram motor:
//! - a **câmera** gira, chega perto e ganha perspectiva, então dá para mergulhar numa peça (`camera.rs`);
//! - a **malha** guarda vértices e faces, e a cada quadro acha o contorno e os vincos como o kit acha (`malha.rs`, `solidos.rs`);
//! - a **luz** é em faixas, não degradê, para continuar parecendo desenho (`tinta.rs`);
//! - a **cena** ordena de trás para a frente e responde qual peça está sob o mouse (`cena.rs`).
//! O tempo é a quarta dimensão: quem monta a cena decide como cada peça está naquele instante e entrega o sólido já na altura certa.
//! O guia de uso está em `skills/blueprint-anatomia/SKILL.md` (e `.claude/skills/` aponta para lá).

mod camera;
mod cena;
mod malha;
mod solidos;
mod tinta;
mod vetor;

pub use camera::{viva as camera_viva, Camera, AZIMUTE, ELEVACAO};
pub use cena::{Camada, Cena, Gravado};
pub use malha::{imagem_na_face, Malha};
pub use solidos::Ponta;
pub use tinta::Tinta;
pub use vetor::V3;
