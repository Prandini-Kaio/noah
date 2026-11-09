# Contexto de produto — NOAH 2.0

A versão do produto é 2.0; o schema persistido e os crates possuem versionamento técnico independente.

## Problema
Ferramentas de fotogrametria produzem geometria, mas um fluxo de engenharia reversa exige algo mais: múltiplas representações do mesmo objeto, escala, fontes de medição, montagem de peças, histórico e novas capturas.

## Público inicial
Usuário técnico individual em desktop local, capaz de fotografar objetos e importar datasets. Colaboração em rede e SaaS estão fora do escopo atual.

## Jornada-alvo
Criar projeto → adicionar captura → executar reconstrução → inspecionar pontos/malha → estabelecer escala → construir/editar → decompor/montar → anexar nova captura → comparar revisões.

## Restrições
Objetos ocultos exigem novas capturas ou modelagem manual. Alinhamento multimodal não é automático em todos os casos. Não assumir tolerâncias industriais por aparência da geometria. No MVP 00 usar dados sintéticos/localmente conhecidos.

## Não objetivos atuais
Editor CAD, automação de desmontagem, IA para reconhecer toda a mecânica, nuvem, multiplayer, certificação metrológica, motor fotogramétrico próprio completo.
