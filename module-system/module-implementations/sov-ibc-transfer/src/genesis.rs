use anyhow::Result;
use sov_modules_api::{Context, Module, WorkingSet};

use crate::Transfer;

impl<C: Context> Transfer<C> {
    pub(crate) fn init_module(
        &self,
        _config: &<Self as Module>::Config,
        _working_set: &mut WorkingSet<C>,
    ) -> Result<()> {
        Ok(())
    }
}
