use super::*;
use noah_photogrammetry::preparation::{self, Audit};
#[derive(Default)]
pub(super) struct PreparationUi {
    pub parent: PathBuf,
    pub open: bool,
    audit: Option<Audit>,
    selected: Vec<bool>,
    worker: Option<Receiver<noah_pipeline::Result<Audit>>>,
    save_worker: Option<Receiver<noah_pipeline::Result<PathBuf>>>,
    name: String,
    max_dimension: u32,
    status: String,
}
impl PreparationUi {
    pub fn with_parent(parent: PathBuf) -> Self {
        Self {
            parent,
            ..Default::default()
        }
    }
}
impl Desktop {
    pub(super) fn preparation_window(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.preparation.worker {
            match rx.try_recv() {
                Ok(result) => {
                    self.preparation.worker = None;
                    match result {
                        Ok(a) => {
                            self.preparation.selected = vec![true; a.photos.len()];
                            self.preparation.audit = Some(a);
                            self.preparation.status =
                                "Análise concluída. Avisos são heurísticos; revise a seleção."
                                    .into();
                        }
                        Err(e) => self.preparation.status = e.to_string(),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.preparation.worker = None;
                    self.preparation.status = "Análise interrompida".into();
                }
                _ => {
                    ctx.request_repaint_after(Duration::from_millis(75));
                }
            }
        }
        if let Some(rx) = &self.preparation.save_worker {
            match rx.try_recv() {
                Ok(result) => {
                    self.preparation.save_worker = None;
                    match result {
                        Ok(output) => {
                            self.reconstruction.images = output.join("images");
                            self.reconstruction.source_run = None;
                            self.reconstruction.clean_revision = None;
                            self.preparation.status = format!(
                                "Preparo salvo em {}. Entrada do pipeline atualizada.",
                                output.display()
                            );
                        }
                        Err(e) => self.preparation.status = e.to_string(),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.preparation.save_worker = None;
                    self.preparation.status = "Preparo interrompido".into();
                }
                _ => {
                    ctx.request_repaint_after(Duration::from_millis(75));
                }
            }
        }
        if !self.preparation.open {
            return;
        }
        let mut open = true;
        let mut choose_output = false;
        let p = self.palette();
        egui::Window::new("Preparar fotografias").open(&mut open).vscroll(true).max_height((ctx.content_rect().height()-160.).max(350.)).default_height(660.).default_width(760.).default_pos([300.,120.]).show(ctx,|ui|{
   let busy=self.preparation.worker.is_some()||self.preparation.save_worker.is_some();
   ui.label("Originais preservados. Avisos não excluem fotos automaticamente.");
   ui.add_enabled_ui(!busy,|ui|{
    ui.horizontal(|ui|{if ui.button("Escolher fotografias").clicked(){self.choose(Purpose::Images);}
if ui.button("Analisar pasta escolhida").clicked(){let root=self.reconstruction.images.clone();let(tx,rx)=mpsc::channel();std::thread::spawn(move||{let _=tx.send(preparation::analyze(&root));});self.preparation.worker=Some(rx);self.preparation.audit=None;self.preparation.name=jobs::available_name(&self.preparation.parent,"preparo");}});
    ui.label(self.reconstruction.images.display().to_string());
    if let Some(audit)=&self.preparation.audit{
     ui.horizontal(|ui|{if ui.button("Selecionar todas").clicked(){self.preparation.selected.fill(true);}
if ui.button("Desmarcar duplicatas exatas").clicked(){for (i,p)in audit.photos.iter().enumerate(){if p.duplicate_of.is_some(){self.preparation.selected[i]=false;}}}});
     ui.label(format!("{} / {} fotos selecionadas",self.preparation.selected.iter().filter(|&&v|v).count(),audit.photos.len()));
     egui::ScrollArea::vertical().max_height(270.).show(ui,|ui|{for(i,photo)in audit.photos.iter().enumerate(){ui.horizontal(|ui|{ui.checkbox(&mut self.preparation.selected[i],&photo.name);ui.label(format!("{}×{} · detalhe {:.1} · escuro {:.0}% · claro {:.0}%",photo.width,photo.height,photo.laplacian_variance,photo.dark_fraction*100.,photo.bright_fraction*100.));});if !photo.warning.is_empty(){ui.colored_label(p.warning,photo.warning.join(" · "));}}});
     ui.separator();ui.label("Redimensionar lado maior: 0 mantém arquivo original");ui.add(egui::DragValue::new(&mut self.preparation.max_dimension).range(0..=12000));
     ui.label("Redimensionamento preserva proporção e gera PNG sem EXIF. Intrínsecos serão estimados novamente. Sem correção de cor ou orientação automática.");
     ui.horizontal(|ui|{if ui.button("Pasta de resultados").clicked(){choose_output=true;}ui.label(self.preparation.parent.display().to_string());});
     ui.horizontal(|ui|{ui.label("Novo nome");ui.text_edit_singleline(&mut self.preparation.name);});
     if ui.add_enabled(self.preparation.selected.iter().filter(|&&v|v).count()>=3,egui::Button::new("Salvar preparo e usar no pipeline")).clicked(){
      let name=&self.preparation.name;if Path::new(name).components().count()!=1||matches!(name.as_str(),"."|"..")||name.trim().is_empty(){self.preparation.status="Nome inválido".into();}else{
       let a=audit.clone();let selected=self.preparation.selected.clone();let max=self.preparation.max_dimension;let output=self.preparation.parent.join(name);let(tx,rx)=mpsc::channel();std::thread::spawn(move||{let r=preparation::prepare(&a,&selected,max,&output).map(|()|output);let _=tx.send(r);});self.preparation.save_worker=Some(rx);
      }
     }
    }
   });if busy{ui.spinner();ctx.request_repaint_after(Duration::from_millis(75));}ui.label(&self.preparation.status);
  });
        self.preparation.open = open;
        if choose_output {
            self.choose(Purpose::PreparationOutput);
        }
    }
}
