---
name: pipeline-adapters
description: Planeja/implementa integrações de processamento de capturas, como COLMAP e futuramente LiDAR; usar somente a partir de E2 aprovado.
---
# Skill: adaptadores de pipeline (E2+)

- **Não executar esta skill na E1.**
- Defina contratos a partir do primeiro motor concreto, sem especulação para futuros sensores.
- Pipeline registra entradas, parâmetros, versão do motor, logs, status e artefatos.
- Execute processos externos com argumentos estruturados, timeout/abort e verificação de saída; nunca concatenar shell com entrada do usuário.
- Parseie resultados para tipos internos, isolando formatos de fornecedor.
- Suporte falhas parciais, diretórios temporários e não substituição silenciosa de resultados.
- Teste com dataset reprodutível e instalação explícita do motor.
