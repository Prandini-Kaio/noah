use noah_core::ObjectId;
use noah_geometry::{Transform, UnitState};
use noah_storage::{Result, StoredProject};
use std::{env, path::Path, process::ExitCode};

const HELP: &str = "NOAH 2.0 — E1/E2 local\n\
create <projeto> <nome>\n\
import <projeto> <arquivo.ply> <nome> --frame z-up [--unit unknown|m|mm]\n\
transform <projeto> <object-id> <tx> <ty> <tz> <rx> <ry> <rz> <escala>\n\
inspect <projeto>\n\
validate <projeto>\n\
export <projeto> <object-id> <novo.ply>\n\
reconstruct <fotos> <novo-run> --colmap <executável> [--dense] [--mesh] [--timeout <segundos>]\n\
mesh-from-run <run> <novo-run> --colmap <executável> [--poisson] [--timeout <segundos>]\n\
run-inspect <run>\n\
run-cancel <run>\n\
Ângulos em radianos, Euler XYZ; transform substitui a transformação da instância.\n\
Um escritor por projeto. Export aplica transformação; PLY não transporta unidade/frame.";
fn object_id(text: &str) -> Result<ObjectId> {
    if text.len() != 32 {
        return Err("object-id exige 32 dígitos hexadecimais".into());
    }
    Ok(ObjectId(u128::from_str_radix(text, 16)?))
}
fn run(args: &[String]) -> Result<()> {
    if args.is_empty() || args == ["--help"] || args == ["help"] {
        println!("{HELP}");
        return Ok(());
    }
    match args[0].as_str() {
        "reconstruct" if args.len() >= 5 => {
            if args[3] != "--colmap" {
                return Err("declare --colmap <executável local>".into());
            }
            let mut dense = false;
            let mut mesh = false;
            let mut timeout = 600u64;
            let mut i = 5;
            while i < args.len() {
                match args[i].as_str() {
                    "--mesh" if !mesh => {
                        mesh = true;
                        i += 1;
                    }
                    "--dense" if !dense => {
                        dense = true;
                        i += 1;
                    }
                    "--timeout" if i + 1 < args.len() => {
                        timeout = args[i + 1].parse()?;
                        i += 2;
                    }
                    _ => return Err("opção de reconstrução inválida".into()),
                }
            }
            noah_photogrammetry::reconstruct(noah_photogrammetry::Config {
                engine: args[4].clone().into(),
                images: args[1].clone().into(),
                output: args[2].clone().into(),
                dense,
                mesh,
                timeout: std::time::Duration::from_secs(timeout),
            })?;
            println!("run concluído; exports em {}/exports", args[2]);
        }
        "mesh-from-run" if args.len() >= 5 => {
            if args[3] != "--colmap" {
                return Err("declare --colmap <executável local>".into());
            }
            let mut method = noah_photogrammetry::surface::Method::DelaunaySparse;
            let mut timeout = 600;
            let mut i = 5;
            while i < args.len() {
                match args[i].as_str() {
                    "--poisson"
                        if method == noah_photogrammetry::surface::Method::DelaunaySparse =>
                    {
                        method = noah_photogrammetry::surface::Method::PoissonDense;
                        i += 1;
                    }
                    "--timeout" if i + 1 < args.len() => {
                        timeout = args[i + 1].parse()?;
                        i += 2;
                    }
                    _ => return Err("opção de malha inválida".into()),
                }
            }
            noah_photogrammetry::surface::from_run(noah_photogrammetry::surface::Config {
                source: args[1].clone().into(),
                output: args[2].clone().into(),
                engine: args[4].clone().into(),
                method,
                timeout: std::time::Duration::from_secs(timeout),
            })?;
            println!("superfície concluída; OBJ em {}/exports/mesh.obj", args[2]);
        }
        "run-inspect" if args.len() == 2 => {
            let manifest = noah_pipeline::inspect(Path::new(&args[1]))?;
            println!(
                "run_id={} status={:?} images={} steps={} artifacts={} engine={:?} diagnostic={:?}",
                manifest.run_id,
                manifest.status,
                manifest.inputs.len(),
                manifest.steps.len(),
                manifest.artifacts.len(),
                manifest.engine_version,
                manifest.diagnostic
            );
            for step in manifest.steps {
                println!(
                    "step={} status={:?} exit={:?} diagnostic={:?} stdout={} stderr={}",
                    step.name,
                    step.status,
                    step.exit_code,
                    step.diagnostic,
                    step.stdout,
                    step.stderr
                );
            }
        }
        "run-cancel" if args.len() == 2 => {
            noah_pipeline::cancel(Path::new(&args[1]))?;
            println!("cancelamento solicitado");
        }
        "create" if args.len() == 3 => {
            let stored = StoredProject::create(Path::new(&args[1]), args[2].clone())?;
            println!("project_id={:032x}", stored.project().id.0);
        }
        "import" if args.len() == 6 || args.len() == 8 => {
            if args[4] != "--frame" || args[5] != "z-up" {
                return Err("declare --frame z-up; PLY não informa o referencial".into());
            }
            let unit = if args.len() == 8 {
                if args[6] != "--unit" {
                    return Err("esperado --unit unknown|m|mm".into());
                }
                match args[7].as_str() {
                    "unknown" => UnitState::Unknown,
                    "m" => UnitState::Metre,
                    "mm" => UnitState::Millimetre,
                    _ => return Err("unidade não suportada".into()),
                }
            } else {
                UnitState::Unknown
            };
            let mut stored = StoredProject::open(Path::new(&args[1]))?;
            let id = stored.import(Path::new(&args[2]), args[3].clone(), unit)?;
            println!("object_id={:032x}", id.0);
        }
        "transform" if args.len() == 10 => {
            let mut values = [0.; 7];
            for (v, text) in values.iter_mut().zip(&args[3..]) {
                *v = text.parse()?;
            }
            let t = Transform::from_euler(
                [values[0], values[1], values[2]],
                [values[3], values[4], values[5]],
                values[6],
            )?;
            StoredProject::open(Path::new(&args[1]))?.transform(object_id(&args[2])?, t)?;
            println!("transformação salva");
        }
        "inspect" if args.len() == 2 => {
            let stored = StoredProject::open(Path::new(&args[1]))?;
            let p = stored.project();
            println!(
                "NOAH 2.0 schema=1 project_id={:032x} name={:?} objects={} assets={}",
                p.id.0,
                p.name,
                p.objects().len(),
                p.assets().len()
            );
            for object in p.objects() {
                let asset = p
                    .assets()
                    .iter()
                    .find(|a| a.id == object.asset_id)
                    .ok_or("asset ausente")?;
                println!(
                    "object_id={:032x} name={:?} asset_id={:032x} points={} unit={:?} frame={:?} translation={:?} rotation={:?} scale={}",
                    object.id.0,
                    object.name,
                    asset.id.0,
                    asset.cloud.positions().len(),
                    asset.unit,
                    asset.frame,
                    object.transform.translation(),
                    object.transform.rotation(),
                    object.transform.scale()
                );
            }
        }
        "validate" if args.len() == 2 => {
            StoredProject::open(Path::new(&args[1]))?;
            println!("projeto válido: schema, referências, geometria e hashes conferidos");
        }
        "export" if args.len() == 4 => {
            StoredProject::open(Path::new(&args[1]))?
                .export(object_id(&args[2])?, Path::new(&args[3]))?;
            println!(
                "PLY exportado; unidade e referencial devem acompanhar o arquivo separadamente"
            );
        }
        _ => return Err(format!("comando ou argumentos inválidos\n{HELP}").into()),
    }
    Ok(())
}
fn main() -> ExitCode {
    match run(&env::args().skip(1).collect::<Vec<_>>()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("NOAH: {error}");
            ExitCode::FAILURE
        }
    }
}
