pub fn select_distro_etc_release(release: &str) -> Option<Box<dyn Distro>> {
    let r = release.to_lowercase();

    if r.contains("ubuntu") {
        Some(Box::new(Debian))
    } else if r.contains("debian") {
        Some(Box::new(Debian))
    } else if r.contains("fedora") {
        Some(Box::new(Fedora))
    } else if r.contains("centos") || r.contains("rhel") || r.contains("red hat") {
        Some(Box::new(Fedora))
    } else if r.contains("arch") {
        Some(Box::new(Arch))
    } else if r.contains("alpine") {
        Some(Box::new(Alpine))
    } else {
        None
    }
}

pub trait Distro {
    fn install_deps_command(&self) -> Box<str>;
}

pub struct Debian;
impl Distro for Debian {
    fn install_deps_command(&self) -> Box<str> {
        "sudo apt install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin".into()
    }
}

pub type Ubuntu = Debian;

pub struct Fedora;
impl Distro for Fedora {
    fn install_deps_command(&self) -> Box<str> {
        "sudo dnf install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin".into()
    }
}

pub type CentOS = Fedora;
pub type Rhel = Fedora;

pub struct Arch;
impl Distro for Arch {
    fn install_deps_command(&self) -> Box<str> {
        "sudo pacman -S docker docker-buildx docker-compose".into()
    }
}

pub struct Alpine;
impl Distro for Alpine {
    fn install_deps_command(&self) -> Box<str> {
        "sudo apk add docker docker-cli-compose".into()
    }
}
