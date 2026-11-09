//! Entidades de domínio; nenhuma dependência de arquivos, sensores ou UI.
pub mod reconstruction;
use noah_geometry::{PointCloud, Transform, UnitState};

/// Identidade fornecida pela aplicação, independente do nome e caminho.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectId(pub u128);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub u128);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetId(pub u128);

/// Referencial canônico: destro, Z para cima, coordenadas XYZ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceFrame {
    RightHandedZUp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryAsset {
    pub id: AssetId,
    pub cloud: PointCloud,
    pub unit: UnitState,
    pub frame: ReferenceFrame,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub id: ObjectId,
    pub name: String,
    pub asset_id: AssetId,
    pub transform: Transform,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    objects: Vec<Object>,
    assets: Vec<GeometryAsset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectError {
    DuplicateObject,
    DuplicateAsset,
    MissingAsset,
}

impl Project {
    pub fn new(id: ProjectId, name: String) -> Self {
        Self {
            id,
            name,
            objects: Vec::new(),
            assets: Vec::new(),
        }
    }
    pub fn objects(&self) -> &[Object] {
        &self.objects
    }
    pub fn assets(&self) -> &[GeometryAsset] {
        &self.assets
    }
    pub fn set_transform(&mut self, id: ObjectId, transform: Transform) -> bool {
        if let Some(object) = self.objects.iter_mut().find(|o| o.id == id) {
            object.transform = transform;
            true
        } else {
            false
        }
    }
    pub fn add_asset(&mut self, asset: GeometryAsset) -> Result<(), ProjectError> {
        if self.assets.iter().any(|a| a.id == asset.id) {
            return Err(ProjectError::DuplicateAsset);
        }
        self.assets.push(asset);
        Ok(())
    }
    pub fn add_object(&mut self, object: Object) -> Result<(), ProjectError> {
        if self.objects.iter().any(|o| o.id == object.id) {
            return Err(ProjectError::DuplicateObject);
        }
        if !self.assets.iter().any(|a| a.id == object.asset_id) {
            return Err(ProjectError::MissingAsset);
        }
        self.objects.push(object);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identities_and_references_are_preserved() {
        let mut project = Project::new(ProjectId(1), "Teste".into());
        let asset = GeometryAsset {
            id: AssetId(2),
            cloud: PointCloud::new(vec![[0.; 3]], None).unwrap(),
            unit: UnitState::default(),
            frame: ReferenceFrame::RightHandedZUp,
        };
        let object = Object {
            id: ObjectId(3),
            name: "Peça".into(),
            asset_id: asset.id,
            transform: Transform::default(),
        };
        assert_eq!(
            project.add_object(object.clone()),
            Err(ProjectError::MissingAsset)
        );
        project.add_asset(asset.clone()).unwrap();
        assert_eq!(project.add_asset(asset), Err(ProjectError::DuplicateAsset));
        project.add_object(object.clone()).unwrap();
        assert_eq!(
            project.add_object(object),
            Err(ProjectError::DuplicateObject)
        );
        project.name = "Outro nome".into();
        assert_eq!(project.id, ProjectId(1));
        assert_eq!(project.objects()[0].id, ObjectId(3));
        assert_eq!(project.assets()[0].unit, UnitState::Unknown);
        assert_eq!(project.objects().len(), 1);
    }
}
