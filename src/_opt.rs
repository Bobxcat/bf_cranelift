use crate::bf_ir::BfIrScope;

pub struct OptPasses {
    passes: Box<dyn Iterator<Item = OptPass>>,
    curr_pass: Option<OptPass>,
}

impl OptPasses {
    pub fn default_passes() -> Self {
        Self::from_passes([OptPass::CombineAdd, OptPass::CombinePtrAdd].into_iter())
    }
    pub fn from_passes(passes: impl Iterator<Item = OptPass> + 'static) -> Self {
        Self {
            passes: Box::new(passes),
            curr_pass: None,
        }
    }
    fn apply_curr(&self, sc: BfIrScope) -> BfIrScope {
        let pass = self.curr_pass.clone().unwrap();

        match &pass {
            OptPass::CombineAdd => sc.modify(|_toks| {
                // let mut i = 0;
                //
            }),
            OptPass::CombinePtrAdd => todo!(),
        }
    }
    pub fn apply_next(&mut self, sc: BfIrScope) -> Option<BfIrScope> {
        self.curr_pass = Some(self.passes.next()?);
        Some(self.apply_curr(sc))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptPass {
    CombineAdd,
    CombinePtrAdd,
}
