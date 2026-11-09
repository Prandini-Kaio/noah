# Modelo conceitual e invariantes

## Vocabulário
- **Project**: coleção persistida de objetos e assets com esquema versionado.
- **Object**: entidade física/conceitual com ID estável; distinta de suas representações.
- **Assembly**: objeto composto com filhos e transformações locais (MVP 04).
- **CaptureSource**: observações brutas (fotos, LiDAR, RGB-D, medições).
- **GeometryAsset**: representação 3D como nuvem de pontos ou malha; pode haver várias por objeto.
- **ReferenceFrame**: referencial com convenções conhecidas e transformações explícitas.
- **Measurement**: valor e unidade, fonte, método, incerteza quando disponível.
- **ProcessingRun**: registro de entradas, parâmetros, versão da ferramenta, status, logs, saídas.
- **Revision**: resultado derivado identificado, preservando versões anteriores.

## Invariantes
1. Todo objeto tem ID persistente; caminho de arquivo não é identidade de objeto.
2. Uma geometria não é um objeto e não é uma captura original.
3. Posição mundial é composição de transformações; não alterar pontos brutos quando basta transformar a instância.
4. Unidade desconhecida deve continuar **desconhecida**, não inferir metros ou milímetros.
5. `scale known` ≠ `measurement validated`; não prometer exatidão.
6. Coordenadas e transformações exigem especificação de handedness, convenção de matrizes e sistemas local/global antes de interoperar.
7. Cada derivado referencia as entradas que o geraram e sua versão de processamento.
8. O modelo de domínio não depende de um tipo particular de sensor.

## Questões em aberto (decidir por ADR no início de E1)
- Convenção canônica de eixos e ordem de matrizes.
- Formato interno do projeto e estratégia de migração.
- Precisão numérica para transformações e armazenamento.
- Limites operacionais para arquivos PLY de entrada.
