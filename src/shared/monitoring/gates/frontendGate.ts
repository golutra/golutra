// 前端被动监控开关：统一由前端调试开关控制。
import { isFrontDebugEnabled } from '@/shared/monitoring/passiveMonitor';

export const isFrontendPassiveEnabled = () => isFrontDebugEnabled();
