use types::distribution::Distribution;

pub trait DistributionTrait{
    fn get_keyring(&self) -> &str;
    fn get_repo_url(&self) -> &str;
}

impl DistributionTrait for Distribution {
    fn get_keyring(&self) -> &str {
        match &self {
            Distribution::Debian(_) => "/usr/share/keyrings/debian-archive-keyring.gpg",
            Distribution::Ubuntu(_) => "/usr/share/keyrings/ubuntu-archive-keyring.gpg",
        }
    }
    
    fn get_repo_url(&self) -> &str {
        match &self {
            Distribution::Debian(_) => "http://deb.debian.org/debian",
            Distribution::Ubuntu(_) => "http://archive.ubuntu.com/ubuntu",
        }
    }
}