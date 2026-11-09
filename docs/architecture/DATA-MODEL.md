# Modelo de dados inicial (conceitual, não API congelada)

```text
Project { id, schema_version, name, objects[], assets[] }
Object { id, name, geometry_asset_ids[], local_transform, parent_id? }
GeometryAsset { id, kind, file_ref, coordinate_frame, unit_state, provenance }
PointCloud { positions[], colors?, normals? }
Transform { translation, rotation, scale_policy }
Provenance { source_refs[], tool?, parameters?, created_at? }
```

Para MVP 00, apenas objetos com nuvens de pontos precisam ser operacionais; demais representações são extensões previstas, não implementação obrigatória.

**UnitState:** `Unknown` ou unidade explicitamente definida/calibrada. Não inferir unidade do tipo de arquivo. Preferir precisão `f64` em transformações e cálculos; documentar conversões de armazenamento e renderização.

**Referenciais:** especificar orientação canônica, handedness, multiplicação e coluna/linha por ADR. Asset possui referencial de origem; objeto pode instanciá-lo com transformação sem alterar o asset.

**Versionamento:** esquema versionado + referência a hashes de arquivos brutos/derivados quando implementado; renomear objeto não altera sua identidade.

**Dados em disco (proposta revisável):** `project.json`, `raw/`, `derived/`. Evitar serializar milhões de pontos dentro de JSON.


## Contrato implementado E1 — schema 1
- core mantém Project, Object e GeometryAsset em memória; metadata/schema pertencem
  ao DTO de storage. Object referencia um asset e uma transformação de instância.
- Project/Object/Asset usam tipos de ID u128 distintos; serialização em hex de 32 dígitos.
- GeometryAsset contém PointCloud validada, UnitState e referencial destro Z-up.
- Persistência: project.json, raw/<asset-id>.ply, derived/<asset-id>.npc. Caminhos
  são derivados de IDs validados, nunca de caminhos arbitrários nos metadados.
- Proveniência persistida inclui nome da origem e SHA-256 do bruto e da nuvem.
  O formato NOAHPC01 identifica a versão de serialização da nuvem.
- Ao abrir, validar schema, IDs, referências, limites, hashes, coordenadas e
  correspondência entre a origem PLY e a nuvem interna. Não há migração automática.
- Transform é translação + rotação 3x3 própria + escala uniforme positiva.
  Euler XYZ e composição estão definidos no ADR 0002. Nada altera pontos brutos.
- Escrita concorrente e durabilidade contra queda de energia não são garantidas.
  O manifesto usa temporário + sync do arquivo + rename; diretório não tem fsync.
  Uma falha pode deixar arquivos órfãos, que não são removidos automaticamente.
