# Dependências de terceiros — E1/E2

Inventário de Cargo.lock para o alvo Linux x86_64 validado neste ambiente.
Licenças declaradas pelos pacotes; textos fornecidos pelos pacotes estão em
third-party-licenses/. O código original do projeto é MIT (ver LICENSE). Dataset sintético: CC0-1.0.
E2 executa COLMAP local opcional; não há serviço de captura nem telemetria.

| Crate | Versão | SPDX declarado |
|---|---|---|
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| cfg_aliases | 0.2.2 | MIT |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 |
| digest | 0.10.7 | MIT OR Apache-2.0 |
| generic-array | 0.14.7 | MIT |
| getrandom | 0.4.3 | MIT OR Apache-2.0 |
| itoa | 1.0.19 | MIT OR Apache-2.0 |
| libc | 0.2.191 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| nix | 0.30.1 | MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| sha2 | 0.10.9 | MIT OR Apache-2.0 |
| syn | 3.0.10 | MIT OR Apache-2.0 |
| typenum | 1.20.1 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| uuid | 1.29.2 | Apache-2.0 OR MIT |
| version_check | 0.9.5 | MIT/Apache-2.0 |
| zmij | 1.0.23 | MIT |

Dependências para outros alvos presentes no lockfile não foram validadas;
revisar novamente ao distribuir para Windows/macOS ou outros alvos.

## Motor externo e gerador de teste
COLMAP 3.11.1 (BSD-3-Clause), distribuição conda-forge CPU para Linux usada no
workspace. Binários não são incluídos no projeto. Seu ambiente inclui outras
licenças (inclusive componentes GPL/LGPL); revisar a distribuição separadamente.
Inventário da instalação de teste: e2-engine-packages.json, com URLs, versões,
hashes e licenças declaradas dos pacotes resolvidos. Não é promessa de pacote
COLMAP redistribuível com qualquer licença. Referência: https://colmap.github.io/.

Gerador sintético de teste usa NumPy 2.3.5 (BSD-3-Clause) e Pillow 13.0.0
(MIT-CMU), já disponíveis neste ambiente. Não são dependências runtime do NOAH.
Cenas/texturas são geradas no projeto e licenciadas como CC0-1.0.
