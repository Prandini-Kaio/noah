//! One guided pipeline entry; application settings and geometry editing are separate.
use super::*;
impl Desktop {
    pub(super) fn selected_run(&self) -> Option<PathBuf> {
        let source = self
            .scene
            .selected
            .as_ref()
            .and_then(|id| self.scene.items.iter().find(|i| &i.id == id))
            .and_then(|i| self.scene.sources.get(i.source));
        source
            .filter(|s| {
                matches!(
                    s.kind,
                    noah_workspace::SourceKind::Run
                        | noah_workspace::SourceKind::RunSurface
                        | noah_workspace::SourceKind::CloudRevision
                )
            })
            .map(|s| s.path.clone())
    }
    pub(super) fn configure_selected_run(&mut self) {
        let result = (|| -> noah_pipeline::Result<()> {
            let path = self
                .selected_run()
                .ok_or("selecione uma reconstrução ou revisão de nuvem na estrutura")?;
            let (source, revision) = if path.join("cloud-revision.json").is_file() {
                let (s, _) = noah_storage::cloud_revision::run_selection(&path)?;
                (s.path, Some(path))
            } else {
                (path, None)
            };
            self.reconstruction.source_run = Some(source);
            self.reconstruction.clean_revision = revision;
            self.reconstruction.mesh = true;
            self.reconstruction.dense = false;
            self.reconstruction.name = self.reconstruction.available_name("malha");
            Ok(())
        })();
        if result.is_ok() {
            self.pipeline_tab = 0;
            self.show_reconstruction = true;
        }
        self.notice(
            result,
            "Origem selecionada. Configure a nova superfície no passo 1.",
        );
    }
    pub(super) fn attach_selected_run(&mut self) {
        let result = (|| -> noah_pipeline::Result<()> {
            let path = self.selected_run().ok_or("selecione uma reconstrução")?;
            let root = if path.join("cloud-revision.json").is_file() {
                noah_storage::cloud_revision::run_selection(&path)?.0.path
            } else {
                path
            };
            self.reconstruction.attach(root)?;
            Ok(())
        })();
        if result.is_ok() {
            self.pipeline_tab = 2;
            self.show_reconstruction = true;
        }
        self.notice(
            result,
            "Resultados carregados. Escolha Visualizar ou Editar.",
        );
    }
    pub(super) fn start_pipeline(&mut self) {
        let result = self.reconstruction.start();
        if result.is_ok() {
            self.pipeline_tab = 1;
            self.preferences.last_run = self.reconstruction.output.clone();
            self.save_preferences();
        }
        self.notice(
            result,
            "Reconstrução iniciada. Acompanhe as etapas de processamento.",
        );
    }
    pub(super) fn open_result(&mut self, path: PathBuf, sparse: bool, edit: bool) {
        if self.pending.is_some() {
            return;
        }
        if self.cleaning_target().is_some() {
            self.notice(
                Err("Salve ou descarte a edição de nuvem antes de abrir outro resultado".into()),
                "",
            );
            return;
        }
        self.show_reconstruction = false;
        self.preparation.open = false;
        self.show_logs = false;
        self.show_effects = false;
        self.restore_blast();
        self.stop_transition();
        self.scene.explosion = 0.;
        let canonical = std::fs::canonicalize(&path).unwrap_or(path.clone());
        let mesh = !sparse && path.is_dir() && path.join("exports/mesh.obj").is_file();
        let existing = self.scene.sources.iter().position(|s| {
            s.path == canonical
                && if path.is_dir() {
                    if sparse {
                        s.kind == noah_workspace::SourceKind::Run
                    } else if mesh {
                        s.kind == noah_workspace::SourceKind::RunSurface
                    } else {
                        true
                    }
                } else {
                    true
                }
        });
        if let Some(index) = existing {
            self.scene.selected = self
                .scene
                .items
                .iter()
                .find(|i| i.source == index)
                .map(|i| i.id.clone());
            self.focus_result(edit);
        } else {
            self.focus_loaded = true;
            self.edit_loaded = edit;
            if sparse {
                self.load_sparse_source(path);
            } else {
                self.load_source(path);
            }
        }
    }
    pub(super) fn focus_result(&mut self, edit: bool) {
        if let Some(id) = self.scene.selected.clone() {
            for item in &mut self.scene.items {
                item.visible = item.id == id;
            }
        }
        self.module = shell::Module::Inspection;
        self.inspector_tab = 0;
        self.analysis_tab = false;
        self.right_visible = true;
        self.show_clouds = true;
        self.show_mesh = true;
        self.display_mode = shell::DisplayMode::Solid;
        self.scene.wireframe = false;
        let result = self.scene.fit(true);
        self.dirty = true;
        self.notice(result,"Resultado aberto e enquadrado. Outros objetos estão ocultos; reative-os na estrutura para comparar.");
        if edit {
            let r = self.begin_cleaning();
            if r.is_ok() {
                self.cleaning.open = true;
            }
            self.notice(r, "Edição da nuvem iniciada. Fonte original preservada.");
        }
    }
    pub(super) fn reconstruction_window(&mut self, ctx: &egui::Context) {
        let p = self.palette();
        let mut open = self.show_reconstruction;
        let mut close = false;
        egui::Window::new("Reconstrução · fluxo guiado").open(&mut open).default_width(720.).default_height(640.).max_height((ctx.content_rect().height()-120.).max(400.)).default_pos([300.,105.]).vscroll(false).collapsible(false).show(ctx,|ui|{
  ui.heading("Da captura ao objeto 3D");ui.label(format!("No workspace: {} nuvens · {} malhas · {} montagens",self.scene.items.iter().filter(|i|matches!(i.geometry.as_ref(),Geometry::Points(_))).count(),self.scene.items.iter().filter(|i|matches!(i.geometry.as_ref(),Geometry::Mesh(_))).count(),self.scene.assemblies.len()));
  ui.horizontal(|ui|{for(i,label)in["1 · Configurar","2 · Processar","3 · Resultados"].iter().enumerate(){ui.selectable_value(&mut self.pipeline_tab,i,*label);}});ui.separator();
  egui::ScrollArea::vertical().id_salt(("pipeline_body",self.pipeline_tab)).max_height((ctx.content_rect().height()-350.).max(260.)).show(ui,|ui|{
  match self.pipeline_tab{
   0=>{let busy=self.reconstruction.worker.is_some();if busy{ui.colored_label(p.warning,"Uma execução está ativa. Aguarde ou cancele no passo 2.");}
    ui.add_enabled_ui(!busy,|ui|{
     ui.heading("Entrada da execução");ui.horizontal(|ui|{if ui.selectable_label(self.reconstruction.source_run.is_none(),"Novas fotografias").clicked(){self.reconstruction.source_run=None;self.reconstruction.clean_revision=None;self.reconstruction.name=self.reconstruction.available_name("reconstrucao");}
      if ui.add_enabled(self.selected_run().is_some(),egui::Button::new("Continuar run/revisão selecionado")).on_hover_text("Selecione um run ou uma revisão sparse na estrutura.").clicked(){self.configure_selected_run();}});
     if let Some(source)=&self.reconstruction.source_run{ui.label(format!("Run de origem: {}",source.display()));if let Some(revision)=&self.reconstruction.clean_revision{ui.label(format!("Revisão limpa: {}",revision.display()));}ui.label("A nova superfície reutiliza câmeras e pontos. O run original é preservado.");}
     else{ui.horizontal(|ui|{if ui.button("Escolher fotografias…").clicked(){self.choose(Purpose::Images);}
if ui.button("Usar fotos do kit").clicked()&&let Some(path)=kit_path("pipeline/fotos"){self.preferences.images=path.clone();self.reconstruction.images=path;self.reconstruction.name=self.reconstruction.available_name("reconstrucao-kit");self.save_preferences();self.notice(Ok(()),"12 fotos sintéticas do kit selecionadas. Você pode prepará-las antes de reconstruir.");}});ui.label(if self.reconstruction.images.as_os_str().is_empty(){"Nenhuma pasta de fotos selecionada".into()}else{self.reconstruction.images.display().to_string()});if ui.add_enabled(self.reconstruction.images.is_dir(),egui::Button::new("Preparar/selecionar imagens…")).clicked(){self.preparation.open=true;}}
     ui.separator();ui.heading("Resultado desejado");let mut mode=(self.reconstruction.dense,self.reconstruction.mesh);egui::ComboBox::from_id_salt("pipeline_mode").selected_text(mode_name(mode)).width(480.).show_ui(ui,|ui|{for value in [(false,false),(false,true),(true,false),(true,true)]{ui.selectable_value(&mut mode,value,mode_name(value));}});self.reconstruction.dense=mode.0;self.reconstruction.mesh=mode.1;
     ui.label(match mode{(false,false)=>"Produz pontos e câmeras; indicado para inspecionar e limpar a nuvem.",(false,true)=>"Produz uma superfície aproximada da nuvem esparsa, sem textura.",(true,false)=>"Produz nuvem densa com profundidade; requer COLMAP com CUDA/GPU NVIDIA.",_=>"Densifica com CUDA e gera superfície Poisson, sem textura UV."});
     if mode.0{ui.colored_label(p.warning,"O fluxo denso completo ainda não foi validado em GPU nesta prévia.");}
     ui.separator();ui.heading("Destino desta execução");ui.horizontal(|ui|{if ui.button("Pasta de resultados…").clicked(){self.choose(Purpose::Output);}ui.label(self.reconstruction.parent.display().to_string());});ui.horizontal(|ui|{ui.label("Nome novo");ui.text_edit_singleline(&mut self.reconstruction.name);});ui.label("Cada execução cria uma pasta nova. Nunca substitui resultados existentes.");
     ui.collapsing("Configuração avançada da pipeline",|ui|{ui.horizontal(|ui|{ui.label("Limite por etapa / segundos");ui.add(egui::DragValue::new(&mut self.reconstruction.timeout).range(1..=86400));});});
    });
   },
   1=>{ui.heading("Processamento");ui.label(&self.reconstruction.status);if let Some(root)=&self.reconstruction.output{ui.label(RichText::new(root.display().to_string()).size(11.));}
    if let Some(m)=&self.reconstruction.manifest{let done=m.steps.iter().filter(|s|s.status==noah_pipeline::Status::Succeeded).count();ui.label(format!("{} · {} imagens · {} etapas concluídas",jobs::status_label(&m.status),m.inputs.len(),done));for(i,step)in m.steps.iter().enumerate(){egui::Frame::group(ui.style()).show(ui,|ui|{ui.set_min_width(ui.available_width());ui.horizontal_wrapped(|ui|{if step.status==noah_pipeline::Status::Running{ui.spinner();}ui.colored_label(match step.status{noah_pipeline::Status::Succeeded=>p.success,noah_pipeline::Status::Failed=>p.error,_=>p.secondary},format!("{:02} · {} · {}",i+1,jobs::step_label(&step.name),jobs::status_label(&step.status)));});if let Some(d)=&step.diagnostic{ui.colored_label(p.error,d);}});}
if let Some(d)=&m.diagnostic{ui.colored_label(p.error,d);}}
    else{ui.label("Nenhuma execução iniciada. Configure a entrada no passo 1.");}
   },
   _=>{ui.heading("Resultados da última execução");if let Some(root)=&self.reconstruction.output{ui.label(root.display().to_string());}else{ui.label("Você ainda não executou uma reconstrução nesta sessão.");}
    if ui.add_enabled(self.selected_run().is_some()&&self.reconstruction.worker.is_none(),egui::Button::new("Carregar resultados do run selecionado")).clicked(){self.attach_selected_run();}
    let output=self.reconstruction.output.clone();
    for(file,title,description,sparse,cloud)in[("exports/sparse.ply","Nuvem esparsa","Pontos e câmeras. Você pode editar e gerar uma nova superfície.",true,true),("exports/dense-view.ply","Nuvem densa","Pontos densificados. Limpeza disponível; malha limpa deste fluxo usa sparse.",false,true),("exports/mesh.obj","Superfície 3D","Malha triangular sem textura. Não é um sólido CAD.",false,false)]{
     let available=self.reconstruction.has_artifact(file);egui::Frame::group(ui.style()).show(ui,|ui|{ui.set_min_width(ui.available_width());ui.horizontal(|ui|{ui.strong(title);ui.colored_label(if available{p.success}else{p.secondary},if available{"Disponível"}else{"Não produzido"});});ui.label(description);ui.horizontal(|ui|{
      if ui.add_enabled(available,egui::Button::new("Visualizar no workspace")).clicked()&&let Some(root)=output.clone(){let path=if sparse||!cloud{root}else{root.join(file)};self.open_result(path,sparse,false);close=true;}
      if cloud&&ui.add_enabled(available,egui::Button::new("Editar nuvem…")).clicked()&&let Some(root)=output.clone(){let path=if sparse{root}else{root.join(file)};self.open_result(path,sparse,true);close=true;}
     });});
    }
    if self.reconstruction.worker.is_some(){ui.spinner();ui.label("A execução continua. Os resultados aparecem quando validados.");}
    ui.collapsing("Exemplos prontos · sem executar COLMAP",|ui|{ui.label("Dados sintéticos pré-processados para experimentar a interface.");ui.horizontal_wrapped(|ui|{for(relative,label)in[("pipeline/sparse-preprocessado.ply","Visualizar nuvem de exemplo"),("pipeline/superficie-cpu.obj","Visualizar superfície de exemplo"),("limpeza/nuvem-com-outliers.ply","Editar exemplo com outliers")]{if ui.button(label).clicked()&&let Some(path)=kit_path(relative){self.open_result(path,false,relative.contains("outliers"));close=true;}}});});
    if let Some(last)=self.preferences.last_run.clone().filter(|p|p.join("run.json").is_file())&&self.reconstruction.output.as_ref()!=Some(&last)&&ui.button("Carregar última execução lembrada").clicked(){let r=self.reconstruction.attach(last);self.notice(r,"Execução anterior carregada. Escolha um resultado para visualizar.");}
   }
  }
  });
  if self.pipeline_tab==1{ui.separator();    ui.horizontal(|ui|{if ui.add_enabled(self.reconstruction.manifest.is_some(),egui::Button::new("Abrir diagnóstico / copiar logs")).clicked(){self.open_logs();}
if self.reconstruction.worker.is_some()&&ui.button("Cancelar execução").clicked(){let r=self.reconstruction.cancel();self.notice(r,"Cancelamento solicitado; resultados parciais preservados.");}
if ui.button("Ver resultados disponíveis").clicked(){self.pipeline_tab=2;}});
}
  if self.pipeline_tab==0{     ui.separator();ui.horizontal_wrapped(|ui|{ui.label(if self.reconstruction.engine.is_file(){"COLMAP configurado"}else{"COLMAP ainda não configurado"});if ui.button("Configurações gerais…").clicked(){self.show_general=true;}});ui.label(RichText::new(self.reconstruction.engine.display().to_string()).size(10.).color(p.secondary));
     if let Some(reason)=self.reconstruction.validation_error(){ui.colored_label(p.warning,reason);}else{ui.colored_label(p.success,"Tudo pronto para iniciar.");}
     if ui.add_enabled(self.reconstruction.ready_inputs(),egui::Button::new("Iniciar reconstrução").fill(p.primary).min_size(Vec2::new(250.,36.))).clicked(){self.start_pipeline();}

  }

 });
        self.show_reconstruction = open && !close;
    }
}
fn mode_name(mode: (bool, bool)) -> &'static str {
    match mode {
        (false, false) => "Nuvem esparsa · CPU",
        (false, true) => "Superfície rápida · CPU",
        (true, false) => "Nuvem densa · CUDA",
        (true, true) => "Superfície da nuvem densa · CUDA",
    }
}
