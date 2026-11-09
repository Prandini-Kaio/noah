# Qualidade e Definition of Done

## Gates por mudança
- Testes para casos felizes e falhas (entradas ausentes, inválidas, grandes demais).
- Segurança de memória nativa; preferir safe Rust; `unsafe` apenas isolado e justificado.
- Mensagens de erro compreensíveis; evitar `unwrap()` em parsing/caminhos de produção.
- Formatação, lint e testes: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` quando executáveis.
- Documentação de API para contratos públicos e atualização dos docs alterados.
- Arquivos originais não são sobrescritos; recuperação de falhas de gravação é considerada.
- Licenças de dados, ferramentas e dependências identificadas.

## Testes geométricos mínimos
- Identidade, translação, rotação conhecida e composição.
- Conversão de unidades explícita, sem inferência.
- Precisão de ida e volta de serialização dentro de tolerância declarada.
- Nuvem vazia, entradas NaN/Inf, cores incompletas e arquivos PLY inválidos.

## Evidência de aceite
Relatório contendo ambiente, commit, comandos executados e seus resultados reais, dataset, medidas de integridade e limitações. Não dizer 'pronto' quando só compila.

## Reprodutibilidade
Artefatos processados têm origem, configurações e versão associadas quando a funcionalidade existir; datasets de teste têm procedência e autorização. Sem conexão remota silenciosa.
