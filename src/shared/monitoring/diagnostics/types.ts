// 诊断事件类型：前端批量发送协议。
export type FrontendLogEntry = {
  runId: string;
  stepId: string;
  payload: unknown;
  round?: number;
  memberId?: string;
  clientTs: number;
  seq: number;
};
