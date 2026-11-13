//! Dedicated presentation workspace: hierarchy disassembly versus surface fragmentation.
use super::*;
use noah_workspace::disassembly::Mode;
impl Desktop {
    pub(super) fn effects_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_effects;
        let p = self.palette();
        let mut close = false;
        egui::Window::new("Explosão e desmontagem").open(&mut open).default_pos([(ctx.content_rect().right()-550.).max(10.),120.]).default_width(540.).default_height(580.).vscroll(true).max_height((ctx.content_rect().height()-140.).max(350.)).show(ctx,|ui|{
 if self.cleaning_target().is_some(){ui.label("Conclua ou descarte a edição da nuvem para usar os efeitos.");if ui.button("Voltar à edição da nuvem").clicked(){self.cleaning.open=true;}return;}
 ui.heading("Apresentação do objeto");ui.label("Efeitos reversíveis. Fontes, medidas e exportação continuam no estado montado.");
 ui.horizontal(|ui|{if ui.selectable_label(!self.blast_mode,"Desmontar componentes").clicked(){self.blast_mode=false;self.restore_blast();}
if ui.selectable_label(self.blast_mode,"Fragmentar superfície").clicked(){self.blast_mode=true;self.stop_transition();self.scene.explosion=0.;}});ui.separator();
 if self.blast_mode{self.blast_controls(ui);}else{
  let before=self.scene.view.disassembly.clone();
  ui.label("Modo de desmontagem");egui::ComboBox::from_id_salt("disassembly_mode").selected_text(match self.scene.view.disassembly.mode{Mode::Radial=>"Radial · todas as peças",Mode::Layers=>"Camadas · preservar submontagens",Mode::Armour=>"Armadura · abertura progressiva"}).width(420.).show_ui(ui,|ui|{ui.selectable_value(&mut self.scene.view.disassembly.mode,Mode::Radial,"Radial · todas as peças");ui.selectable_value(&mut self.scene.view.disassembly.mode,Mode::Layers,"Camadas · preservar submontagens");ui.selectable_value(&mut self.scene.view.disassembly.mode,Mode::Armour,"Armadura · abertura progressiva");});
  if self.scene.view.disassembly.mode!=Mode::Radial{
   ui.add(egui::Slider::new(&mut self.scene.view.disassembly.depth,1..=8).text("Níveis a abrir"));
   ui.horizontal(|ui|{ui.label("Eixo das camadas");for(a,label)in["X","Y","Z"].iter().enumerate(){ui.selectable_value(&mut self.scene.view.disassembly.axis,a,*label);}});
   ui.checkbox(&mut self.scene.view.disassembly.selected_only,"Somente peça/montagem selecionada");
   ui.label("Nível 1 separa grupos mantendo seus filhos juntos. Aumente para revelar as peças internas. Armadura abre os níveis em sequência.");
  }
  if before!=self.scene.view.disassembly{self.stop_transition();if let Err(e)=self.scene.validate_world(){self.scene.view.disassembly=before;self.notice(Err(e),"");}else{self.dirty=true;}}
  let old_distance=self.scene.explosion_distance;let old_amount=self.scene.explosion;
  let distance_changed=ui.add(egui::DragValue::new(&mut self.scene.explosion_distance).range(0.0..=1e6).speed(0.05).prefix("Afastamento / u. cena ")).changed();
  let mut amount=self.scene.explosion;if ui.add(egui::Slider::new(&mut amount,0.0..=1.0).text("Abertura")).changed(){self.stop_transition();self.scene.explosion=amount;self.dirty=true;}
  if distance_changed{self.stop_transition();if let Err(e)=self.scene.validate_world(){self.scene.explosion_distance=old_distance;self.scene.explosion=old_amount;self.notice(Err(e),"");}else{self.dirty=true;}}
  ui.horizontal(|ui|{if ui.add_enabled(!self.scene.items.is_empty(),egui::Button::new("Desmontar / Explodir").fill(p.primary)).clicked(){if self.cleaning_target().is_some(){self.notice(Err("Conclua a edição da nuvem antes de desmontar".into()),"");}else{self.animate_explosion(1.);close=self.transition.is_some()||self.scene.explosion>0.;}}
if ui.button("Remontar").clicked(){self.animate_explosion(0.);close=true;}
if ui.button("Enquadrar").clicked(){self.fit_view();}});
  if self.scene.items.len()<2{ui.colored_label(p.warning,"Uma malha isolada não contém peças internas separadas. Use Painéis de superfície para uma desmontagem visual, ou crie uma montagem com componentes.");}
  ui.separator();ui.strong("Exemplo com objetos dentro de objetos");ui.label("Carcaça, peitoral, capacete, braços e núcleo. Geometria sintética, sem inferir partes ocultas de uma captura.");if ui.button("Criar exemplo de armadura…").clicked(){self.choose(Purpose::ArmourDemo);}
 }
 });
        if self
            .blast
            .as_ref()
            .is_some_and(|b| b.progress == 0. && b.playing)
        {
            close = true;
        }
        self.show_effects = open && !close;
    }
}
