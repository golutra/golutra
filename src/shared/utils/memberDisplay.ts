// 成员展示名称规则：统一使用成员名称，避免跨层依赖与类型耦合。
import type { MemberDisplaySource } from '@/shared/types/memberDisplay';

export const resolveMemberDisplayName = (member: MemberDisplaySource) => member.name;
