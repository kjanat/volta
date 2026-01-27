use std::path::PathBuf;

use super::executable;
use volta_layout_macro::layout;

pub use crate::v1::VoltaInstall;

layout! {
    pub struct VoltaHome {
        "cache": cache_dir {
            "node": node_cache_dir {
                "index.json": node_index_file;
                "index.json.expires": node_index_expiry_file;
            }
        }
        "bin": shim_dir {}
        "log": log_dir {}
        "tools": tools_dir {
            "inventory": inventory_dir {
                "node": node_inventory_dir {}
                "npm": npm_inventory_dir {}
                "packages": package_inventory_dir {}
                "yarn": yarn_inventory_dir {}
            }
            "image": image_dir {
                "node": node_image_root_dir {}
                "npm": npm_image_root_dir {}
                "yarn": yarn_image_root_dir {}
                "packages": package_image_root_dir {}
            }
            "user": default_toolchain_dir {
                "bins": default_bin_dir {}
                "packages": default_package_dir {}
                "platform.json": default_platform_file;
            }
        }
        "tmp": tmp_dir {}
        "hooks.json": default_hooks_file;
        "layout.v2": layout_file;
    }
}

impl VoltaHome {
    #[must_use]
    pub fn package_distro_file(&self, name: &str, version: &str) -> PathBuf {
        path_buf!(
            self.package_inventory_dir.clone(),
            format!("{}-{}.tgz", name, version)
        )
    }

    #[must_use]
    pub fn package_distro_shasum(&self, name: &str, version: &str) -> PathBuf {
        path_buf!(
            self.package_inventory_dir.clone(),
            format!("{}-{}.shasum", name, version)
        )
    }

    #[must_use]
    pub fn node_image_dir(&self, node: &str) -> PathBuf {
        path_buf!(self.node_image_root_dir.clone(), node)
    }

    #[must_use]
    pub fn npm_image_dir(&self, npm: &str) -> PathBuf {
        path_buf!(self.npm_image_root_dir.clone(), npm)
    }

    #[must_use]
    pub fn npm_image_bin_dir(&self, npm: &str) -> PathBuf {
        path_buf!(self.npm_image_dir(npm), "bin")
    }

    #[must_use]
    pub fn yarn_image_dir(&self, version: &str) -> PathBuf {
        path_buf!(self.yarn_image_root_dir.clone(), version)
    }

    #[must_use]
    pub fn yarn_image_bin_dir(&self, version: &str) -> PathBuf {
        path_buf!(self.yarn_image_dir(version), "bin")
    }

    #[must_use]
    pub fn package_image_dir(&self, name: &str, version: &str) -> PathBuf {
        path_buf!(self.package_image_root_dir.clone(), name, version)
    }

    #[must_use]
    pub fn default_package_config_file(&self, package_name: &str) -> PathBuf {
        path_buf!(
            self.default_package_dir.clone(),
            format!("{}.json", package_name)
        )
    }

    #[must_use]
    pub fn default_tool_bin_config(&self, bin_name: &str) -> PathBuf {
        path_buf!(self.default_bin_dir.clone(), format!("{}.json", bin_name))
    }

    #[must_use]
    pub fn node_npm_version_file(&self, version: &str) -> PathBuf {
        path_buf!(
            self.node_inventory_dir.clone(),
            format!("node-v{}-npm", version)
        )
    }

    #[must_use]
    pub fn shim_file(&self, toolname: &str) -> PathBuf {
        path_buf!(self.shim_dir.clone(), executable(toolname))
    }
}

#[cfg(windows)]
impl VoltaHome {
    #[must_use]
    pub fn shim_git_bash_script_file(&self, toolname: &str) -> PathBuf {
        path_buf!(self.shim_dir.clone(), toolname)
    }

    #[must_use]
    pub fn node_image_bin_dir(&self, node: &str) -> PathBuf {
        self.node_image_dir(node)
    }
}

#[cfg(unix)]
impl VoltaHome {
    #[must_use]
    pub fn node_image_bin_dir(&self, node: &str) -> PathBuf {
        path_buf!(self.node_image_dir(node), "bin")
    }
}
