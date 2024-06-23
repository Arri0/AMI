use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Default, Clone)]
pub struct VirtualPaths {
    paths: HashMap<PathBuf, PathBuf>,
}

impl VirtualPaths {
    pub fn insert(&mut self, virtual_path: PathBuf, real_path: PathBuf) {
        self.paths.insert(virtual_path, real_path);
    }

    // Translate virtual path to real one, if the resulting path is outside the base,
    // or the virtual path was not found, then return None
    pub fn translate(&self, path: &Path) -> Option<PathBuf> {
        for (vp, rp) in self.paths.iter() {
            if let Some(p) = remap_prefix(path, vp, rp) {
                return Some(p);
            }
        }
        None
    }

    pub fn translate_back(&self, path: &Path) -> Option<PathBuf> {
        for (vp, rp) in self.paths.iter() {
            if let Some(p) = remap_prefix(path, rp, vp) {
                return Some(p);
            }
        }
        None
    }
}

pub fn normalize_path(path: &Path) -> Option<PathBuf> {
    let mut test_path = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                if !test_path.pop() {
                    return None;
                }
            }
            std::path::Component::CurDir => {}
            _ => test_path.push(component.as_os_str()),
        }
    }
    Some(test_path)
}

pub fn is_path_within_base(path: &Path, base: &Path) -> bool {
    if let (Some(norm_path), Some(norm_base)) = (normalize_path(path), normalize_path(base)) {
        norm_path.starts_with(norm_base)
    } else {
        false
    }
}

pub fn remap_prefix(path: &Path, prefix: &Path, new_prefix: &Path) -> Option<PathBuf> {
    if !path.starts_with(prefix) {
        None
    } else {
        let prefix_len = prefix.components().count();
        let mut new_path = new_prefix.to_owned();
        new_path.extend(path.components().skip(prefix_len));
        Some(new_path)
    }
}

pub fn remove_prefix(path: &Path, prefix: &Path) -> PathBuf {
    if !path.starts_with(prefix) {
        path.to_owned()
    } else {
        let prefix_len = prefix.components().count();
        let mut new_path = PathBuf::new();
        new_path.extend(path.components().skip(prefix_len));
        new_path
    }
}

#[cfg(test)]
mod tests {
    use super::VirtualPaths;
    use std::path::{Path, PathBuf};

    #[test]
    fn translate() {
        let mut vp = VirtualPaths::default();
        vp.insert(PathBuf::from("samples:"), PathBuf::from("/samples"));
        vp.insert(PathBuf::from("projects:"), PathBuf::from("/projects"));

        assert_eq!(
            vp.translate(Path::new("samples:")),
            Some(PathBuf::from("/samples"))
        );
        assert_eq!(
            vp.translate(Path::new("projects:")),
            Some(PathBuf::from("/projects"))
        );
        assert_eq!(vp.translate(Path::new("dsd")), None);
        assert_eq!(
            vp.translate_back(Path::new("/projects")),
            Some(PathBuf::from("projects:"))
        );
    }

    #[test]
    fn remap_prefix() {
        let x = super::remap_prefix(
            Path::new("sample:/test/1/2/3"),
            Path::new("sample:/test"),
            Path::new("/sample"),
        );
        assert_eq!(x, Some(PathBuf::from("/sample/1/2/3")));
    }

    #[test]
    fn remove_prefix() {
        let x = super::remove_prefix(
            Path::new("sample:/test/1/2/3"),
            Path::new("sample:/test"),
        );
        assert_eq!(x, PathBuf::from("1/2/3"));
    }
}
