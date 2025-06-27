use fs_extra::dir::get_dir_content;
use hashbrown::hash_map::HashMap;
use rayon::prelude::*;
use std::fs;
use tracing::*;

/// Implementation of resource loader for model
#[cfg(feature = "model-resource-manager")]
pub mod model;

/// Implementation of resource loader for texture
#[cfg(feature = "texture-resource-manager")]
pub mod texture;

/// Identifier for the resources
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Identifier {
    /// Root folder of the resource
    /// First folder to search
    /// Example 'assets' or 'resources'
    pub root: String,
    /// Group of the resource
    /// Folder inside root, 'textures' or 'models'
    pub group: String,
    /// Id of the resource
    /// File inside the group folder, or alone file inside the root folder
    pub id: String,
}
impl Identifier {
    /// Convert a path into a Indentifier
    /// Example
    /// ```rust
    /// let identifier = Indentifier::parse_from_str("assets/textures/tree.png");
    /// assert_eq!(identifier.root, "assets");
    /// assert_eq!(identifier.group, "textures");
    /// assert_eq!(identifier.id, "tree.png");
    /// ```
    /// If the path doesnt has a group
    /// ```rust
    /// let identifier = Identifier::parse_from_str("assets/cube.obj");
    /// assert_eq!(identifier.root, "assets");
    /// assert_eq!(identifier.group, "");
    /// assert_eq!(identifier.id, "tree.png")
    /// ```
    ///
    pub fn parse_from_str(id: &str) -> Self {
        let seg: Vec<&str> = id.splitn(3, "/").collect();
        if seg.len() < 3 {
            let root = seg[0].to_owned();
            let id = seg[1].to_owned();
            return Self {
                root,
                group: String::new(),
                id,
            };
        }
        let root = seg[0].to_owned();
        let group = seg[1].to_owned();
        let id = seg[2].to_owned();
        Self { root, group, id }
    }
    pub fn root(&self) -> String {
        self.root.clone()
    }
    pub fn group(&self) -> String {
        self.group.clone()
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub fn new(&self, root: &str, group: &str, id: &str) -> Self {
        Self {
            root: root.to_owned(),
            group: group.to_owned(),
            id: id.to_owned(),
        }
    }
}
impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}-{}:{}", self.root(), self.group(), self.id()))
    }
}

/// Abstraction of a resource loader
pub trait ResourceLoader: Send + Sync {
    type Resource: Send + Sync;
    type Error: std::error::Error
        + Send
        + Sync
        + 'static
        + From<std::io::Error>
        + From<fs_extra::error::Error>;
    /// Implemented by default, load a file from path
    fn load_from_path(&self, path: &str) -> Result<Self::Resource, Self::Error> {
        let file = fs::read(path)?;
        self.load_from_bytes(file)
    }
    /// Load a file system into a hashmap
    fn load_all(&self, root: &str) -> Result<HashMap<Identifier, Self::Resource>, Self::Error> {
        let entries = get_dir_content(root)?;
        let result: HashMap<Identifier, Self::Resource> = entries
            .files
            .par_iter()
            .filter_map(|entry| {
                debug!(
                    "READ ENTRY \"{}\" WITH RESOURCE LOADER \"{}\"",
                    entry,
                    self.label()
                );
                let id = Identifier::parse_from_str(entry);
                let res = match self.load_from_path(entry) {
                    Ok(res) => res,
                    Err(err) => {
                        error!("Error loading entry in \"{}\" - {}", self.label(), err);
                        return None;
                    }
                };
                Some((id, res))
            })
            .collect();
        Ok(result)
    }
    /// load a file from the bytes
    fn load_from_bytes(&self, bytes: Vec<u8>) -> Result<Self::Resource, Self::Error>;
    /// Name of the resource loader
    fn label(&self) -> &'static str;
}
