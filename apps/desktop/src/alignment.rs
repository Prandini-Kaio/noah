//! Initial E5 guided alignment: exact vertex pair, translation preview and guarded undo.
use super::*;
use noah_workspace::alignment::{AlignmentPlan, AlignmentUndo};
pub struct AlignmentUi {
    pub open: bool,
    pub preview: bool,
    pub plan: Option<AlignmentPlan>,
    undo: Option<AlignmentUndo>,
    source: String,
    target: String,
    source_vertex: usize,
    target_vertex: usize,
}
impl Default for AlignmentUi {
    fn default() -> Self {
        Self {
            open: false,
            preview: false,
            plan: None,
            undo: None,
            source: String::new(),
            target: String::new(),
            source_vertex: 1,
            target_vertex: 2,
        }
    }
}
impl Desktop {
    pub(super) fn open_alignment_kit(&mut self) {
        if let (Some(a), Some(b)) = (
            kit_path("alinhamento/fixa.obj"),
            kit_path("alinhamento/movel.obj"),
        ) {
            self.request_replace(Replace::AlignmentKit([a, b]));
            self.module = shell::Module::Assembly;
            self.analysis_tab = false;
        } else {
            self.notice(
                Err("Peças do kit não encontradas junto ao aplicativo".into()),
                "",
            );
        }
    }

    pub(super) fn open_alignment(&mut self) {
        self.restore_blast();
        self.stop_transition();
        if self.scene.explosion != 0. {
            self.scene.explosion = 0.;
            self.dirty = true;
        }
        self.measure_mode = false;
        self.show_measurements = false;
        if !self
            .scene
            .items
            .iter()
            .any(|i| i.id == self.alignment.source)
        {
            self.alignment.source = self
                .scene
                .selected
                .clone()
                .filter(|id| self.scene.items.iter().any(|i| &i.id == id))
                .or_else(|| self.scene.items.last().map(|i| i.id.clone()))
                .unwrap_or_default();
        }
        if self.alignment.target == self.alignment.source
            || !self
                .scene
                .items
                .iter()
                .any(|i| i.id == self.alignment.target)
        {
            self.alignment.target = self
                .scene
                .items
                .iter()
                .find(|i| i.id != self.alignment.source)
                .map(|i| i.id.clone())
                .unwrap_or_default();
        }
        self.alignment.source_vertex = self.alignment.source_vertex.max(1);
        self.alignment.target_vertex = self.alignment.target_vertex.max(1);
        self.alignment.plan = None;
        self.alignment.preview = false;
        self.alignment.open = true;
    }
    pub(super) fn alignment_window(&mut self, ctx: &egui::Context) {
        let p = self.palette();
        let mut open = self.alignment.open;
        egui::Window::new("Alinhar peças por vértices").open(&mut open).default_width(480.).default_pos([300.,110.]).show(ctx,|ui|{
   ui.add_enabled_ui(self.pending.is_none(),|ui|{
    ui.label("Mova um vértice da peça móvel até um vértice da peça de destino.");
    ui.label(RichText::new("Somente translação no estado montado. Rotação e escala são preservadas.").size(11.).color(p.secondary));
    let mut changed=false;
    for (id,label,selected,vertex) in [("align_source","Peça móvel",&mut self.alignment.source,&mut self.alignment.source_vertex),("align_target","Destino fixo",&mut self.alignment.target,&mut self.alignment.target_vertex)]{
     ui.add_space(8.);ui.label(label);
     let name=self.scene.items.iter().find(|i|&i.id==selected).map(|i|i.name.as_str()).unwrap_or("Selecione uma peça");
     egui::ComboBox::from_id_salt(id).selected_text(name).width(320.).show_ui(ui,|ui|{for item in &self.scene.items{changed|=ui.selectable_value(selected,item.id.clone(),&item.name).changed();}});
     ui.horizontal(|ui|{ui.label("Vértice original (inicia em 1)");changed|=ui.add(egui::DragValue::new(vertex).range(1..=2_000_000)).changed();});
    }
    let pair=self.anchors.len()==2&&self.anchors[0].item_id!=self.anchors[1].item_id;
    if ui.add_enabled(pair,egui::Button::new("Usar pontos A/B escolhidos em Medir")).clicked(){self.alignment.source=self.anchors[0].item_id.clone();self.alignment.source_vertex=self.anchors[0].vertex+1;self.alignment.target=self.anchors[1].item_id.clone();self.alignment.target_vertex=self.anchors[1].vertex+1;changed=true;}
    if changed{self.alignment.plan=None;self.alignment.preview=false;}
    if ui.add_enabled(self.scene.items.len()>=2,egui::Button::new("Calcular deslocamento")).clicked(){
     let a=Anchor{item_id:self.alignment.source.clone(),vertex:self.alignment.source_vertex.saturating_sub(1)};
     let b=Anchor{item_id:self.alignment.target.clone(),vertex:self.alignment.target_vertex.saturating_sub(1)};
     match self.scene.plan_alignment(a,b){Ok(plan)=>{self.alignment.plan=Some(plan);self.anchors.clear();self.notice(Ok(()),"Deslocamento calculado; confira a prévia antes de aplicar.");},Err(e)=>self.notice(Err(e),"")}
    }
    if let Some(plan)=self.alignment.plan.clone(){
     if plan.is_current(&self.scene){
      let (factor,unit)=self.scene.scale_factor().ok().flatten().map(|(f,u)|(f,u.symbol())).unwrap_or((1.,"u. cena"));
      for (axis,value) in ["ΔX","ΔY","ΔZ"].iter().zip(plan.delta()){ui.monospace(format!("{axis}: {:.*} {unit}",self.scene.view.decimals,value*factor));}
      if self.scene.scale_factor().is_err(){ui.colored_label(p.warning,"Referência desatualizada: deslocamento exibido em unidades da cena.");}
      ui.checkbox(&mut self.alignment.preview,"Pré-visualizar sem alterar a montagem");
      if ui.button("Aplicar alinhamento").clicked(){match self.scene.apply_alignment(&plan){Ok(undo)=>{self.alignment.undo=Some(undo);self.alignment.plan=None;self.alignment.preview=false;self.dirty=true;self.notice(Ok(()),"Peça alinhada. Fonte original preservada.");},Err(e)=>self.notice(Err(e),"")}}
     }else{self.alignment.preview=false;ui.colored_label(p.warning,"A cena mudou. Calcule novamente.");}
    }
    let can_undo=self.alignment.undo.as_ref().is_some_and(|u|u.is_current(&self.scene));
    if ui.add_enabled(can_undo,egui::Button::new("Desfazer último alinhamento")).on_hover_text("Disponível enquanto a geometria, hierarquia e transformações posteriores não mudarem").clicked()&&let Some(undo)=self.alignment.undo.take(){let r=self.scene.undo_alignment(&undo);if r.is_ok(){self.dirty=true;self.alignment.plan=None;self.alignment.preview=false;}self.notice(r,"Alinhamento desfeito.");}
    if self.scene.items.len()<2{ui.colored_label(p.warning,"Importe pelo menos duas peças. O pacote inclui fixa.obj e movel.obj em examples/alinhamento.");}
    ui.separator();ui.label(RichText::new("Medidas são recalculadas; referências entre peças podem precisar de recalibração. Salvar guarda o ajuste aplicado, não a prévia ou o histórico de desfazer.").size(11.).color(p.secondary));
   });
  });
        self.alignment.open = open;
        if !open {
            self.alignment.preview = false;
        }
    }
}
