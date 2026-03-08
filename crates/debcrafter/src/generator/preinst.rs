use std::io;
use debcrafter::im_repr::PackageInstance;
use crate::codegen::LazyCreateBuilder;

pub fn generate(_instance: &PackageInstance, _out: LazyCreateBuilder) -> io::Result<()> {
    Ok(())
}
