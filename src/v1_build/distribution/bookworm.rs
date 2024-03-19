use super::packager::{Packager, PackagerConfig};


pub struct BookwormPackager {
    config: BookwormPackagerConfig,
}
pub struct BookwormPackagerConfig {}

impl PackagerConfig for BookwormPackagerConfig {}

impl Packager for BookwormPackager {
    type Config = BookwormPackagerConfig;

    fn new(config: Self::Config) -> Self {
        return BookwormPackager { config };
    }

    fn package(&self) -> Result<bool, bool> {
        todo!()
    }
}
