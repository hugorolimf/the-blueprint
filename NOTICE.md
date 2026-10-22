# Avisos de terceiros / Third-party notices

## Anatomy (kit isométrico e figuras)

As pastas `kit/` (`kit/iso-kit/`, `kit/lathe/`) e partes de `emblemas.mjs` e `agentes-figura.mjs` descendem do kit da skill
**Anatomy** (https://github.com/wheresryan22/anatomy), reorganizado em módulos menores. O aviso de licença original segue abaixo.

The `kit/` folder (`kit/iso-kit/`, `kit/lathe/`) and parts of `emblemas.mjs` and `agentes-figura.mjs` derive from the kit of the
**Anatomy** skill (https://github.com/wheresryan22/anatomy), split into smaller modules. The original license notice follows.

    MIT License
    
    Copyright (c) 2026 Ryan
    
    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:
    
    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.
    
    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

## ORCA

Algumas ideias (orquestração de agentes por MCP, painel de agentes) foram inspiradas no ORCA (https://github.com/stablyai/orca, MIT, Lovecast Inc.).
Nenhum código do ORCA foi copiado; se isso mudar, o aviso MIT dele deve entrar aqui.

Some ideas (agent orchestration over MCP, agents panel) were inspired by ORCA (https://github.com/stablyai/orca, MIT, Lovecast Inc.).
No ORCA code was copied; if that changes, its MIT notice must be added here.

## Dependências Rust / Rust dependencies

As crates listadas em `Cargo.toml` e `Cargo.lock` (`tiny_http`, `rusqlite`, `serde_json`, `flate2`, e no Windows `tao`, `wry`, `tray-icon`, `windows-sys`)
têm suas próprias licenças (MIT, Apache-2.0 ou equivalentes). Rode `cargo tree` para ver a lista completa.
