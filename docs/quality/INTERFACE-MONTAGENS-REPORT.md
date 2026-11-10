# Interface integrada e montagem inicial — relatório
Data: 2026-11-10. Escopo autorizado: evolução E3 e recorte explícito E5.

## Entregue
- App com Novo/Abrir workspace, Importar peças, Abrir projeto/run, Salvar revisão,
  Exportar peça e Reconstruir fotos. Explorador de arquivos embutido, sem digitar
  comandos para essas operações. Sessão de workspace é o projeto de cena E3;
  não altera automaticamente o projeto de armazenamento E1.
- Reconstrução em background com escolha de fotos/motor/destino, timeout, modo
  sparse/dense, etapas, logs, cancelamento e abertura do resultado na cena.
- Montagens hierárquicas (até 64 níveis), atribuição de pai, transformação de
  grupo e seleção de descendentes. Validação e rollback em ciclos/pais inválidos.
- Vista explodida radial reversível, intensidade e distância numéricas. Não
  altera posição montada ou geometria. Exportação ignora a explosão e grava
  metadados de unidade/referencial ao lado do arquivo.
- Demo sintética com dez componentes em três montagens (base/suportes e eixo/
  acionamento sob um conjunto raiz). Fontes embarcadas no binário e materializadas
  em pasta escolhida, para sessões reabríveis.
- Sessão schema 2 com grupos/pais/explosão e nomes de peças, leitura compatível
  com schema 1. Sem migração do schema E1 ou dos runs E2.

## Arquivos e decisões
workspace: assembly.rs, demo.rs, lib.rs, session.rs e testes de montagem;
desktop: browser.rs, jobs.rs e main.rs; dados em examples/assembly; empacotamento
em scripts/desktop/package-linux.sh. Contrato: ADR 0005. Sem E4/E6/E7/E8.
Código continua local-first e sem uploads de fotos. O motor é executável externo
confiável, escolhido explicitamente, usando os contratos seguros de E2.

## Verificações reais
Linux x86_64, Rust 1.99.1, Xorg dummy :99, OpenGL Mesa software.

| Verificação | Resultado |
|---|---|
| cargo fmt --check | passou |
| cargo clippy --workspace --all-targets -- -D warnings | passou |
| cargo test --workspace --locked | 30 testes passaram, 0 falhas |
| cargo build --release -p noah-desktop --locked | passou |
| Criar demo pela interface | dez peças / três montagens abertas |
| Alternar Montada / Explodida | executado e inspecionado na janela real |
| Salvar / reabrir workspace pela interface | schema 2, grupos, seleção e explosão restaurados |
| Confirmar descarte de ajustes antes de abrir | diálogo exibido e aceito no teste |
| Escolher fotos, motor e destino pela interface | executado no explorador integrado |
| Iniciar reconstrução COLMAP real pela interface | 12/12 imagens, 7613 pontos; status Succeeded |
| Abrir resultado na cena pela interface | executado, fonte adicionada |
| Fontes da montagem / fotos originais | SHA-256 conferidos e preservados |

Run de validação: 9f3cd297-039b-4c1f-80ba-586a68c709e0. Tempo observado: 76.540s, medição
pontual sem garantia de desempenho. A geração SfM não é bit a bit determinística.
Cenário sintético E2 existente, sem fotos externas. Cancelamento é coberto pelos
testes de processo E2; não foi acionado no teste interativo desta reconstrução.

Os 30 testes incluem regressões E1/E2/E3, quatro novos testes de montagem/
exportação/compatibilidade e um teste do refresh terminal do job. Durante a
inspeção foram corrigidos o explorador atrás da janela de reconstrução, o salto
de altura entre pastas e o status intermediário que podia ficar desatualizado
após a conclusão. O último tem regressão automatizada específica.

## Distribuição Linux
Prévia compilada x86_64 GNU, requer glibc >=2.39, X11 ou XWayland, X11/xkbcommon
carregáveis e OpenGL. Conferido com objdump/ldd e janela real em Linux. O binário
não é universal para toda distribuição; não foi validado em Linux antigo,
Windows/macOS ou Wayland nativo. COLMAP não acompanha o pacote e continua
requerendo instalação separada 3.11.x. Densificação real continua pendente de CUDA.
O executável empacotado NOAH foi executado separadamente em pasta de teste,
sem argumentos; a demonstração embarcada foi criada e montada/explodida pela
interface. Capturas reais em docs/desktop/montagem.png e vista-explodida.png.
Abrir o arquivo NOAH extraído dispensa Rust/Cargo e comandos para o uso diário.
Instruções em docs/desktop/LEIA-ME-LINUX.txt.

## Limites e pendências
Montagem é manual e visual, sem reconhecimento de peças ocultas, CAD, restrições
mecânicas ou detecção de colisões. Atribuir pai mantém valores locais e pode mudar
posição mundial. Cada fonte tem os limites de importação anteriores; instâncias
repetidas do mesmo arquivo e vínculo editável direto com assets E1 ainda não são
implementados. Renderer usa ordenação aproximada e até 50k pontos/20k faces.
As fontes permanecem em caminhos absolutos; workspace salvo não é autocontido.
Arquivos exportados não sobrescrevem existentes. Se a escrita dos metadados de
exportação falhar após salvar geometria, pode restar um arquivo parcial de entrega.

O explorador lista até mil entradas por pasta, omite symlinks e permite navegação
por pastas; para diretórios muito grandes, informar uma subpasta no campo de pasta.
Metadados de status são lidos localmente sem rehash completo para evitar bloquear
a interface; abertura do resultado usa validação de integridade do loader E2.
Reconstrução continua se a janela de acompanhamento for fechada. Fechar o app
com job ativo exige solicitar cancelamento e aguardar; kill externo continua
sujeito às limitações de E2. Há um escritor por workspace/run.

Próximos refinamentos possíveis: instâncias repetidas, importação OBJ mais ampla,
projetos portáveis e renderer com depth buffer, após avaliação do uso da prévia.
