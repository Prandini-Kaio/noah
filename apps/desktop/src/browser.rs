use eframe::egui;
use std::path::{Path, PathBuf};
#[derive(Clone, Copy)]
pub enum Purpose {
    Import,
    Project,
    OpenSession,
    SaveSession,
    Demo,
    Images,
    Engine,
    Output,
    Export,
    ExportMeasurements,
}
impl Purpose {
    fn title(self) -> &'static str {
        match self {
            Self::Import => "Importar geometria",
            Self::Project => "Abrir projeto ou reconstrução",
            Self::OpenSession => "Abrir workspace",
            Self::SaveSession => "Salvar nova revisão do workspace",
            Self::Demo => "Criar montagem de exemplo",
            Self::Images => "Escolher pasta de fotografias",
            Self::Engine => "Escolher executável COLMAP",
            Self::Output => "Escolher pasta de resultados",
            Self::Export => "Exportar peça montada",
            Self::ExportMeasurements => "Exportar relatório de medições",
        }
    }
    fn folder(self) -> bool {
        matches!(self, Self::Project | Self::Images | Self::Output)
    }
    fn creates(self) -> bool {
        matches!(
            self,
            Self::SaveSession | Self::Demo | Self::Export | Self::ExportMeasurements
        )
    }
}
pub struct Browser {
    pub purpose: Purpose,
    pub closed: bool,
    dir: PathBuf,
    path_text: String,
    name: String,
    error: String,
}
impl Browser {
    pub fn new(purpose: Purpose, dir: &Path) -> Self {
        let dir = if dir.is_dir() {
            dir.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };
        Self {
            purpose,
            closed: false,
            path_text: dir.display().to_string(),
            dir,
            name: match purpose {
                Purpose::SaveSession => "workspace-001.noah.json",
                Purpose::Demo => "montagem-noah",
                Purpose::Export => "peca.obj",
                Purpose::ExportMeasurements => "medicoes.json",
                _ => "",
            }
            .into(),
            error: String::new(),
        }
    }
    pub fn export_extension(&mut self, mesh: bool) {
        self.name = if mesh { "peca.obj" } else { "peca.ply" }.into();
    }
    pub fn show(&mut self, ctx: &egui::Context) -> Option<PathBuf> {
        let mut chosen = None;
        let mut open = true;
        egui::Window::new(self.purpose.title())
            .order(egui::Order::Foreground)
            .id(egui::Id::new("file_browser"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(680.)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Pasta acima").clicked()
                        && let Some(parent) = self.dir.parent()
                    {
                        self.dir = parent.to_path_buf();
                        self.path_text = self.dir.display().to_string();
                    }
                    ui.add(egui::TextEdit::singleline(&mut self.path_text).desired_width(460.));
                    if ui.button("Ir").clicked() {
                        let p = PathBuf::from(&self.path_text);
                        if p.is_dir() {
                            self.dir = p;
                            self.error.clear();
                        } else {
                            self.error = "Pasta não encontrada".into();
                        }
                    }
                });
                ui.separator();
                let listing = (|| -> std::io::Result<Vec<(PathBuf, bool)>> {
                    let mut list = Vec::new();
                    for entry in std::fs::read_dir(&self.dir)?.take(1000) {
                        let entry = entry?;
                        let t = entry.file_type()?;
                        if t.is_symlink() {
                            continue;
                        }
                        list.push((entry.path(), t.is_dir()));
                    }
                    list.sort_by_key(|(p, d)| (!*d, p.file_name().map(|s| s.to_os_string())));
                    Ok(list)
                })();
                egui::ScrollArea::vertical()
                    .max_height(330.)
                    .min_scrolled_height(240.)
                    .show(ui, |ui| {
                        ui.set_min_height(330.);
                        match listing {
                            Ok(entries) => {
                                if entries.is_empty() {
                                    ui.label("Pasta vazia");
                                }
                                for (path, is_dir) in entries {
                                    let name =
                                        path.file_name().unwrap_or_default().to_string_lossy();
                                    let ext = path
                                        .extension()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("")
                                        .to_ascii_lowercase();
                                    let allowed = is_dir
                                        || match self.purpose {
                                            Purpose::Import => {
                                                matches!(ext.as_str(), "ply" | "obj")
                                            }
                                            Purpose::OpenSession | Purpose::SaveSession => {
                                                ext == "json"
                                            }
                                            Purpose::Engine => true,
                                            _ => self.purpose.creates(),
                                        };
                                    if !allowed {
                                        continue;
                                    }
                                    let response = ui.selectable_label(
                                        false,
                                        format!(
                                            "{}  {name}",
                                            if is_dir { "[pasta]" } else { "arquivo" }
                                        ),
                                    );
                                    if response.clicked() {
                                        if is_dir {
                                            self.dir = path.clone();
                                            self.path_text = path.display().to_string();
                                        } else if self.purpose.creates() {
                                            self.name = name.into_owned();
                                        } else {
                                            chosen = Some(path);
                                        }
                                    }
                                }
                            }
                            Err(error) => {
                                ui.colored_label(ui.visuals().error_fg_color, error.to_string());
                            }
                        }
                    });
                ui.separator();
                if self.purpose.creates() {
                    ui.horizontal(|ui| {
                        ui.label("Nome novo");
                        ui.text_edit_singleline(&mut self.name);
                    });
                }
                if !self.error.is_empty() {
                    ui.colored_label(ui.visuals().error_fg_color, &self.error);
                }
                ui.horizontal(|ui| {
                    if ui.button("Cancelar").clicked() {
                        self.closed = true;
                    }
                    if self.purpose.folder() && ui.button("Escolher esta pasta").clicked() {
                        chosen = Some(self.dir.clone());
                    }
                    if self.purpose.creates()
                        && ui
                            .button(if matches!(self.purpose, Purpose::Demo) {
                                "Criar montagem"
                            } else {
                                "Salvar"
                            })
                            .clicked()
                    {
                        if Path::new(&self.name).components().count() != 1
                            || self.name.trim().is_empty()
                            || matches!(self.name.as_str(), "." | "..")
                        {
                            self.error = "Use somente um nome de arquivo/pasta".into();
                        } else {
                            let p = self.dir.join(&self.name);
                            if p.exists() {
                                self.error =
                                    "Esse nome já existe. Escolha uma nova revisão.".into();
                            } else {
                                chosen = Some(p);
                            }
                        }
                    }
                });
            });
        if !open {
            self.closed = true;
        }
        if chosen.is_some() {
            self.closed = true;
        }
        chosen
    }
}
