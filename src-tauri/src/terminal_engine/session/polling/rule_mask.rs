//! 轮询规则开关：用于按触发事件选择执行范围。

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RuleMask(u8);

impl RuleMask {
  pub(crate) const STATUS_FALLBACK: RuleMask = RuleMask(1 << 0);
  pub(crate) const POST_READY: RuleMask = RuleMask(1 << 1);
  pub(crate) const SEMANTIC_FLUSH: RuleMask = RuleMask(1 << 3);

  pub(crate) fn contains(self, other: RuleMask) -> bool {
    (self.0 & other.0) == other.0
  }

  pub(crate) fn union(self, other: RuleMask) -> RuleMask {
    RuleMask(self.0 | other.0)
  }

  pub(crate) fn is_empty(self) -> bool {
    self.0 == 0
  }
}

impl std::ops::BitOr for RuleMask {
  type Output = RuleMask;

  fn bitor(self, rhs: RuleMask) -> RuleMask {
    self.union(rhs)
  }
}
