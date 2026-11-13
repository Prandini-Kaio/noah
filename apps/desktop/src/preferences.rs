//! Local application preferences, independent from scene/session and pipeline runs.
use super::*;
use std::collections::BTreeMap;
pub(super) struct Preferences {
    file: Option<PathBuf>,
    pub directories: BTreeMap<String, PathBuf>,
    pub engine: PathBuf,
    pub images: PathBuf,
    pub output: PathBuf,
    pub last_run: Option<PathBuf>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            file: config_file(),
            directories: BTreeMap::new(),
            engine: PathBuf::new(),
            images: PathBuf::new(),
            output: std::env::current_dir().unwrap_or_default(),
            last_run: None,
        }
    }
}
fn config_file() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
        .map(|p| p.join("noah/preferences.json"))
}
impl Preferences {
    pub fn load() -> (Self, Option<String>) {
        Self::load_file(config_file())
    }
    fn load_file(file: Option<PathBuf>) -> (Self, Option<String>) {
        let mut p = Self {
            file,
            ..Default::default()
        };
        let Some(file) = &p.file else {
            return (p, Some("Pasta de configurações indisponível".into()));
        };
        if !file.exists() {
            return (p, None);
        }
        let result = (|| -> noah_pipeline::Result<()> {
            let v: serde_json::Value =
                serde_json::from_slice(&noah_pipeline::read_bounded(file, 1024 * 1024)?)?;
            if v["schema_version"] != 1 {
                return Err("versão de preferências incompatível".into());
            }
            if let Some(d) = v["directories"].as_object() {
                for (k, v) in d {
                    if let Some(s) = v.as_str() {
                        let path = PathBuf::from(s);
                        if path.is_absolute() {
                            p.directories.insert(k.clone(), path);
                        }
                    }
                }
            }
            for (key, target) in [
                ("engine", &mut p.engine),
                ("images", &mut p.images),
                ("output", &mut p.output),
            ] {
                if let Some(s) = v[key].as_str() {
                    let path = PathBuf::from(s);
                    if path.is_absolute() {
                        *target = path;
                    }
                }
            }
            p.last_run = v["last_run"]
                .as_str()
                .map(PathBuf::from)
                .filter(|p| p.is_absolute());
            Ok(())
        })();
        let error = result
            .err()
            .map(|e| format!("Preferências não carregadas: {e}"));
        (p, error)
    }
    pub fn directory(&self, purpose: Purpose) -> PathBuf {
        self.directories
            .get(purpose.key())
            .cloned()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
    }
    pub fn remember(&mut self, purpose: Purpose, dir: &Path) {
        if dir.is_absolute() {
            self.directories.insert(purpose.key().into(), dir.into());
        }
    }
    pub fn save(&self) -> noah_pipeline::Result<()> {
        let file = self
            .file
            .as_ref()
            .ok_or("pasta de configurações indisponível")?;
        let parent = file.parent().ok_or("pasta de configurações inválida")?;
        std::fs::create_dir_all(parent)?;
        let bytes = serde_json::to_vec_pretty(
            &serde_json::json!({"schema_version":1,"directories":self.directories,"engine":self.engine,"images":self.images,"output":self.output,"last_run":self.last_run}),
        )?;
        let tmp = parent.join(format!("preferences-{:032x}.tmp", noah_storage::new_id()));
        noah_pipeline::write_new(&tmp, &bytes)?;
        if let Err(e) = std::fs::rename(&tmp, file) {
            let _ = std::fs::remove_file(&tmp);
            return Err(e.into());
        }
        Ok(())
    }
}
impl Desktop {
    pub(super) fn save_preferences(&mut self) {
        if let Err(e) = self.preferences.save() {
            self.notice(Err(e), "");
        }
    }
    pub(super) fn general_window(&mut self, ctx: &egui::Context) {
        if !self.show_general {
            return;
        }
        let mut open = true;
        egui::Window::new("Configurações gerais").open(&mut open).default_width(620.).vscroll(true).max_height(650.).show(ctx,|ui|{
 ui.heading("Motor de reconstrução");ui.label("COLMAP 3.11.x instalado neste computador. O caminho é lembrado; só executamos ao iniciar a pipeline.");ui.horizontal(|ui|{if ui.button("Selecionar COLMAP…").clicked(){self.choose(Purpose::Engine);}
if ui.button("Esquecer motor").clicked(){self.reconstruction.engine=PathBuf::new();self.preferences.engine=PathBuf::new();self.save_preferences();}});ui.label(self.reconstruction.engine.display().to_string());ui.separator();ui.heading("Interface e navegação");if ui.button("Tema, réguas e movimento…").clicked(){self.show_visual=true;}ui.checkbox(&mut self.left_visible,"Mostrar estrutura");ui.checkbox(&mut self.right_visible,"Mostrar inspetor");ui.separator();ui.heading("Pastas lembradas por finalidade");ui.label("Selecionar imagens não muda a pasta do COLMAP, exportação, preparo ou limpeza.");let entries:Vec<_>=self.preferences.directories.iter().map(|(k,v)|(k.clone(),v.clone())).collect();for(key,path)in entries{ui.horizontal_wrapped(|ui|{ui.label(format!("{}: {}",Purpose::key_label(&key),path.display()));if ui.small_button("Esquecer").clicked(){self.preferences.directories.remove(&key);self.save_preferences();}});}ui.separator();ui.label("Configurações da execução (saída, modo e timeout) ficam na pipeline. Motor e histórico de pastas são locais ao app; tema e réguas acompanham o workspace.");});
        self.show_general = open;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_purposes_and_persistence() {
        let root = std::env::temp_dir().join(format!("noah-prefs-{:032x}", noah_storage::new_id()));
        std::fs::create_dir(&root).unwrap();
        let mut p = Preferences {
            file: Some(root.join("prefs.json")),
            ..Default::default()
        };
        p.remember(Purpose::Images, &root.join("photos"));
        p.remember(Purpose::Engine, &root.join("engine"));
        p.remember(Purpose::PreparationOutput, &root.join("prepared"));
        assert_eq!(p.directory(Purpose::Images), root.join("photos"));
        assert_eq!(p.directory(Purpose::Engine), root.join("engine"));
        p.engine = root.join("colmap");
        p.images = root.join("photos");
        p.save().unwrap();
        let (restored, error) = Preferences::load_file(Some(root.join("prefs.json")));
        assert!(error.is_none());
        assert_eq!(restored.engine, p.engine);
        assert_eq!(restored.images, p.images);
        assert_eq!(restored.directory(Purpose::Engine), root.join("engine"));
        assert_eq!(restored.directory(Purpose::Images), root.join("photos"));
        let v: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("prefs.json")).unwrap()).unwrap();
        assert_eq!(
            v["directories"]["images"],
            root.join("photos").to_str().unwrap()
        );
        assert_eq!(
            v["directories"]["preparation_output"],
            root.join("prepared").to_str().unwrap()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
