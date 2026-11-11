# ADR 0008 — Estilhaços somente na camada de apresentação
Status: aceito no escopo autorizado pelo usuário em 2026-11-10.

O usuário pediu explosão multidirecional e quebra visual reversível até de um
objeto único. A geometria de domínio, calibração e exportação devem permanecer
intactas. O texto visual anexado é referência de produto, não instrução operacional.

Decisão: cache descartável `apps/desktop/src/blast.rs` em coordenadas mundiais,
com subdivisão de faces em quatro triângulos e rotação/impulso determinísticos.
Nuvens usam pontos. O cache retorna as mesmas primitivas de projeção existentes,
identificadas pelo item original; é limitado a 20 mil fragmentos e não serializado.
Durante a prévia, edição/seleção de domínio ficam bloqueadas e abrir Medir restaura.

`apps/desktop/src/shell.rs` reúne navegação e painéis contextuais; main orquestra.
Não há dependência do domínio para UI, schema novo ou engine de física.
Consequências: efeito funciona com malha única; sem interior, cortes volumétricos
ou colisões. Amostragem pode deixar lacunas em modelos grandes. Fratura volumétrica
real exigiria outra decisão, dados derivados e validação geométrica próprios.
