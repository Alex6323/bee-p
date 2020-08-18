pub struct TipSelector {
    non_lazy_tips: NonLazyTipPool,
    semi_lazy_tips: SemiLazyTipPool,
}

impl TipSelector {
    pub fn new() -> Self {
        Self {
            non_lazy_tips: NonLazyTipPool {},
            semi_lazy_tips: SemiLazyTipPool {}
        }
    }
}

struct NonLazyTipPool {

}

struct SemiLazyTipPool {

}