//! Reconstruction and surface generation UI; execution stays in the adapter worker.
use super::*;
impl Desktop {
    fn selected_run(&self) -> Option<PathBuf> {
        let source = if let Some(id) = &self.scene.selected {
            self.scene
                .items
                .iter()
                .find(|i| &i.id == id)
                .and_then(|i| self.scene.sources.get(i.source))
        } else if self.scene.sources.len() == 1 {
            self.scene.sources.first()
        } else {
            None
        };
        source
            .filter(|s| {
                matches!(
                    s.kind,
                    noah_workspace::SourceKind::Run | noah_workspace::SourceKind::RunSurface
                )
            })
            .map(|s| s.path.clone())
    }
    pub(super) fn reconstruction_window(&mut self, ctx: &egui::Context) {
        let p = self.palette();
        let mut open = self.show_reconstruction;
        egui::Window::new("Fotogrametria e superfície").open(&mut open).default_width(680.).default_pos([270.,100.]).collapsible(false).show(ctx,|ui|{
   egui::ScrollArea::vertical().id_salt("reconstruction_scroll").max_height((ctx.content_rect().height()-180.).max(340.)).show(ui,|ui|{
    ui.heading("Fotos / Nuvem / Superfície");ui.label("COLMAP 3.11.x local. Arquivos originais são preservados.");
    let busy=self.reconstruction.worker.is_some();
    ui.add_enabled_ui(!busy,|ui|{
     ui.horizontal_wrapped(|ui|{
      if ui.button("Usar fotos do kit").clicked(){if let Some(path)=kit_path("pipeline/fotos"){self.reconstruction.images=path;self.reconstruction.source_run=None;self.reconstruction.name=self.reconstruction.available_name("reconstrucao-kit");self.notice(Ok(()),"Fotos selecionadas. Escolha a saída, COLMAP e destino.");}else{self.notice(Err("Kit não encontrado junto ao aplicativo".into()),"");}}
      if ui.button("Abrir nuvem pronta").clicked()&&let Some(path)=kit_path("pipeline/sparse-preprocessado.ply"){self.load_source(path);}
      let ready=kit_path("pipeline/superficie-cpu.obj");if ui.add_enabled(ready.is_some(),egui::Button::new("Abrir superfície pronta")).clicked()&&let Some(path)=ready{self.load_source(path);}
     });
     ui.label(RichText::new("Os botões Abrir pronta usam exemplos pré-processados, sem executar o motor.").size(10.).color(p.secondary));
     ui.separator();
     let existing=self.selected_run();
     if ui.add_enabled(existing.is_some(),egui::Button::new("Usar run selecionado para gerar malha")).clicked(){self.reconstruction.source_run=existing;self.reconstruction.mesh=true;self.reconstruction.dense=false;self.reconstruction.name=self.reconstruction.available_name("malha");self.notice(Ok(()),"Run original selecionado. A malha será gerada em novo diretório, sem repetir SfM.");}
     if let Some(source)=self.reconstruction.source_run.clone(){ui.label(format!("Origem: {}",source.display()));if ui.small_button("Voltar ao processamento de fotos").clicked(){self.reconstruction.source_run=None;self.reconstruction.name=self.reconstruction.available_name("reconstrucao");}}
     else{ui.horizontal_wrapped(|ui|{if ui.button("Escolher fotografias").clicked(){self.choose(Purpose::Images);}ui.label(self.reconstruction.images.display().to_string());});}
     ui.horizontal_wrapped(|ui|{if ui.button("Escolher COLMAP").clicked(){self.choose(Purpose::Engine);}ui.label(self.reconstruction.engine.display().to_string());});
     ui.horizontal_wrapped(|ui|{if ui.button("Pasta de resultados").clicked(){self.choose(Purpose::Output);}ui.label(self.reconstruction.parent.display().to_string());});
     ui.horizontal(|ui|{ui.label("Novo nome");ui.text_edit_singleline(&mut self.reconstruction.name);});
     ui.add_space(8.);ui.label("Saída desejada");
     let mut mode=(self.reconstruction.dense,self.reconstruction.mesh);
     egui::ComboBox::from_id_salt("reconstruction_output").selected_text(mode_name(mode)).width(400.).show_ui(ui,|ui|{for value in [(false,false),(false,true),(true,false),(true,true)]{ui.selectable_value(&mut mode,value,mode_name(value));}});
     self.reconstruction.dense=mode.0;self.reconstruction.mesh=mode.1;
     if mode.0{ui.colored_label(p.warning,"Densificação pelas fotos exige COLMAP com CUDA/GPU NVIDIA. Poisson usa uma nuvem densa com normais.");}
     if mode.1{ui.label(RichText::new(if mode.0{"Superfície da nuvem densa, sem textura UV. Esta prévia limita a 100 mil faces."}else{"Delaunay CPU usa câmeras e pontos esparsos: superfície aproximada, pode ter buracos; sem textura UV."}).size(11.).color(p.secondary));}
     if self.reconstruction.source_run.is_some()&&!mode.1{ui.colored_label(p.warning,"Escolha um modo de malha para continuar o run, ou volte às fotos.");}
     ui.horizontal(|ui|{ui.label("Limite por etapa / segundos");ui.add(egui::DragValue::new(&mut self.reconstruction.timeout).range(1..=86400));});
     if ui.add_enabled(self.reconstruction.ready_inputs(),egui::Button::new(if self.reconstruction.source_run.is_some(){"Gerar malha do run"}else{"Iniciar pelas fotos"})).clicked(){let r=self.reconstruction.start();self.notice(r,"Processamento iniciado; acompanhe as etapas.");}
    });
    ui.separator();
    if busy{ui.horizontal(|ui|{ui.spinner();ui.label("Processando");if ui.button("Cancelar").clicked(){let r=self.reconstruction.cancel();self.notice(r,"Cancelamento solicitado.");}});}
    ui.label(&self.reconstruction.status);
    if let Some(m)=&self.reconstruction.manifest{ui.label(format!("Estado: {} · {} imagens",jobs::status_label(&m.status),m.inputs.len()));for step in &m.steps{ui.horizontal(|ui|{ui.colored_label(match step.status{noah_pipeline::Status::Succeeded=>p.success,noah_pipeline::Status::Failed=>p.error,noah_pipeline::Status::Cancelled=>p.warning,_=>p.secondary},jobs::status_label(&step.status));ui.label(jobs::step_label(&step.name));});}}
    if ui.button("Ver logs e diagnóstico").clicked(){match self.reconstruction.logs(){Ok(s)=>{self.log_text=s;self.show_logs=true;},Err(e)=>self.notice(Err(e),"")}}
   });
   // Keep output actions visible even when the list of stages needs scrolling.
   ui.separator();ui.horizontal_wrapped(|ui|{
    let output=self.reconstruction.output.clone();
    let mesh=self.reconstruction.has_artifact("exports/mesh.obj")&&self.reconstruction.has_artifact("exports/mesh.metadata.json");
    if ui.add_enabled(mesh,egui::Button::new("Abrir malha no workspace")).clicked()&&let Some(root)=output.clone(){self.load_source(root);}
    for (file,label) in [("exports/sparse.ply","Abrir esparsa"),("exports/dense-view.ply","Abrir densa")]{if ui.add_enabled(self.reconstruction.has_artifact(file),egui::Button::new(label)).clicked()&&let Some(root)=output.clone(){self.load_source(root.join(file));}}
   });ui.label(RichText::new("Falhas preservam resultados parciais válidos. Malha não é sólido CAD ou medição validada.").size(10.).color(p.secondary));
  });
        self.show_reconstruction = open;
    }
}
fn mode_name(value: (bool, bool)) -> &'static str {
    match value {
        (false, false) => "Nuvem esparsa · CPU",
        (false, true) => "Malha rápida da esparsa · CPU",
        (true, false) => "Nuvem densa · CUDA",
        (true, true) => "Malha da nuvem densa · CUDA",
    }
}
