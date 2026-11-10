# Arctic — relatório de implementação
Data: 2026-11-10. Escopo: pedido de animação, seleção mais controlada, réguas,
opções visuais e paletas Arctic exatas, com Dark padrão.

## Alterações
Desktop: theme.rs, animation.rs, viewport_tools.rs, browser e main. Workspace:
projeção ortográfica, ViewOptions, cálculo de réguas, snap com oclusão e sessão 4.
Sem dependência nova, mudança de assets ou calibração automática.
Contratos/limites: ADR 0007 e docs/desktop/ARCTIC.md.

- Dark/Light conforme cores fornecidas; estilos de botões, hover/foco animados,
  hierarquia visual e superfícies. Dark é padrão; preferências seguem sessão.
- Explosão/remontagem com easing e câmera interpolada, reversíveis; movimento
  reduzido. Interromper por navegação preserva estado visual corrente.
- Ortográfica/perspectiva, frontal/superior/lateral, eixos, réguas/barra de escala,
  grade de plano U/V em ortográfica e guias do cursor. Perspectiva explicita plano
  de referência; sem escala ou com escala desatualizada não inventar unidade.
- Prévia do snap, filtro de seleção, proteção contra faces oclusoras, entrada por
  índice original, coordenadas/deltas e formatação 0–9 casas decimais.
- Sessão 4, compatível na leitura com 1/2/3; tema, câmera e preferências validados.

## Verificação executada
Linux x86_64, Rust/Cargo 1.99.1, Xorg dummy :99, OpenGL Mesa software.

| Verificação | Resultado |
|---|---|
| cargo fmt --check | passou |
| cargo clippy --workspace --all-targets -- -D warnings | passou |
| cargo test --workspace --locked | 41 testes, 0 falhas |
| cargo build --release -p noah-desktop --locked | passou |
| Tema Dark e Light na janela | inspecionados; Light ortográfica salvo em sessão 4 |
| Explosão/remontagem | inspecionadas, incluindo frame intermediário a 33% |
| Vídeo nativo da transição | 60 frames, 1440×900, 20 fps, 3s, sem dados externos |
| Réguas físicas ortográficas | referência sintética 100 mm coincidiu com marcas 0/100 |
| Snap e retorno da janela | vértices 2/3 selecionados, 100 mm |
| Índices originais | índices 1/2 produziram ~244,949 mm; 2/3 restauraram 100 mm |
| Coordenadas/deltas | A=(0,1,0), B=(0,0,0) u. cena; ΔY=-100 mm |
| Gravação de preferências | sessão 4 com Dark/Light, projeção e movimento reduzido conferida |

Seis testes novos: paletas, easing/extremos/reversibilidade, ortográfica uniforme
em profundidade, régua/calibração desatualizada, sessão/legado/preferências inválidas,
e oclusão/filtro de peças. Trinta e cinco testes anteriores continuam passando.
Nenhuma afirmação de precisão industrial foi adicionada: são testes numéricos
com dados sintéticos e referência declarada, não ensaios de um instrumento.

## Evidências e limites
Capturas em docs/desktop/arctic-dark.png, arctic-light.png, reguas-calibradas.png e
arctic-evidence/. A captura Light final foi conferida no executável empacotado,
com grade ortográfica U/V. Estados de fontes não foram alterados.

Animação separa componentes já existentes; não fratura malhas/objetos únicos.
Renderer permanece CPU/OpenGL, amostragem 50k pontos/20k faces e oclusão aproximada.
Snap frontal é separado: verifica triângulos completos em f64 e até oito candidatos,
com limite de 200k faces visíveis. Não infere superfícies de nuvens. Hover é limitado
a atualizações de ~75ms e pode responder mais devagar em modelos grandes. Isolar
peças/filtro ajudam a controlar custo e ambiguidade. Índices originais continuam
sendo a forma determinística de selecionar vértices conhecidos.

Réguas medem projeção U/V; não o comprimento 3D de toda aresta. Em perspectiva a
escala vale no plano do alvo, não em todas as profundidades. Escala declarada não
confere exatidão física, e as casas decimais não são uma estimativa de incerteza.
Pontos de medição sempre usam estado montado, mesmo com linhas na vista explodida.

A duração nominal depende de frames disponíveis. Uma cena enorme pode exigir
movimento reduzido. Preferências são do workspace, não configuração global de SO.
Apps anteriores não abrem schema 4. Persistem limites E2 dense/CUDA, montagem
manual, fontes por caminhos absolutos e pacote Linux glibc 2.39+.
