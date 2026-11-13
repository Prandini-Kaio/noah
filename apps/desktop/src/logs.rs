//! Selectable read-only log view and clipboard actions.
use super::*;
impl Desktop {
    pub(super) fn open_logs(&mut self) {
        self.show_logs = true;
        self.refresh_logs();
    }
    fn refresh_logs(&mut self) {
        match self
            .reconstruction
            .logs_filtered(self.log_step, self.log_stream)
        {
            Ok(s) => self.log_text = s,
            Err(e) => self.log_text = format!("Não foi possível carregar o diagnóstico: {e}"),
        }
    }
    pub(super) fn logs_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_logs;
        let mut refresh = false;
        egui::Window::new("Diagnóstico da reconstrução").open(&mut open).default_size([900.,580.]).vscroll(false).show(ctx,|ui|{
 ui.label("Texto selecionável · Ctrl+A seleciona tudo · Ctrl+C copia a seleção");
 ui.horizontal_wrapped(|ui|{
  if ui.button("Copiar texto exibido").clicked(){ctx.copy_text(self.log_text.clone());self.notice(Ok(()),"Diagnóstico copiado para a área de transferência.");}
  if ui.button("Copiar relatório completo").clicked(){match self.reconstruction.logs_filtered(None,0){Ok(s)=>{ctx.copy_text(s);self.notice(Ok(()),"Relatório completo copiado.");},Err(e)=>self.notice(Err(e),"")}}
  if ui.button("Atualizar").clicked(){refresh=true;}
  let before=(self.log_step,self.log_stream);
  egui::ComboBox::from_id_salt("log_step").selected_text(self.log_step.and_then(|i|self.reconstruction.manifest.as_ref().and_then(|m|m.steps.get(i))).map(|s|jobs::step_label(&s.name)).unwrap_or("Todas as etapas")).show_ui(ui,|ui|{ui.selectable_value(&mut self.log_step,None,"Todas as etapas");if let Some(m)=&self.reconstruction.manifest{for(i,s)in m.steps.iter().enumerate(){ui.selectable_value(&mut self.log_step,Some(i),format!("{} · {}",i+1,jobs::step_label(&s.name)));}}});
  egui::ComboBox::from_id_salt("log_stream").selected_text(["Saída e diagnóstico","Somente saída","Somente diagnóstico"][self.log_stream.min(2)]).show_ui(ui,|ui|{for(i,label)in["Saída e diagnóstico","Somente saída","Somente diagnóstico"].iter().enumerate(){ui.selectable_value(&mut self.log_stream,i,*label);}});refresh|=before!=(self.log_step,self.log_stream);
 });ui.separator();egui::ScrollArea::both().id_salt("diagnostic_text").max_height((ctx.content_rect().height()-250.).max(250.)).show(ui,|ui|{let mut text=self.log_text.as_str();ui.add(egui::TextEdit::multiline(&mut text).code_editor().desired_width(1000.).desired_rows(26).interactive(true));});
 ui.label("Até 128 KiB finais por fluxo/etapa, total de 2 MiB. Arquivos completos permanecem na pasta logs do run.");});
        self.show_logs = open;
        if refresh {
            self.refresh_logs();
        }
    }
}
