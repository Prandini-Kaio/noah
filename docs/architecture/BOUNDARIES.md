# Fronteiras arquiteturais

**Direção permitida:** UI/CLI → serviços de aplicação → domínio + geometria + persistência; adapters externos → contratos internos. Nunca domínio → UI/COLMAP/LiDAR.

## Responsabilidades
- `core`: tipos estáveis de Project, Object, AssetId, revisão e relações; não conhece arquivos PLY.
- `geometry`: PointCloud, Mesh (ainda que importação OBJ fique para uma entrega posterior), transformações, referenciais, validação matemática; não acessa disco.
- `storage`: lê/grava projeto e assets, versão de schema, migração futura, atomicidade quando viável.
- `apps/cli`: comandos de alto nível, erros legíveis, diagnósticos e códigos de saída.
- `pipeline` (E2): orquestra jobs, checkpoints, cache e parâmetros.
- `photogrammetry` (E2): traduz entradas/saídas de COLMAP, sem contaminar tipos de domínio.
- `renderer` (E3): renderiza assets; não controla verdade dos dados.

## Preparação para sensores futuros
Definir no domínio proveniência, atributos opcionais, referenciais e múltiplos assets. **Não implementar** traits genéricas especulativas para 10 sensores na E1. Formalizar interface de adaptadores somente em E2 com o primeiro caso concreto.

## Persistência
Separar `project metadata` de `raw/` e `derived/`. O projeto aponta para assets por IDs e caminhos controlados, não por ponteiros de memória. Especificar versão do schema e garantir reabertura de dados. Não apagar arquivos sem intenção explícita.

## Não funcionalidades de E1
Reconstrução por fotos, renderização 3D, editor de montagem, medições industriais, LiDAR, CAD, networking.


## Fronteiras implementadas em E1
CLI depende de core, geometry e storage. Storage depende de core/geometry e possui
PLY, DTOs de persistência, proveniência e hashing. Core depende só de geometry;
geometry não faz I/O nem importa serialização. Não existem crates vazias para
adapters futuros. schema_version é parte do manifesto de storage; a entidade
Project em memória não conhece o formato físico do projeto.


## Fronteiras implementadas em E2
CLI chama photogrammetry; photogrammetry constrói os comandos do COLMAP, converte
TXT para core::reconstruction e exporta com o encoder PLY de storage. Pipeline
não conhece COLMAP: executa etapas estruturadas, snapshot, status e hashes.
Core::reconstruction contém Camera, CameraView e Reconstruction sem I/O/serde.
As convenções de poses e as fórmulas/modelos de intrínsecos exportados são
documentadas no adaptador; intrínsecos mantêm o nome e a ordem de parâmetros
suportados pelo COLMAP 3.11. Nenhum tipo de libcolmap é importado pelo domínio.
Run schema 1 é separado de Project schema 1. A E2 não atribui referencial
canônico nem calibração ao resultado e não altera a origem PLY exigida por E1.

## Incremento de interface e montagens
Desktop orquestra noah-photogrammetry em thread de trabalho e lê status local
sem revalidar fotos a cada frame; a abertura final valida os artefatos.
Workspace compõe hierarquia e deslocamentos de explosão e persiste sessão 2;
domínio geométrico permanece independente da UI. Projetos E1/runs E2 não mudam
de schema. Explosão afeta apenas visualização; export de peça usa estado montado.

## Estilhaços e organização Arctic
Desktop `blast` mantém apenas um cache visual de fragmentos. Não grava malhas,
objetos ou medidas; usa transforms e primitives do workspace, sem dependência
inversa. `shell` organiza navegação e contexto; domínio/sessão schema 4 não mudam.
Fratura volumétrica/colisões não fazem parte deste efeito. Contrato: ADR 0008.

## Continuação E4: ângulos e dimensões
Workspace/inspection calcula ângulo ABC e AABB montada sem I/O ou UI. Sessão 5
persiste definições e lê 1–4; relatório 2 inclui transformações/proveniência.
Desktop/metrology reúne formulários, anotações e cache descartável de caixas,
invalidado por geometria, transformações e hierarquia. Contrato: ADR 0009.

## Alinhamento E5 inicial e kit
Workspace/alignment planeja/aplica translação por duas âncoras com contexto guardado;
UI mantém preview/undo temporários, sem alterar assets. Ajustes aplicados persistem
nos Placements da sessão 5. Desktop usa exemplos locais empacotados; reconstrução
continua via contrato COLMAP existente. Kit não atribui escala nem representa malha
sintética como derivada da reconstrução. ADR 0010.

## Superfície E2 / sessão 6
Adapter mantém malha PLY bruta e normalização OBJ com proveniência. Storage
valida PLY COLMAP limitado; workspace usa RunSurface distinto de Run, preservando
identidade e âncoras de nuvens antigas. Sessão 6 lê 1–5; relatório 3 inclui novo
tipo de fonte. Derivação copia modelos/fotos registrados para novo run sem alterar
o pai. Delaunay CPU validado; dense CUDA completo continua pendente. O kit agora
inclui malha real pré-processada, com origem explícita, sem textura ou escala.
