# Incremento autorizado — interface integrada e montagem inicial
Data: 2026-11-10. Pedido: continuar desenvolvimento, levar operações à interface,
compor objetos e disponibilizar vista explodida. Amplia E3 com recorte explícito E5;
não depende da implementação de medição E4 e mantém pendência CUDA de E2.

Plano: alterar noah-workspace (hierarquia, transformações compostas, explosão e
sessão schema 2 compatível com 1), desktop (explorador de arquivos integrado,
fluxo novo/abrir/salvar/exportar, reconstrução acompanhada e cancelável), demo
multiparte e testes de hierarquia, reversibilidade e persistência. Documentar ADR.

Aceite: realizar operações primárias sem digitar comandos; carregar demonstração
montada pela interface, alternar vista explodida, transformar grupo, salvar e
reabrir com mesma hierarquia. Reconstrução em background com estados e cancelamento.
Regressão E1/E2/E3; lint/testes; inspeção da janela com interações reais.

Limites: montagem manual, não engenharia/CAD; sem inferência automática de partes,
sem alinhamento multimodal e sem garantias métricas. Não implementar E4/E6/E7/E8.
Renderer mantém limite de amostragem e oclusão aproximada da E3.
