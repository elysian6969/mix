use crate::atom::Atom;
use crate::config::Config;
use crate::package::{Graph, PackageId};
use crate::shell::Text;
use crossterm::style::Colorize;
use std::collections::HashSet;

pub async fn install(config: &Config, atoms: HashSet<Atom>) -> crate::Result<()> {
    let repositories = config.repositories().keys();
    let graph = Graph::open(repositories).await?;
    //let mut fetch_list = Vec::new();

    for atom in atoms {
        let package_id = PackageId::new(atom.name);
        let order = graph.dependency_order(&package_id);

        for package_id in order {
            let (node, _relationships) = graph.get(&package_id).expect("should always be some");

            for source in node.metadata.source.iter() {
                /*let scheme = source.url.scheme();
                let path = source.url.path();
                let url = match scheme {
                    "github" => format!("https://github.com/{path}.git"),
                    "kernel" => {
                        format!("https://git.kernel.org/pub/scm/linux/kernel/git/{path}.git")
                    }
                    "savannah" => {
                        format!("https://git.savannah.gnu.org/git/{path}.git")
                    }
                    "sourceware" => {
                        format!("https://sourceware.org/git/{path}.git")
                    }
                    url => {
                        println!("{url}");
                        continue;
                    }
                };

                fetch_list.push((url, node));*/
            }
        }
    }

    /*fetch_list.dedup_by(|(a, _), (b, _)| a == b);

    for (url, node) in fetch_list {
        let format = format!(
            "{}/{} {url}\n",
            node.group_id.as_str().green(),
            node.package_id.as_str().green()
        );

        Text::new(format).render(config.shell()).await?;
    }*/

    Ok(())
}

pub enum Source {
    /// https  version  https://api.github.com/repos/{user}/{repository}/tags
    /// https  release  https://api.github.com/repos/{user}/{repository}/releases
    /// git    source   https://github.com/{user}/{repository}.git
    Github {
        user: String,
        repository: String,
    },

    /// git    source   https://gitlab.com/{user}/{repository}.git
    Gitlab {
        user: String,
        repository: String,
    },

    /// git    source   https://git.kernel.org/pub/scm/linux/kernel/git/{user}/{repository}.git
    Kernel {
        user: String,
        repository: String,
    },

    /// git    source   https://git.savannah.gnu.org/git/{repository}.git
    Savannah {
        repository: String,
    },

    /// git    source   https://sourceware.org/git/{repository}.git
    Sourceware {
        repository: String,
    },

    // GCC mirror
    // ftp://ftp.lip6.fr/pub/gcc/releases/gcc-11.1.0/gcc-11.1.0.tar.xz
    Lip6 {},
}

pub enum Mirror {
    Lip6,
    Irisa,
    Uvsq,
    FuBerlin,
    Gwdg,
    Mpg,
    CyberMirror,
    Ntua,
    RobotLab,
    Tsukuba,
    Marwan,
    Koddos,
    Ia64,
    Fyxm,
    MirrorService,
    BigSearcher,
    NetGull,
    ConcertPass,
}
